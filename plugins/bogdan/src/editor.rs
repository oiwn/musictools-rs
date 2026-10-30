//! Dark Neon editor, laid out like Kirya's: a header with help, one scope
//! graph in an inset well, and a control band of MODE tabs, the four knobs,
//! and SHAPE tabs.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use neon_ui::bindings::{param_knob, param_tabs_vertical};
use neon_ui::knob::KnobSize;
use neon_ui::{button, fonts, glow, palette, surface, tabs, text};
use nice_plug::context::gui::{GuiContext, ParamSetter};
use nice_plug::prelude::Enum;
use nice_plug_egui::baseview::WindowSize;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, Frame, NiceEguiApp, RepaintNotifier,
    create_egui_editor,
};

use crate::{BogdanParams, Mode, Shape};

const VIEW_SAMPLES: usize = 1_024;
const HISTORY_SAMPLES: usize = 8_192;
const REPAINT_INTERVAL: Duration = Duration::from_millis(16);

/// Logical window size.
const EDITOR_SIZE: (f32, f32) = (540.0, 420.0);

// Vertical budget, top to bottom (see `vertical_budget`).
const MARGIN: f32 = 12.0;
const HEADER_HEIGHT: f32 = 32.0;
const HEADER_GAP: f32 = 10.0;
const GRAPH_HEIGHT: f32 = 206.0;
const GRAPH_GAP: f32 = 12.0;
/// Caption row over the side columns.
const CAPTION_HEIGHT: f32 = 18.0;
const CAPTION_GAP: f32 = 8.0;
/// Inner padding of the control band panel.
const BAND_PAD: f32 = 10.0;

/// Gap between knobs in the row.
const GAP: f32 = 14.0;
/// Space either side of a column divider.
const COLUMN_GAP: f32 = 16.0;

const DRIVEN_COLOR: Color32 = palette::ORANGE;
const PROCESSED_COLOR: Color32 = palette::CYAN;
const CEILING_COLOR: Color32 = palette::CLIP;

/// Below this peak the scope shows a "waiting for audio" hint instead of a
/// flat line that reads as a broken display.
const SILENCE: f32 = 1.0e-4;

/// Knob width and full knob column height.
fn knob_size() -> Vec2 {
    KnobSize::Medium.footprint()
}

/// Height of the control band panel: caption row, gap, knob row, padding.
fn band_height() -> f32 {
    BAND_PAD + CAPTION_HEIGHT + CAPTION_GAP + knob_size().y + BAND_PAD
}

/// Total height the layout uses; must land just under the window height.
fn vertical_budget() -> f32 {
    MARGIN + HEADER_HEIGHT + HEADER_GAP + GRAPH_HEIGHT + GRAPH_GAP + band_height() + MARGIN
}

