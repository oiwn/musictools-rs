//! Dark Neon editor: one impulse-response graph over a tabbed control band.
//!
//! The probe is the point of the plugin's face — a plate is hard to judge from
//! knob positions alone, so the editor renders the current settings' impulse
//! response off-thread and draws it as one graph: the log-frequency
//! spectrogram as a dim backdrop, the L/R decay envelopes glowing on top.
//! Below it, Dry/Wet and Freeze stay in reach while PLATE/TONE/MOD tabs swap
//! the knob row between.

use std::sync::Arc;
use std::sync::TryLockError;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use egui::{
    Align2, Color32, ColorImage, FontId, Painter, Pos2, Rect, Stroke, TextureHandle,
    TextureOptions, Vec2,
};
use kirya_dsp::analysis::{FLOOR_DB, IR_WINDOW_STOPS, IrRender};
use neon_ui::bindings::{param_knob, param_toggle};
use neon_ui::knob::KnobSize;
use neon_ui::{button, fonts, glow, palette, surface, tabs, text};
use nice_plug::context::gui::{AsyncExecutor, GuiContext, ParamSetter};
use nice_plug::prelude::FloatParam;
use nice_plug_egui::baseview::WindowSize;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, Frame, NiceEguiApp, RepaintNotifier,
    create_egui_editor,
};

use crate::{Kirya, KiryaParams, RenderIr};

/// Logical window size.
const EDITOR_SIZE: (f32, f32) = (720.0, 540.0);

// Vertical budget, top to bottom (see `vertical_budget`).
const MARGIN: f32 = 12.0;
const HEADER_HEIGHT: f32 = 32.0;
const HEADER_GAP: f32 = 10.0;
const GRAPH_HEIGHT: f32 = 312.0;
const GRAPH_GAP: f32 = 12.0;
/// The kit's horizontal tab strip height.
const TABS_HEIGHT: f32 = 26.0;
/// Clear air between the tab strip and the knob arcs below it.
const TABS_GAP: f32 = 16.0;
/// Inner padding of the control band panel.
const BAND_PAD: f32 = 10.0;

/// Gap between knobs in a row.
const GAP: f32 = 16.0;
/// Freeze toggle width before the first frame measures it.
const TOGGLE_WIDTH_GUESS: f32 = 84.0;

/// Quietest level the graph plots. Anything below reads as the floor.
const DISPLAY_FLOOR_DB: f32 = -70.0;

/// How long a knob must sit still before a re-render is posted. A drag that
/// outruns this collapses into one render (see `RenderIr::run`).
const DEBOUNCE: Duration = Duration::from_millis(150);

/// The editor polls for a finished render, so it needs a steady repaint.
const REPAINT_INTERVAL: Duration = Duration::from_millis(33);

const LEFT_COLOR: Color32 = palette::CYAN;
const RIGHT_COLOR: Color32 = palette::ORANGE;

/// Tabs over the centre knob row.
const TAB_LABELS: [&str; 3] = ["PLATE", "TONE", "MOD"];
/// Most knobs a tab holds; the row is laid out for this many slots.
const TAB_SLOTS: usize = 4;

/// Knob width and full knob column height.
fn knob_size() -> Vec2 {
    KnobSize::Medium.footprint()
}

/// Height of the control band panel: tab strip, gap, knob row, padding.
fn band_height() -> f32 {
    BAND_PAD + TABS_HEIGHT + TABS_GAP + knob_size().y + BAND_PAD
}

/// Total height the layout uses; must land just under the window height.
fn vertical_budget() -> f32 {
    MARGIN + HEADER_HEIGHT + HEADER_GAP + GRAPH_HEIGHT + GRAPH_GAP + band_height() + MARGIN
}

/// Knobs that stay visible outside the tabs.
fn fixed_knobs(params: &KiryaParams) -> [(&FloatParam, &'static str); 2] {
    [(&params.dry, "Dry"), (&params.wet, "Wet")]
}

/// The knobs a tab shows, at most [`TAB_SLOTS`].
fn tab_knobs(params: &KiryaParams, tab: usize) -> Vec<(&FloatParam, &'static str)> {
    match tab {
        0 => vec![
            (&params.pre_delay, "Pre-Dly"),
            (&params.size, "Size"),
            (&params.diffusion, "Diffuse"),
            (&params.decay, "Decay"),
        ],
        1 => vec![
            (&params.input_low_cut, "In Lo"),
            (&params.input_high_cut, "In Hi"),
            (&params.reverb_low_cut, "Rev Lo"),
            (&params.reverb_high_cut, "Rev Hi"),
        ],
        _ => vec![
            (&params.mod_rate, "Rate"),
            (&params.mod_depth, "Depth"),
            (&params.mod_shape, "Shape"),
        ],
    }
}

/// The parameters an impulse-response render actually depends on.
///
/// Dry, Wet and Freeze are excluded on purpose: the probe renders wet-only
/// with Freeze forced off, so moving those would only trigger renders that
/// produce a pixel-identical picture.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RenderKey {
    pre_delay_ms: f32,
    size: f32,
    diffusion: f32,
    decay: f32,
    input_low_cut_hz: f32,
    input_high_cut_hz: f32,
    reverb_low_cut_hz: f32,
    reverb_high_cut_hz: f32,
    mod_rate_hz: f32,
    mod_depth: f32,
    mod_shape: f32,
    sample_rate: f64,
}

