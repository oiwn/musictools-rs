//! Bogdan's editor on nice-plug-egui, keeping the pre-neon look until the
//! 0006 rebuild.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use neon_ui::bindings::{param_knob, param_tabs_vertical};
use neon_ui::button;
use neon_ui::knob::KnobSize;
use neon_ui::tabs;
use nice_plug::context::gui::GuiContext;
use nice_plug::prelude::Enum;
use nice_plug_egui::baseview::WindowSize;
use nice_plug_egui::{
    EguiEditor, EguiEditorState, EguiNiceSettings, Frame, NiceEguiApp, RepaintNotifier,
    create_egui_editor,
};

use crate::{BogdanParams, Mode, Shape};

const VIEW_SAMPLES: usize = 1_024;
const HISTORY_SAMPLES: usize = 8_192;
// Consumes the window's vertical slack: compact header + scope + gaps + the
// knob column (86px) add up just under the 376px window height.
const SCOPE_HEIGHT: f32 = 226.0;
const REPAINT_INTERVAL: Duration = Duration::from_millis(16);

const DRIVEN_COLOR: Color32 = Color32::from_rgb(241, 174, 74);
const PROCESSED_COLOR: Color32 = Color32::from_rgb(71, 207, 218);
const GRID_COLOR: Color32 = Color32::from_rgb(66, 67, 78);

// The truce dark theme the editor shipped with, kept until the 0006 rebuild
// paints over it with the neon kit.
const THEME_BACKGROUND: Color32 = Color32::from_rgb(31, 31, 36);
const THEME_HEADER_BG: Color32 = Color32::from_rgb(20, 20, 26);
const THEME_SURFACE: Color32 = Color32::from_rgb(46, 46, 56);
const THEME_PRIMARY: Color32 = Color32::from_rgb(77, 153, 242);
const THEME_TEXT: Color32 = Color32::from_rgb(230, 230, 235);
const THEME_TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 153);
const THEME_KNOB_TRACK: Color32 = Color32::from_rgb(64, 64, 77);
const THEME_METER_CLIP: Color32 = Color32::from_rgb(224, 69, 69);

fn legacy_dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = THEME_BACKGROUND;
    visuals.window_fill = THEME_SURFACE;
    visuals.extreme_bg_color = THEME_HEADER_BG;
    visuals.faint_bg_color = THEME_SURFACE;
    visuals.widgets.noninteractive.bg_fill = THEME_SURFACE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, THEME_TEXT_DIM);
    visuals.widgets.inactive.bg_fill = THEME_KNOB_TRACK;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, THEME_TEXT);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(77, 77, 92);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, THEME_TEXT);
    visuals.widgets.active.bg_fill = THEME_PRIMARY;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.selection.bg_fill = THEME_PRIMARY.linear_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.0_f32, THEME_PRIMARY);
    visuals.override_text_color = Some(THEME_TEXT);
    visuals
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
    /// "?" toggles between the interface and a text help screen.
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

    fn draw_scope(&self, ui: &mut egui::Ui, ceiling: f32) {
        let width = (ui.available_width() - 24.0).max(1.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, SCOPE_HEIGHT), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, THEME_HEADER_BG);
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0_f32, GRID_COLOR),
            egui::StrokeKind::Inside,
        );

        let plot = rect.shrink2(Vec2::new(10.0, 12.0));
        painter.line_segment(
            [
                Pos2::new(plot.left(), plot.center().y),
                Pos2::new(plot.right(), plot.center().y),
            ],
            Stroke::new(1.0_f32, GRID_COLOR),
        );

        let safe_ceiling = finite_sample(ceiling).abs().max(f32::MIN_POSITIVE);
        let vertical_extent = safe_ceiling * 2.0;
        for polarity in [-1.0_f32, 1.0] {
            let y = sample_y(polarity * safe_ceiling, vertical_extent, plot);
            painter.line_segment(
                [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
                Stroke::new(1.0_f32, THEME_METER_CLIP),
            );
        }

        let range = self.history.display_range();
        if !range.is_empty() {
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
                    Stroke::new(1.2_f32, DRIVEN_COLOR),
                ));
                painter.add(egui::Shape::line(
                    processed,
                    Stroke::new(1.6_f32, PROCESSED_COLOR),
                ));
            }
        }

        draw_legend(&painter, plot);
    }
}