/// Width the control band's contents need.
fn band_content_width() -> f32 {
    let knob = knob_size();
    tabs::tabs_vertical_width(Mode::variants())
        + 2.0 * (2.0 * COLUMN_GAP)
        + 4.0 * knob.x
        + 3.0 * GAP
        + tabs::tabs_vertical_width(Shape::variants())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ScopeFrame {
    driven: f32,
    processed: f32,
}

struct ScopeHistory {
    frames: VecDeque<ScopeFrame>,
}

impl ScopeHistory {
    fn new() -> Self {
        Self {
            frames: VecDeque::with_capacity(HISTORY_SAMPLES),
        }
    }

    fn clear(&mut self) {
        self.frames.clear();
    }

    fn ingest_interleaved(&mut self, interleaved: &[f32]) {
        for pair in interleaved.as_chunks::<2>().0 {
            if self.frames.len() == HISTORY_SAMPLES {
                let _ = self.frames.pop_front();
            }
            self.frames.push_back(ScopeFrame {
                driven: finite_sample(pair[0]),
                processed: finite_sample(pair[1]),
            });
        }
    }

    fn display_range(&self) -> std::ops::Range<usize> {
        let len = self.frames.len();
        if len <= VIEW_SAMPLES {
            return 0..len;
        }

        let latest_start = len - VIEW_SAMPLES;
        let start = (1..=latest_start)
            .rev()
            .find(|&index| self.frames[index - 1].driven <= 0.0 && self.frames[index].driven > 0.0)
            .unwrap_or(latest_start);
        start..start + VIEW_SAMPLES
    }
}

pub struct BogdanEditor {
    params: Arc<BogdanParams>,
    gui: Option<GuiContext>,
    repaint: RepaintNotifier,
    history: ScopeHistory,
    /// "?" swaps the graph and controls for a text help screen.
    show_help: bool,
}

impl BogdanEditor {
    fn new(params: Arc<BogdanParams>, repaint: RepaintNotifier) -> Self {
        Self {
            params,
            gui: None,
            repaint,
            history: ScopeHistory::new(),
            show_help: false,
        }
    }
}

impl NiceEguiApp for BogdanEditor {
    fn build(
        &mut self,
        egui_ctx: egui::Context,
        nice_gui_ctx: GuiContext,
        _frame: &mut Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        egui_ctx.set_visuals(neon_ui::visuals::dark_neon_visuals());
        fonts::install(&egui_ctx);
        self.gui = Some(nice_gui_ctx);
        self.history.clear();
        self.params.scope_tap.clear();
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.gui = None;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let Some(gui) = self.gui.clone() else {
            return;
        };
        // Ping the window's repaint poll every frame (picked up at up to
        // 1/64 s): the scope animates and clicks must render without waiting
        // for OS events.
        self.repaint.request_repaint();
        ui.ctx().request_repaint_after(REPAINT_INTERVAL);
        let drained = self.params.scope_tap.drain();
        self.history.ingest_interleaved(&drained);

        let setter = gui.param_setter();
        self.draw(ui, &setter);
    }
}

impl BogdanEditor {
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
        let ceiling_db = self.params.ceiling.value();
        self.draw_scope(&ui.painter_at(graph), graph, ceiling_db);
        self.draw_controls(ui, setter, band);
    }

    /// The scope: PROCESSED glowing over a dimmer DRIVEN, between Red
    /// ceiling guides, with labels in gutters around the plot.
    fn draw_scope(&self, painter: &Painter, rect: Rect, ceiling_db: f32) {
        surface::inset(painter, rect);
        let plot = Rect::from_min_max(
            Pos2::new(rect.left() + 40.0, rect.top() + 24.0),
            Pos2::new(rect.right() - 12.0, rect.bottom() - 10.0),
        );

        let ceiling = db_to_linear(ceiling_db);
        let safe_ceiling = finite_sample(ceiling).abs().max(f32::MIN_POSITIVE);
        let vertical_extent = safe_ceiling * 2.0;

        painter.line_segment(
            [
                Pos2::new(plot.left(), plot.center().y),
                Pos2::new(plot.right(), plot.center().y),
            ],
            Stroke::new(1.0_f32, palette::STROKE),
        );
        painter.text(
            Pos2::new(plot.left() - 6.0, plot.center().y),
            Align2::RIGHT_CENTER,
            "0",
            FontId::proportional(9.0),
            palette::TEXT_MUTED,
        );
        for polarity in [-1.0_f32, 1.0] {
            let y = sample_y(polarity * safe_ceiling, vertical_extent, plot);
            painter.line_segment(
                [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
                Stroke::new(1.0_f32, CEILING_COLOR.gamma_multiply(0.7)),
            );
            // Both guides sit at ± the ceiling; label them with its level.
            painter.text(
                Pos2::new(plot.left() - 6.0, y),
                Align2::RIGHT_CENTER,
                format!("{ceiling_db:.1}"),
                FontId::proportional(9.0),
                CEILING_COLOR.gamma_multiply(0.85),
            );
        }

        let range = self.history.display_range();
        let peak = self
            .history
            .frames
            .range(range.clone())
            .map(|frame| frame.driven.abs().max(frame.processed.abs()))
            .fold(0.0_f32, f32::max);
        if peak < SILENCE {
            painter.text(
                Pos2::new(plot.center().x, plot.center().y - 14.0),
                Align2::CENTER_CENTER,
                "waiting for audio",
                FontId::proportional(11.0),
                palette::TEXT_MUTED,
            );
        } else {
            let mut driven = Vec::with_capacity(range.len());
            let mut processed = Vec::with_capacity(range.len());
            for (display_index, history_index) in range.enumerate() {
                let x = sample_x(display_index, plot);
                let frame = self.history.frames[history_index];
                driven.push(Pos2::new(x, sample_y(frame.driven, vertical_extent, plot)));
                processed.push(Pos2::new(
                    x,
                    sample_y(frame.processed, vertical_extent, plot),
                ));
            }
            if driven.len() > 1 {
                painter.add(egui::Shape::line(
                    driven,
                    Stroke::new(1.2_f32, DRIVEN_COLOR.gamma_multiply(0.75)),
                ));
                glow::glow_line_with(painter, &processed, PROCESSED_COLOR, 2, 3.0, 0.22);
                painter.add(egui::Shape::line(
                    processed,
                    Stroke::new(1.5_f32, PROCESSED_COLOR),
                ));
            }
        }

        legend(painter, plot);
    }

    /// The control band: MODE tabs left, the four knobs centre, SHAPE tabs
    /// right (only live in Fold), each in an exact rect.
    fn draw_controls(&mut self, ui: &mut egui::Ui, setter: &ParamSetter<'_>, band: Rect) {
        surface::panel_lit(ui.painter(), band);
        let knob = knob_size();
        let inner = band.shrink(BAND_PAD);
        let row_top = inner.top() + CAPTION_HEIGHT + CAPTION_GAP;
        let params = Arc::clone(&self.params);

        // Centre the whole cluster in the band.
        let lead = ((inner.width() - band_content_width()) / 2.0).max(0.0);
        let mode_width = tabs::tabs_vertical_width(Mode::variants());
        let shape_width = tabs::tabs_vertical_width(Shape::variants());
        let mode = Rect::from_min_size(
            Pos2::new(inner.left() + lead, row_top),
            Vec2::new(mode_width, knob.y),
        );
        let knobs = Rect::from_min_size(
            Pos2::new(mode.right() + 2.0 * COLUMN_GAP, row_top),
            Vec2::new(4.0 * knob.x + 3.0 * GAP, knob.y),
        );
        let shape = Rect::from_min_size(
            Pos2::new(knobs.right() + 2.0 * COLUMN_GAP, row_top),
            Vec2::new(shape_width, knob.y),
        );

        caption(ui.painter(), mode.center().x, inner.top(), "MODE");
        caption(ui.painter(), knobs.center().x, inner.top(), "CLIPPER");
        caption(ui.painter(), shape.center().x, inner.top(), "SHAPE");
        for x in [mode.right() + COLUMN_GAP, knobs.right() + COLUMN_GAP] {
            ui.painter().line_segment(
                [
                    Pos2::new(x, inner.top() + 4.0),
                    Pos2::new(x, inner.bottom() - 4.0),
                ],
                Stroke::new(1.0_f32, palette::STROKE),
            );
        }

        let mut mode_ui = child(ui, "mode", mode);
        param_tabs_vertical(&mut mode_ui, setter, &params.mode, knob.y, "mode");

        let mut row = child(ui, "knobs", knobs);
        row.spacing_mut().item_spacing.x = 0.0;
        for (index, (param, label)) in [
            (&params.drive, "Drive"),
            (&params.ceiling, "Ceiling"),
            (&params.detail, "Detail"),
            (&params.amount, "Amount"),
        ]
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                row.add_space(GAP);
            }
            param_knob(&mut row, setter, param, label);
        }

        // Shape only feeds the Fold voice — gray the stack out in the
        // other modes instead of offering a no-op choice.
        let mut shape_ui = child(ui, "shape", shape);
        if params.mode.value() != Mode::Fold {
            shape_ui.disable();
        }
        param_tabs_vertical(&mut shape_ui, setter, &params.shape, knob.y, "shape");
    }
}