impl RenderKey {
    fn read(params: &KiryaParams, sample_rate: f64) -> Self {
        let settings = params.target_settings();
        Self {
            pre_delay_ms: settings.pre_delay_ms,
            size: settings.size,
            diffusion: settings.diffusion,
            decay: settings.decay,
            input_low_cut_hz: settings.input_low_cut_hz,
            input_high_cut_hz: settings.input_high_cut_hz,
            reverb_low_cut_hz: settings.reverb_low_cut_hz,
            reverb_high_cut_hz: settings.reverb_high_cut_hz,
            mod_rate_hz: settings.mod_rate_hz,
            mod_depth: settings.mod_depth,
            mod_shape: settings.mod_shape,
            sample_rate,
        }
    }
}

pub struct KiryaEditor {
    params: Arc<KiryaParams>,
    /// Host context while the window is open; `None` before `build`.
    gui: Option<GuiContext>,
    /// Posts IR renders to the plugin's background executor.
    executor: AsyncExecutor<Kirya>,
    repaint: RepaintNotifier,
    /// Selected control tab (an editor view, not a host parameter).
    tab: usize,
    /// "?" swaps the graph and controls for a text help screen.
    show_help: bool,
    /// Freeze toggle width measured last frame, to place it exactly.
    toggle_width: f32,
    /// Most recent finished render, kept locally so drawing never holds the
    /// handoff lock.
    render: Option<Box<IrRender>>,
    /// Spectrogram uploaded once per render, not rebuilt per frame.
    texture: Option<TextureHandle>,
    /// Generation of the render currently held, so a repeat is not re-uploaded.
    generation: u64,
    /// Settings the last request was posted for.
    requested: Option<RenderKey>,
    /// When the settings last changed, for the debounce.
    changed_at: Option<Instant>,
}

impl KiryaEditor {
    fn new(
        params: Arc<KiryaParams>,
        executor: AsyncExecutor<Kirya>,
        repaint: RepaintNotifier,
    ) -> Self {
        Self {
            params,
            gui: None,
            executor,
            repaint,
            tab: 0,
            show_help: false,
            toggle_width: TOGGLE_WIDTH_GUESS,
            render: None,
            texture: None,
            generation: 0,
            requested: None,
            changed_at: None,
        }
    }

    /// Host rate if the plugin has been activated, else a sensible stand-in so
    /// the probe still draws something before the first activation.
    fn sample_rate(params: &KiryaParams) -> f64 {
        let bits = params.sample_rate_bits.load(Ordering::Relaxed);
        let rate = f64::from_bits(bits);
        if rate.is_finite() && rate > 0.0 {
            rate
        } else {
            48_000.0
        }
    }

    /// Post a render when the settings have been still for [`DEBOUNCE`].
    fn maybe_request(&mut self, ui: &egui::Ui) {
        let sample_rate = Self::sample_rate(&self.params);
        let key = RenderKey::read(&self.params, sample_rate);

        if self.requested != Some(key) {
            self.requested = Some(key);
            self.changed_at = Some(Instant::now());
        }

        let Some(changed_at) = self.changed_at else {
            return;
        };
        if changed_at.elapsed() < DEBOUNCE {
            // Make sure a frame lands once the debounce expires even if the
            // user has stopped moving the mouse.
            ui.ctx().request_repaint_after(DEBOUNCE);
            return;
        }

        // A fresh request supersedes any still queued, so a backlog renders
        // once (see `RenderIr::run`).
        self.executor
            .execute_background(RenderIr::new(&self.params, sample_rate));
        self.changed_at = None;
    }

    /// Pick up a finished render and upload its spectrogram exactly once.
    fn collect_render(&mut self, ui: &egui::Ui) {
        let params = &self.params;
        let generation = params.ir_generation.load(Ordering::Acquire);
        if generation == self.generation {
            return;
        }

        // `try_lock` keeps a busy worker from stalling the GUI thread; the
        // next frame picks the render up instead. A poisoned lock is still
        // worth reading — the slot is a plain cache, so the worst a panicking
        // writer left behind is a stale render.
        let mut slot = match params.ir_slot.try_lock() {
            Ok(slot) => slot,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return,
        };
        let Some(render) = slot.take() else {
            return;
        };
        drop(slot);

        self.texture = Some(ui.ctx().load_texture(
            "kirya-spectrogram",
            spectrogram_image(&render),
            TextureOptions::LINEAR,
        ));
        self.render = Some(render);
        self.generation = generation;
    }

    /// Seconds the graph currently spans. The render picks its own window from
    /// the reverb time, so the axis follows the data rather than a constant.
    fn duration(&self) -> f32 {
        #[allow(clippy::cast_possible_truncation)]
        self.render
            .as_ref()
            .map_or(IR_WINDOW_STOPS[0] as f32, |render| {
                render.duration_seconds()
            })
    }
}