impl NiceEguiApp for BogdanEditor {
    fn build(
        &mut self,
        egui_ctx: egui::Context,
        nice_gui_ctx: GuiContext,
        _frame: &mut Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        egui_ctx.set_visuals(legacy_dark_visuals());
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

        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, THEME_BACKGROUND);
        ui.vertical_centered(|ui| {
            self.draw_header(ui);
            ui.add_space(8.0);

            if self.show_help {
                self.draw_help(ui);
                return;
            }

            let ceiling = db_to_linear(self.params.ceiling.value());
            self.draw_scope(ui, ceiling);

            ui.add_space(14.0);
            // Control cluster: mode and shape switch stacks flanking the knob
            // row. The stacks fill the knob column height so the segments use
            // the vertical space and align with the knobs top and bottom.
            // Zero the auto item-spacing so the gaps are exactly GAP, then a
            // single leading pad centers the row.
            const GAP: f32 = 12.0;
            let knob = KnobSize::Medium.footprint();
            let cluster_width = tabs::tabs_vertical_width(Mode::variants())
                + tabs::tabs_vertical_width(Shape::variants())
                + 4.0 * knob.x
                + 5.0 * GAP;
            let setter = gui.param_setter();
            // Shape only feeds the Fold voice — gray the stack out in the
            // other modes instead of offering a no-op choice.
            let shape_enabled = self.params.mode.value() == Mode::Fold;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let lead = ((ui.available_width() - cluster_width) / 2.0).max(0.0);
                ui.add_space(lead);
                param_tabs_vertical(ui, &setter, &self.params.mode, knob.y, "mode");
                ui.add_space(GAP);
                param_knob(ui, &setter, &self.params.drive, "Drive");
                ui.add_space(GAP);
                param_knob(ui, &setter, &self.params.ceiling, "Ceiling");
                ui.add_space(GAP);
                param_knob(ui, &setter, &self.params.detail, "Detail");
                ui.add_space(GAP);
                param_knob(ui, &setter, &self.params.amount, "Amount");
                ui.add_space(GAP);
                ui.add_enabled_ui(shape_enabled, |ui| {
                    param_tabs_vertical(ui, &setter, &self.params.shape, knob.y, "shape")
                });
            });
            ui.add_space(10.0);
        });
    }
}