/// A left-to-right child ui filling exactly `rect`.
fn child(ui: &mut egui::Ui, salt: &str, rect: Rect) -> egui::Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .scope_id(ui.scope_id().with(salt))
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Min)),
    )
}

/// Title strip: tracked name, Muted subtitle with the build stamp so the
/// loaded bundle is identifiable.
fn draw_header(painter: &Painter, rect: Rect) {
    surface::panel_lit(painter, rect);
    let title = text::tracked(
        painter,
        Pos2::new(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        "PPC BOGDAN",
        fonts::heading(13.0),
        palette::TEXT_PRIMARY,
        1.5,
    );
    painter.text(
        Pos2::new(title.right() + 10.0, rect.center().y + 1.0),
        Align2::LEFT_CENTER,
        concat!("detail clipper · v", env!("CARGO_PKG_VERSION")),
        FontId::proportional(10.0),
        palette::TEXT_MUTED,
    );
}

/// Small tracked caption centred over a column, in the caption row.
fn caption(painter: &Painter, center_x: f32, top: f32, label: &str) {
    text::tracked(
        painter,
        Pos2::new(center_x, top + CAPTION_HEIGHT / 2.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        palette::TEXT_MUTED,
        1.2,
    );
}

/// Legend chips right-aligned in the gutter above the plot.
fn legend(painter: &Painter, plot: Rect) {
    let y = plot.top() - 12.0;
    let mut x = plot.right();
    for (label, color) in [
        ("CEILING", CEILING_COLOR),
        ("PROCESSED", PROCESSED_COLOR),
        ("DRIVEN", DRIVEN_COLOR),
    ] {
        let text_rect = painter.text(
            Pos2::new(x, y),
            Align2::RIGHT_CENTER,
            label,
            FontId::proportional(9.0),
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
        x = swatch_right - 24.0;
    }
}

/// Help copy: a section heading, then its lines. Mirrors `specs/bogdan.md`.
const HELP: &[(&str, &[&str])] = &[
    (
        "PPC BOGDAN — DETAIL-PRESERVING CLIPPER",
        &[
            "A clipper keeps a fixed outer ceiling but throws away everything that crossed \
             it. Bogdan folds that information back inward, so the waveform keeps the \
             ceiling with smaller detail moving inside the region that would otherwise be \
             flat. Output never exceeds the ceiling, and unclipped samples pass unchanged.",
            "The scope shows the driven input (orange) against the output (cyan), between \
             the red ceiling guides.",
        ],
    ),
    (
        "MODE",
        &[
            "CLEAN — plain hard clip; the clipping delta is discarded. The reference path.",
            "DETAIL — the original voice: the delta is high-passed, rectified, and \
             subtracted inward from the clipped plateau. Suits drum-and-bass.",
            "FOLD — antialiased wavefolder: excursions past the ceiling reflect back inside \
             it.",
        ],
    ),
    (
        "CLIPPER",
        &[
            "DRIVE — input gain into the clipper, 0–24 dB. In FOLD it sets fold density.",
            "CEILING — clip threshold, −24–0 dB. In FOLD it sets fold amplitude.",
            "DETAIL — cutoff of the high-pass on the clipping delta, 20–2000 Hz. DETAIL \
             mode only.",
            "AMOUNT — effect depth, 0–100%. Inward duck depth in DETAIL (0 equals plain \
             hard clip); clip-to-fold blend in FOLD; ignored by CLEAN.",
        ],
    ),
    (
        "SHAPE",
        &["SINE / TRIANGLE — the fold curve. Only applies in FOLD, greyed out otherwise."],
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
        .id_salt("bogdan-help")
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

pub(crate) fn create(params: Arc<BogdanParams>) -> Option<EguiEditor<BogdanEditor>> {
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
    let settings = EguiNiceSettings::new().with_tile("ppc_bogdan");
    create_egui_editor(
        state,
        repaint.clone(),
        settings,
        BogdanEditor::new(params, repaint),
    )
}

fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

fn finite_sample(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

#[allow(clippy::cast_precision_loss)]
fn sample_x(index: usize, plot: Rect) -> f32 {
    let fraction = index as f32 / (VIEW_SAMPLES - 1) as f32;
    plot.left() + fraction * plot.width()
}

fn sample_y(sample: f32, vertical_extent: f32, plot: Rect) -> f32 {
    let normalized = (finite_sample(sample) / vertical_extent).clamp(-1.0, 1.0);
    plot.center().y - normalized * plot.height() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_fills_the_window_without_overflowing() {
        let slack = EDITOR_SIZE.1 - vertical_budget();
        assert!(slack >= 0.0, "layout overflows the window by {}", -slack);
        assert!(slack <= 8.0, "{slack} px of dead space at the bottom");

        let used = 2.0 * MARGIN + 2.0 * BAND_PAD + band_content_width();
        assert!(used <= EDITOR_SIZE.0, "controls need {used} px");
    }

    #[test]
    fn ingests_interleaved_frames_and_sanitizes_non_finite_values() {
        let mut history = ScopeHistory::new();
        history.ingest_interleaved(&[1.0, 0.5, f32::NAN, f32::INFINITY, 99.0]);

        assert_eq!(
            history.frames,
            VecDeque::from([
                ScopeFrame {
                    driven: 1.0,
                    processed: 0.5,
                },
                ScopeFrame {
                    driven: 0.0,
                    processed: 0.0,
                },
            ])
        );
    }

    #[test]
    fn circular_history_retains_only_the_newest_frames() {
        let mut history = ScopeHistory::new();
        let samples: Vec<f32> = (0..HISTORY_SAMPLES + 3)
            .flat_map(|index| [index as f32, -(index as f32)])
            .collect();
        history.ingest_interleaved(&samples);

        assert_eq!(history.frames.len(), HISTORY_SAMPLES);
        assert_eq!(history.frames.front().unwrap().driven, 3.0);
        assert_eq!(
            history.frames.back().unwrap().driven,
            (HISTORY_SAMPLES + 2) as f32
        );
    }

    #[test]
    fn display_uses_newest_trigger_with_a_complete_view() {
        let mut history = ScopeHistory::new();
        for index in 0..VIEW_SAMPLES + 20 {
            let driven = if index == 8 || index == 20 { 1.0 } else { -1.0 };
            history.frames.push_back(ScopeFrame {
                driven,
                processed: 0.0,
            });
        }

        assert_eq!(history.display_range(), 20..20 + VIEW_SAMPLES);
    }

    #[test]
    fn display_falls_back_to_newest_complete_view_without_a_trigger() {
        let mut history = ScopeHistory::new();
        history.frames.resize(
            VIEW_SAMPLES + 12,
            ScopeFrame {
                driven: 1.0,
                processed: 0.0,
            },
        );

        assert_eq!(history.display_range(), 12..12 + VIEW_SAMPLES);
    }

    #[test]
    fn display_uses_all_available_frames_while_history_is_short() {
        let mut history = ScopeHistory::new();
        history.frames.resize(32, ScopeFrame::default());

        assert_eq!(history.display_range(), 0..32);
    }

    #[test]
    fn ceiling_guides_land_halfway_between_center_and_edge() {
        let plot = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));

        assert_eq!(sample_y(0.5, 1.0, plot), 25.0);
        assert_eq!(sample_y(-0.5, 1.0, plot), 75.0);
    }
}