impl NiceEguiApp for KiryaEditor {
    fn build(
        &mut self,
        egui_ctx: egui::Context,
        nice_gui_ctx: GuiContext,
        _frame: &mut Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        egui_ctx.set_visuals(neon_ui::visuals::dark_neon_visuals());
        fonts::install(&egui_ctx);
        self.gui = Some(nice_gui_ctx);
        // Forget what was requested and the uploaded texture (it belonged to
        // the previous window's context), so the first frame posts a render
        // for whatever the parameters are now.
        self.requested = None;
        self.changed_at = None;
        self.texture = None;
        self.generation = 0;
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.gui = None;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let Some(gui) = self.gui.clone() else {
            return;
        };
        // Ping the window's repaint poll every frame: hosts repaint lazily,
        // and the probe has to pick up finished renders without OS events.
        self.repaint.request_repaint();
        ui.ctx().request_repaint_after(REPAINT_INTERVAL);
        self.maybe_request(ui);
        self.collect_render(ui);
        let setter = gui.param_setter();
        self.draw(ui, &setter);
    }
}

impl KiryaEditor {
    /// Paint the whole window. Host-free apart from `setter`, so it can be
    /// rendered offscreen.
    fn draw(&mut self, ui: &mut egui::Ui, setter: &ParamSetter<'_>) {
        // Every block sits in an exact rect computed from the budget above,
        // so nothing depends on layout-engine centring or spacing.
        let window = ui.max_rect();
        ui.painter().rect_filled(window, 0.0, palette::BG);
        let left = window.left() + MARGIN;
        let width = window.width() - 2.0 * MARGIN;
        let mut top = window.top() + MARGIN;
        let mut take = |height: f32, gap: f32| {
            let rect = Rect::from_min_size(Pos2::new(left, top), Vec2::new(width, height));
            top += height + gap;
            rect
        };
        let header = take(HEADER_HEIGHT, HEADER_GAP);
        let graph = take(GRAPH_HEIGHT, GRAPH_GAP);
        let band = take(band_height(), 0.0);

        draw_header(&ui.painter_at(header), header);
        // The help toggle sits in the header's right end, in its own exact
        // rect, so it never shifts the title.
        let help_rect = Rect::from_center_size(
            Pos2::new(header.right() - 6.0 - 13.0, header.center().y),
            Vec2::splat(26.0),
        );
        let mut help_ui = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(ui.scope_id().with("help-button"))
                .max_rect(help_rect),
        );
        let glyph = if self.show_help { "×" } else { "?" };
        if button::icon_button(&mut help_ui, glyph).clicked() {
            self.show_help = !self.show_help;
        }

        if self.show_help {
            draw_help(ui, graph.union(band));
            return;
        }
        self.draw_graph(&ui.painter_at(graph), graph);
        self.draw_controls(ui, setter, band);
    }
}

/// Help copy: a section heading, then its lines. Mirrors `specs/kirya.md`.
const HELP: &[(&str, &[&str])] = &[
    (
        "PPC KIRYA — PLATE REVERB",
        &[
            "A clean-room take on Jon Dattorro's plate from \"Effect Design Part 1\" (1997), \
             with the Plateau control set. Built for turning arpeggios into pads.",
            "The graph is the current settings' impulse response: L and R decay curves \
             over a dim spectrogram (low notes at the bottom), with the measured RT60. It \
             re-renders off the audio thread shortly after the knobs settle — wet only, \
             Freeze off. RT60 reads \"> N s\" when the tail is too long to measure in the \
             window.",
        ],
    ),
    (
        "MIX",
        &[
            "DRY — level of the untouched input.",
            "WET — level of the reverb.",
        ],
    ),
    (
        "PLATE",
        &[
            "PRE-DLY — gap before the plate answers, 0–500 ms.",
            "SIZE — scales every delay in the plate, 0.05–4. Moving it glides like tape.",
            "DIFFUSE — echo density: low is grainy and discrete, high is a smooth wash.",
            "DECAY — tail length. Most of the travel sits in the long-tail region.",
        ],
    ),
    (
        "TONE",
        &[
            "IN LO / IN HI — cut lows and highs before the plate.",
            "REV LO / REV HI — damping inside the plate: every pass loses lows or highs, \
             so the tail thins or darkens as it decays.",
        ],
    ),
    (
        "MOD",
        &[
            "RATE — speed of the four delay LFOs, 0.01–20 Hz.",
            "DEPTH — how far they move the delays; smears metallic ringing into chorus.",
            "SHAPE — LFO waveform, triangle at 0% to sine at 100%.",
        ],
    ),
    (
        "HOLD",
        &[
            "FREEZE — holds the tail forever. The input stays live, so you can layer new \
           material into a frozen pad.",
        ],
    ),
    (
        "TIPS",
        &["Double-click (or ctrl/⌘-click) a knob to reset it to its default."],
    ),
];

/// Text help in place of the graph and controls.
fn draw_help(ui: &mut egui::Ui, rect: Rect) {
    surface::inset(ui.painter(), rect);
    let mut help = ui.new_child(
        egui::UiBuilder::new()
            .scope_id(ui.scope_id().with("help"))
            .max_rect(rect.shrink2(Vec2::new(18.0, 14.0))),
    );
    egui::ScrollArea::vertical()
        .id_salt("kirya-help")
        .auto_shrink([false, false])
        .show(&mut help, |ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            for (index, (heading, lines)) in HELP.iter().enumerate() {
                if index > 0 {
                    ui.add_space(6.0);
                }
                ui.label(
                    egui::RichText::new(*heading)
                        .font(fonts::heading(11.0))
                        .extra_letter_spacing(1.2)
                        .color(palette::ACCENT),
                );
                for line in *lines {
                    ui.label(
                        egui::RichText::new(*line)
                            .size(12.0)
                            .color(palette::TEXT_SECONDARY),
                    );
                }
            }
        });
}