impl BogdanEditor {
    /// Name/version line on the left, help toggle on the right — both in one
    /// 26px strip flush with the top edge, so no layout engine padding can
    /// creep in above or between them.
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        const STRIP: f32 = 26.0;
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), STRIP), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.center().y),
            Align2::LEFT_CENTER,
            concat!("PPC BOGDAN v", env!("CARGO_PKG_VERSION")),
            FontId::proportional(12.0),
            THEME_TEXT,
        );
        let button_rect = Rect::from_min_size(
            Pos2::new(rect.right() - 12.0 - STRIP, rect.top()),
            Vec2::splat(STRIP),
        );
        let mut button_ui = ui.new_child(
            egui::UiBuilder::new()
                .scope_id(ui.scope_id().with("help-button"))
                .max_rect(button_rect),
        );
        if button::icon_button(&mut button_ui, "?").clicked() {
            self.show_help = !self.show_help;
        }
    }

    /// Text help screen shown in place of the scope and controls.
    fn draw_help(&mut self, ui: &mut egui::Ui) {
        let panel_width = ui.available_width() - 24.0;
        let panel_height = ui.available_height() - 10.0;
        egui::Frame::new()
            .fill(THEME_HEADER_BG)
            .stroke(Stroke::new(1.0_f32, GRID_COLOR))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(14, 12))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("bogdan-help")
                    .max_width(panel_width - 28.0)
                    .max_height(panel_height - 26.0)
                    .show(ui, |ui| {
                        ui.set_min_width(panel_width - 28.0);
                        ui.spacing_mut().item_spacing.y = 6.0;
                        ui.label(help_text("PPC BOGDAN — detail-preserving clipper", true));
                        ui.label(help_text(
                            "A clipper keeps a fixed outer ceiling but throws away everything that \
                     crossed it. Bogdan folds that information back inward, so the waveform \
                     keeps the ceiling with smaller detail moving inside the region that \
                     would otherwise be flat. Output never exceeds the ceiling, and \
                     unclipped samples pass unchanged.",
                            false,
                        ));
                        ui.add_space(4.0);
                        ui.label(help_text("MODES", true));
                        ui.label(help_text(
                            "CLEAN — plain hard clip; the clipping delta is discarded. The \
                     reference path.",
                            false,
                        ));
                        ui.label(help_text(
                            "DETAIL — the original voice: the delta is high-passed, rectified, and \
                     subtracted inward from the clipped plateau. Suits drum-and-bass.",
                            false,
                        ));
                        ui.label(help_text(
                            "FOLD — antialiased wavefolder: excursions past the ceiling reflect \
                     back inside it. SHAPE (sine / triangle) picks the fold curve and only \
                     applies in this mode.",
                            false,
                        ));
                        ui.add_space(4.0);
                        ui.label(help_text("CONTROLS", true));
                        ui.label(help_text(
                            "DRIVE — input gain into the clipper, 0–24 dB. In FOLD it sets fold \
                     density.",
                            false,
                        ));
                        ui.label(help_text(
                            "CEILING — clip threshold, −24–0 dB. In FOLD it sets fold amplitude.",
                            false,
                        ));
                        ui.label(help_text(
                            "DETAIL — cutoff of the high-pass on the clipping delta, 20–2000 Hz. \
                     DETAIL mode only.",
                            false,
                        ));
                        ui.label(help_text(
                            "AMOUNT — effect depth, 0–100%. Inward duck depth in DETAIL (0 equals \
                     plain hard clip); clip-to-fold blend in FOLD; ignored by CLEAN.",
                            false,
                        ));
                        ui.add_space(4.0);
                        ui.label(help_text("TIPS", true));
                        ui.label(help_text(
                            "Double-click (or ctrl/⌘-click) a knob to reset it to its default.",
                            false,
                        ));
                    });
            });
    }
}

/// One styled help line: bold section headers, dim body text.
fn help_text(text: &str, heading: bool) -> RichText {
    let text = RichText::new(text).size(12.0);
    if heading {
        text.strong().color(THEME_TEXT)
    } else {
        text.color(THEME_TEXT_DIM)
    }
}

pub(crate) fn create(params: Arc<BogdanParams>) -> Option<EguiEditor<BogdanEditor>> {
    let state = EguiEditorState::from_size(
        WindowSize::from_logical(
            nice_plug::editor::dpi::LogicalSize::new(500.0_f64, 376.0_f64),
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

fn draw_legend(painter: &egui::Painter, plot: Rect) {
    let top = plot.top() + 2.0;
    let driven_left = plot.left() + 4.0;
    painter.line_segment(
        [
            Pos2::new(driven_left, top + 5.0),
            Pos2::new(driven_left + 14.0, top + 5.0),
        ],
        Stroke::new(1.5_f32, DRIVEN_COLOR),
    );
    painter.text(
        Pos2::new(driven_left + 19.0, top),
        Align2::LEFT_TOP,
        "DRIVEN",
        FontId::proportional(9.0),
        DRIVEN_COLOR,
    );

    let processed_left = driven_left + 76.0;
    painter.line_segment(
        [
            Pos2::new(processed_left, top + 5.0),
            Pos2::new(processed_left + 14.0, top + 5.0),
        ],
        Stroke::new(1.5_f32, PROCESSED_COLOR),
    );
    painter.text(
        Pos2::new(processed_left + 19.0, top),
        Align2::LEFT_TOP,
        "PROCESSED",
        FontId::proportional(9.0),
        PROCESSED_COLOR,
    );

    painter.text(
        Pos2::new(plot.right() - 4.0, top),
        Align2::RIGHT_TOP,
        "CEILING",
        FontId::proportional(9.0),
        THEME_METER_CLIP,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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