/// Title strip: tracked name, Muted subtitle with the build stamp so the
/// loaded bundle is identifiable.
fn draw_header(painter: &Painter, rect: Rect) {
    surface::panel_lit(painter, rect);
    let title = text::tracked(
        painter,
        Pos2::new(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        "PPC KIRYA",
        fonts::heading(13.0),
        palette::TEXT_PRIMARY,
        1.5,
    );
    painter.text(
        Pos2::new(title.right() + 10.0, rect.center().y + 1.0),
        Align2::LEFT_CENTER,
        concat!("plate reverb · v", env!("CARGO_PKG_VERSION")),
        FontId::proportional(10.0),
        palette::TEXT_MUTED,
    );
}

impl KiryaEditor {
    /// The combined IR graph: dim spectrogram backdrop, dB grid, time ruler,
    /// glowing L/R envelopes, and the legend with the RT60 readout.
    fn draw_graph(&self, painter: &Painter, rect: Rect) {
        surface::inset(painter, rect);
        // Axis labels live in gutters around the plot, never on the data:
        // legend on top, dB on the left, Hz on the right, time below.
        let plot = Rect::from_min_max(
            Pos2::new(rect.left() + 30.0, rect.top() + 24.0),
            Pos2::new(rect.right() - 34.0, rect.bottom() - 20.0),
        );
        let duration = self.duration();

        if let Some(texture) = self.texture.as_ref() {
            painter.image(
                texture.id(),
                plot,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Level grid for the envelopes, labelled on the left.
        for db in [0.0_f32, -20.0, -40.0, -60.0] {
            let y = level_y(db, plot);
            painter.line_segment(
                [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
                Stroke::new(1.0_f32, palette::STROKE),
            );
            painter.text(
                Pos2::new(plot.left() - 5.0, y),
                Align2::RIGHT_CENTER,
                format!("{db:.0}"),
                FontId::proportional(9.0),
                palette::TEXT_MUTED,
            );
        }

        // Frequency axis for the backdrop: short ticks into the right gutter
        // only, so it never competes with the level grid.
        for (hz, label) in [(100.0, "100"), (1_000.0, "1k"), (10_000.0, "10k")] {
            let y = frequency_y(hz, plot);
            painter.line_segment(
                [Pos2::new(plot.right(), y), Pos2::new(plot.right() + 4.0, y)],
                Stroke::new(1.0_f32, palette::TEXT_GHOST),
            );
            painter.text(
                Pos2::new(plot.right() + 7.0, y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(9.0),
                palette::TEXT_MUTED,
            );
        }
        time_ruler(painter, plot, duration);

        let Some(render) = self.render.as_ref() else {
            painter.text(
                plot.center(),
                Align2::CENTER_CENTER,
                "rendering…",
                FontId::proportional(11.0),
                palette::TEXT_MUTED,
            );
            return;
        };

        for (envelope, color) in [
            (&render.left_envelope_db, LEFT_COLOR),
            (&render.right_envelope_db, RIGHT_COLOR),
        ] {
            // Stop where the tail meets the floor instead of running a flat
            // line along the plot's bottom edge.
            let audible = envelope_end(envelope);
            if audible < 2 {
                continue;
            }
            let points: Vec<Pos2> = envelope[..audible]
                .iter()
                .enumerate()
                .map(|(index, &db)| {
                    #[allow(clippy::cast_precision_loss)]
                    let fraction = index as f32 / (envelope.len() - 1) as f32;
                    Pos2::new(plot.left() + fraction * plot.width(), level_y(db, plot))
                })
                .collect();
            glow::glow_line_with(painter, &points, color, 2, 3.0, 0.22);
            painter.add(egui::Shape::line(points, Stroke::new(1.5_f32, color)));
        }

        let readout = match render.rt60_seconds {
            Some(rt60) => format!("RT60  {rt60:.2} s"),
            // A tail that has not fallen 35 dB by the end of the window has no
            // reliable slope to fit, so say so rather than invent a number.
            None => format!("RT60  > {duration:.0} s"),
        };
        legend(painter, plot, &readout);
    }

    /// The control band: Dry/Wet on the left, the tabbed knob row in the
    /// centre, Freeze on the right, each in an exact rect.
    fn draw_controls(&mut self, ui: &mut egui::Ui, setter: &ParamSetter<'_>, band: Rect) {
        surface::panel_lit(ui.painter(), band);
        let knob = knob_size();
        let inner = band.shrink(BAND_PAD);
        // Knob rows sit under the tab strip; the side columns align to them.
        let row_top = inner.top() + TABS_HEIGHT + TABS_GAP;
        let params = Arc::clone(&self.params);

        // Left column: the mix pair, captioned at tab-strip height.
        let mix_width = 2.0 * knob.x + GAP;
        let mix = Rect::from_min_size(
            Pos2::new(inner.left() + 4.0, row_top),
            Vec2::new(mix_width, knob.y),
        );
        caption(ui.painter(), Pos2::new(mix.center().x, inner.top()), "MIX");
        knob_row(ui, setter, mix, &fixed_knobs(&params), "mix");

        // Right column: Freeze, centred on the knob bodies.
        let freeze_width = self.toggle_width.max(TOGGLE_WIDTH_GUESS);
        let freeze_column = Rect::from_min_size(
            Pos2::new(inner.right() - 4.0 - freeze_width, row_top),
            Vec2::new(freeze_width, knob.y),
        );
        caption(
            ui.painter(),
            Pos2::new(freeze_column.center().x, inner.top()),
            "HOLD",
        );
        let toggle_rect = Rect::from_center_size(
            Pos2::new(
                freeze_column.center().x,
                row_top + KnobSize::Medium.diameter() / 2.0 + 4.0,
            ),
            Vec2::new(self.toggle_width, TABS_HEIGHT),
        );
        let mut freeze_ui = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(ui.scope_id().with("freeze"))
                .max_rect(toggle_rect),
        );
        let toggle = param_toggle(&mut freeze_ui, setter, &params.freeze, "FREEZE");
        self.toggle_width = toggle.rect.width();

        // Dividers between the columns.
        for x in [mix.right() + 20.0, freeze_column.left() - 20.0] {
            ui.painter().line_segment(
                [
                    Pos2::new(x, inner.top() + 4.0),
                    Pos2::new(x, inner.bottom() - 4.0),
                ],
                Stroke::new(1.0_f32, palette::STROKE),
            );
        }

        // Centre: tabs over the knob row they select.
        let centre_x = (mix.right() + freeze_column.left()) / 2.0;
        let tabs_rect = Rect::from_center_size(
            Pos2::new(centre_x, inner.top() + TABS_HEIGHT / 2.0),
            Vec2::new(tabs::tabs_width(&TAB_LABELS), TABS_HEIGHT),
        );
        let mut tabs_ui = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(ui.scope_id().with("tabs"))
                .max_rect(tabs_rect),
        );
        tabs::tabs(&mut tabs_ui, &mut self.tab, &TAB_LABELS);

        let knobs = tab_knobs(&params, self.tab);
        debug_assert!(knobs.len() <= TAB_SLOTS, "a tab overflows its row");
        #[allow(clippy::cast_precision_loss)]
        let row_width = knobs.len() as f32 * knob.x + knobs.len().saturating_sub(1) as f32 * GAP;
        let row = Rect::from_center_size(
            Pos2::new(centre_x, row_top + knob.y / 2.0),
            Vec2::new(row_width, knob.y),
        );
        // Scope the row by tab so a knob's drag state never leaks into the
        // knob that takes its slot on another tab.
        knob_row(ui, setter, row, &knobs, TAB_LABELS[self.tab.min(2)]);
    }
}

/// Small tracked caption centred above a column.
fn caption(painter: &Painter, center_top: Pos2, label: &str) {
    text::tracked(
        painter,
        Pos2::new(center_top.x, center_top.y + TABS_HEIGHT / 2.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        palette::TEXT_MUTED,
        1.2,
    );
}

/// Knobs left to right inside `rect`, separated by [`GAP`].
fn knob_row(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    rect: Rect,
    knobs: &[(&FloatParam, &str)],
    salt: &str,
) {
    let mut row = ui.new_child(
        egui::UiBuilder::new()
            .scope_id(ui.scope_id().with(("knobs", salt)))
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Min)),
    );
    row.spacing_mut().item_spacing.x = 0.0;
    for (index, (param, label)) in knobs.iter().enumerate() {
        if index > 0 {
            row.add_space(GAP);
        }
        param_knob(&mut row, setter, param, label);
    }
}

/// Legend chips (L, R) and the RT60 readout, right-aligned in the gutter
/// above the plot.
fn legend(painter: &Painter, plot: Rect, readout: &str) {
    let y = plot.top() - 12.0;
    let readout_rect = painter.text(
        Pos2::new(plot.right(), y),
        Align2::RIGHT_CENTER,
        readout,
        FontId::proportional(11.0),
        palette::TEXT_PRIMARY,
    );
    let mut x = readout_rect.left() - 16.0;
    for (label, color) in [("R", RIGHT_COLOR), ("L", LEFT_COLOR)] {
        let text_rect = painter.text(
            Pos2::new(x, y),
            Align2::RIGHT_CENTER,
            label,
            FontId::proportional(10.0),
            palette::TEXT_SECONDARY,
        );
        let swatch_right = text_rect.left() - 4.0;
        painter.line_segment(
            [
                Pos2::new(swatch_right - 12.0, y),
                Pos2::new(swatch_right, y),
            ],
            Stroke::new(2.0_f32, color),
        );
        x = swatch_right - 22.0;
    }
}

/// How many leading envelope points to draw: everything up to and including
/// the first point after the last one above the display floor.
fn envelope_end(envelope: &[f32]) -> usize {
    envelope
        .iter()
        .rposition(|&db| db > DISPLAY_FLOOR_DB)
        .map_or(0, |last| (last + 2).min(envelope.len()))
}

/// Vertical position of a level, from 0 dB at the top to the display floor.
fn level_y(db: f32, plot: Rect) -> f32 {
    let fraction = ((db.max(DISPLAY_FLOOR_DB)) / DISPLAY_FLOOR_DB).clamp(0.0, 1.0);
    plot.top() + fraction * plot.height()
}

/// Vertical position of a frequency on the spectrogram's log axis.
fn frequency_y(hz: f64, plot: Rect) -> f32 {
    use kirya_dsp::analysis::{SPECTROGRAM_MAX_HZ, SPECTROGRAM_MIN_HZ};
    let span = (SPECTROGRAM_MAX_HZ / SPECTROGRAM_MIN_HZ).log10();
    #[allow(clippy::cast_possible_truncation)]
    let fraction = ((hz / SPECTROGRAM_MIN_HZ).log10() / span).clamp(0.0, 1.0) as f32;
    // Low frequencies at the bottom.
    plot.bottom() - fraction * plot.height()
}

/// Spacings the time ruler is allowed to tick at, in seconds.
const TICK_STEPS: [f32; 8] = [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0];

/// Most intervals the ruler draws before it reaches for a coarser step.
const MAX_TICKS: f32 = 8.0;

/// Pick a readable tick spacing for a `duration`-second axis.
///
/// The window ranges over a factor of 64 between the shortest and longest
/// render, so a fixed 0.5 s tick would draw four marks on the shortest and
/// sixty-four on the longest.
fn nice_step(duration: f32) -> f32 {
    let last = TICK_STEPS[TICK_STEPS.len() - 1];
    if !duration.is_finite() || duration <= 0.0 {
        return TICK_STEPS[0];
    }
    TICK_STEPS
        .into_iter()
        .find(|&step| duration / step <= MAX_TICKS)
        .unwrap_or(last)
}

/// Time markers under the plot, spanning `duration` seconds, on a baseline.
fn time_ruler(painter: &Painter, plot: Rect, duration: f32) {
    painter.line_segment(
        [plot.left_bottom(), plot.right_bottom()],
        Stroke::new(1.0_f32, palette::STROKE_BRIGHT),
    );
    if !duration.is_finite() || duration <= 0.0 {
        return;
    }
    let step = nice_step(duration);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ticks = (duration / step) as usize;

    for index in 1..=ticks {
        #[allow(clippy::cast_precision_loss)]
        let time = index as f32 * step;
        let x = plot.left() + (time / duration) * plot.width();
        painter.line_segment(
            [
                Pos2::new(x, plot.bottom()),
                Pos2::new(x, plot.bottom() + 3.0),
            ],
            Stroke::new(1.0_f32, palette::TEXT_GHOST),
        );
        // Sub-second steps need a decimal; whole-second ones read better
        // without one.
        let label = if step < 1.0 {
            format!("{time:.1}s")
        } else {
            format!("{time:.0}s")
        };
        painter.text(
            Pos2::new(x, plot.bottom() + 4.0),
            Align2::CENTER_TOP,
            label,
            FontId::proportional(9.0),
            palette::TEXT_MUTED,
        );
    }
}

/// Colour stops of the backdrop ramp, silence to peak, as RGB floats.
///
/// Built only from palette shades: the floor is the well's own Inset colour,
/// rising through the hairline Stroke and a muted blue to a greyed cyan at
/// about a third of full luma — no amber or white, so the envelopes drawn
/// over it stay the brightest thing in the graph.
type Ramp = [(f32, [f32; 3]); 4];

fn backdrop_ramp() -> Ramp {
    let rgb = |c: Color32| [f32::from(c.r()), f32::from(c.g()), f32::from(c.b())];
    [
        (0.00, rgb(palette::INSET)),
        (0.45, rgb(palette::STROKE)),
        (0.75, rgb(palette::BLUE.lerp_to_gamma(palette::STROKE, 0.7))),
        (
            1.00,
            rgb(palette::CYAN
                .lerp_to_gamma(palette::TEXT_MUTED, 0.55)
                .lerp_to_gamma(palette::INSET, 0.5)),
        ),
    ]
}

/// Turn a render's spectrogram into an RGB image, lowest frequency at the
/// bottom (image rows run top-down, the spectrogram's run bottom-up).
fn spectrogram_image(render: &IrRender) -> ColorImage {
    let spectrogram = &render.spectrogram;
    if spectrogram.rows == 0 || spectrogram.columns == 0 {
        let floor = palette::INSET;
        return ColorImage::from_rgb([1, 1], &[floor.r(), floor.g(), floor.b()]);
    }

    let ramp = backdrop_ramp();
    let mut pixels = Vec::with_capacity(spectrogram.rows * spectrogram.columns * 3);
    for row in (0..spectrogram.rows).rev() {
        for column in 0..spectrogram.columns {
            let [r, g, b] = heat(&ramp, spectrogram.get(row, column));
            pixels.extend_from_slice(&[r, g, b]);
        }
    }
    ColorImage::from_rgb([spectrogram.columns, spectrogram.rows], &pixels)
}

/// Map a level in dB onto the backdrop ramp.
fn heat(ramp: &Ramp, db: f32) -> [u8; 3] {
    let floor = FLOOR_DB.max(DISPLAY_FLOOR_DB);
    let t = ((db - floor) / -floor).clamp(0.0, 1.0);

    let mut color = ramp[ramp.len() - 1].1;
    for pair in ramp.windows(2) {
        let (low_t, low) = pair[0];
        let (high_t, high) = pair[1];
        if t <= high_t {
            let span = (high_t - low_t).max(f32::EPSILON);
            let blend = (t - low_t) / span;
            color = [
                low[0] + blend * (high[0] - low[0]),
                low[1] + blend * (high[1] - low[1]),
                low[2] + blend * (high[2] - low[2]),
            ];
            break;
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        [
            color[0].clamp(0.0, 255.0).round() as u8,
            color[1].clamp(0.0, 255.0).round() as u8,
            color[2].clamp(0.0, 255.0).round() as u8,
        ]
    }
}

pub(crate) fn create(
    params: Arc<KiryaParams>,
    executor: AsyncExecutor<Kirya>,
) -> Option<EguiEditor<KiryaEditor>> {
    debug_assert!(vertical_budget() <= EDITOR_SIZE.1, "layout overflows");
    let state = EguiEditorState::from_size(
        WindowSize::from_logical(
            nice_plug::editor::dpi::LogicalSize::new(
                f64::from(EDITOR_SIZE.0),
                f64::from(EDITOR_SIZE.1),
            ),
            1.0,
        ),
        1.0,
    );
    let repaint = RepaintNotifier::new();
    let settings = EguiNiceSettings::new().with_tile("ppc_kirya");
    create_egui_editor(
        state,
        repaint.clone(),
        settings,
        KiryaEditor::new(params, executor, repaint),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use audio_harness::automate;
    use kirya_dsp::analysis::Analyzer;

    const RATE: f32 = 48_000.0;
    /// Brightest the spectrogram backdrop may get, as a share of full luma: it
    /// is texture behind the envelopes, not the subject.
    const BACKDROP_LUMA_CAP: f32 = 0.4;

    /// Perceived brightness on a 0..255 scale (Rec. 709 weights).
    fn luma(c: [u8; 3]) -> f32 {
        0.2126 * f32::from(c[0]) + 0.7152 * f32::from(c[1]) + 0.0722 * f32::from(c[2])
    }

    #[test]
    fn the_backdrop_ramp_rises_steadily_and_stays_dim() {
        let ramp = backdrop_ramp();
        let inset = palette::INSET;
        assert_eq!(
            heat(&ramp, DISPLAY_FLOOR_DB),
            [inset.r(), inset.g(), inset.b()],
            "silence must disappear into the well"
        );

        // Brightness has to rise monotonically or the display would read as
        // louder in places that are quieter...
        let mut previous = 0.0;
        for step in (0..=70).rev() {
            #[allow(clippy::cast_precision_loss)]
            let value = luma(heat(&ramp, -(step as f32)));
            assert!(value + 1.0e-3 >= previous, "dimmer at -{step} dB");
            previous = value;
        }
        // ...and even the peak stays a backdrop, under the cap.
        let peak = luma(heat(&ramp, 0.0));
        assert!(
            peak <= BACKDROP_LUMA_CAP * 255.0,
            "peak luma {peak} is past the cap"
        );
        assert!(peak > luma(heat(&ramp, -35.0)), "the ramp is flat");

        // Anything below the display floor pins to the floor colour.
        assert_eq!(heat(&ramp, -200.0), heat(&ramp, DISPLAY_FLOOR_DB));
    }

    #[test]
    fn the_envelope_stops_one_point_after_it_reaches_the_floor() {
        let floor = DISPLAY_FLOOR_DB;
        assert_eq!(envelope_end(&[-10.0, -40.0, floor, floor, -200.0]), 3);
        // Never past the end, and nothing audible draws nothing.
        assert_eq!(envelope_end(&[-10.0, -20.0]), 2);
        assert_eq!(envelope_end(&[floor, -90.0]), 0);
        assert_eq!(envelope_end(&[]), 0);
    }

    #[test]
    fn the_layout_fills_the_window_without_overflowing() {
        let slack = EDITOR_SIZE.1 - vertical_budget();
        assert!(slack >= 0.0, "layout overflows the window by {}", -slack);
        assert!(slack <= 8.0, "{slack} px of dead space at the bottom");

        // The widest row (four knobs) and both side columns fit across.
        let knob = knob_size();
        let used = 2.0 * MARGIN
            + 2.0 * BAND_PAD
            + (2.0 * knob.x + GAP)
            + 4.0 * knob.x
            + 3.0 * GAP
            + TOGGLE_WIDTH_GUESS
            + 4.0 * 20.0;
        assert!(used < EDITOR_SIZE.0, "controls need {used} px");
    }

    #[test]
    fn every_float_param_has_exactly_one_knob() {
        let params = KiryaParams::default();
        let mut shown: Vec<*const FloatParam> = fixed_knobs(&params)
            .iter()
            .map(|(param, _)| std::ptr::from_ref(*param))
            .collect();
        for tab in 0..TAB_LABELS.len() {
            let knobs = tab_knobs(&params, tab);
            assert!(knobs.len() <= TAB_SLOTS, "tab {tab} overflows its row");
            shown.extend(knobs.iter().map(|(param, _)| std::ptr::from_ref(*param)));
        }

        let all: Vec<*const FloatParam> = params
            .floats()
            .iter()
            .map(|param| std::ptr::from_ref(*param))
            .collect();
        assert_eq!(shown.len(), all.len(), "a knob is missing or doubled");
        for param in all {
            assert_eq!(shown.iter().filter(|&&p| p == param).count(), 1);
        }
    }

    #[test]
    fn levels_and_frequencies_map_onto_the_plot_the_right_way_up() {
        let plot = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));

        // 0 dB at the top, the floor at the bottom.
        assert!((level_y(0.0, plot) - 0.0).abs() < 1.0e-3);
        assert!((level_y(DISPLAY_FLOOR_DB, plot) - 100.0).abs() < 1.0e-3);
        assert!(level_y(-20.0, plot) < level_y(-40.0, plot));
        assert!((level_y(-500.0, plot) - 100.0).abs() < 1.0e-3);

        // 20 Hz at the bottom, 20 kHz at the top.
        assert!((frequency_y(20.0, plot) - 100.0).abs() < 1.0e-3);
        assert!((frequency_y(20_000.0, plot) - 0.0).abs() < 1.0e-3);
        assert!(frequency_y(10_000.0, plot) < frequency_y(100.0, plot));
        // A decade is a third of the three-decade axis.
        let decade = frequency_y(100.0, plot) - frequency_y(1_000.0, plot);
        assert!((decade - 100.0 / 3.0).abs() < 0.1, "{decade}");
    }

    #[test]
    fn a_render_becomes_an_image_with_one_pixel_per_cell() {
        let params = KiryaParams::default();
        RenderIr::new(&params, 48_000.0).run(&params);
        let render = params.ir_slot.lock().unwrap().take().unwrap();

        let image = spectrogram_image(&render);
        assert_eq!(
            image.size,
            [render.spectrogram.columns, render.spectrogram.rows]
        );

        // Row 0 of the image is the *top* of the display, so it must hold the
        // spectrogram's highest-frequency row.
        let top = image.pixels[0];
        let expected = heat(
            &backdrop_ramp(),
            render.spectrogram.get(render.spectrogram.rows - 1, 0),
        );
        assert_eq!([top.r(), top.g(), top.b()], expected);
    }

    #[test]
    fn an_empty_render_still_produces_a_valid_image() {
        let mut analyzer = Analyzer::new();
        // Far shorter than one transform, so the spectrogram comes back empty.
        let render = Box::new(analyzer.analyse(&[0.0; 16], &[0.0; 16], 48_000.0));
        let image = spectrogram_image(&render);
        assert_eq!(image.size, [1, 1]);
    }

    #[test]
    fn the_render_key_ignores_parameters_the_probe_does_not_use() {
        let params = KiryaParams::default();
        let before = RenderKey::read(&params, 48_000.0);

        // Wet-only with Freeze forced off: none of these change the picture.
        automate(&params.dry, 0.0, RATE);
        automate(&params.wet, 0.25, RATE);
        automate(&params.freeze, true, RATE);
        assert_eq!(before, RenderKey::read(&params, 48_000.0));

        // Anything that shapes the tail must invalidate it.
        automate(&params.decay, 0.9, RATE);
        assert_ne!(before, RenderKey::read(&params, 48_000.0));

        // ...as must the host changing rate underneath us.
        let moved = RenderKey::read(&params, 48_000.0);
        assert_ne!(moved, RenderKey::read(&params, 96_000.0));
    }

    #[test]
    fn the_time_ruler_stays_readable_across_every_window_length() {
        // The window spans a factor of 64, so a fixed tick spacing would draw
        // four marks on the shortest render and sixty-four on the longest.
        for &window in &IR_WINDOW_STOPS {
            #[allow(clippy::cast_possible_truncation)]
            let duration = window as f32;
            let step = nice_step(duration);
            let ticks = duration / step;
            assert!(
                (2.0..=MAX_TICKS).contains(&ticks),
                "{duration} s window drew {ticks} ticks at a {step} s step"
            );
            assert!(TICK_STEPS.contains(&step), "{step} is not a listed step");
        }

        // Degenerate durations must not divide by zero or hang the ruler.
        for duration in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(nice_step(duration) > 0.0, "{duration}");
        }
    }

    #[test]
    fn the_axis_falls_back_to_the_shortest_window_before_the_first_render() {
        let editor = KiryaEditor::new(
            Arc::new(KiryaParams::default()),
            AsyncExecutor::new(Arc::new(|_| ()), Arc::new(|_| ())),
            RepaintNotifier::new(),
        );
        #[allow(clippy::cast_possible_truncation)]
        let shortest = IR_WINDOW_STOPS[0] as f32;
        assert_eq!(editor.duration(), shortest);
    }

    #[test]
    fn the_sample_rate_falls_back_until_the_host_activates_the_plugin() {
        let params = KiryaParams::default();
        assert_eq!(KiryaEditor::sample_rate(&params), 48_000.0);

        params
            .sample_rate_bits
            .store(96_000.0_f64.to_bits(), Ordering::Relaxed);
        assert_eq!(KiryaEditor::sample_rate(&params), 96_000.0);

        // A garbage publish must not divide the display by zero.
        params
            .sample_rate_bits
            .store(f64::NAN.to_bits(), Ordering::Relaxed);
        assert_eq!(KiryaEditor::sample_rate(&params), 48_000.0);
    }
}
