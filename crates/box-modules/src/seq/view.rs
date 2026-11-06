//! Seq tile: a row of eight pitch knobs, each above its gate button; the
//! current step's button lights.

use egui::{Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use neon_ui::{glow, palette};

use super::{STEP_METER, STEPS, gate, pitch};
use crate::tile::{KNOB, Tile, knob_row, param_knob};

/// Gap between the knob row and the gate buttons.
const GAP: f32 = 10.0;
/// Gate button height.
const BUTTON: f32 = 22.0;

/// Knob rects (first `STEPS`), then gate button rects (next `STEPS`).
pub(super) fn layout(body: Rect) -> Vec<Rect> {
    let knob = KNOB.footprint();
    let stack = knob.y + GAP + BUTTON;
    let top = body.center().y - stack / 2.0;
    let row = Rect::from_min_size(pos2(body.left(), top), vec2(body.width(), knob.y));
    let knobs = knob_row(row, STEPS);
    let buttons: Vec<Rect> = knobs
        .iter()
        .map(|knob| {
            Rect::from_min_size(
                pos2(knob.left() + 4.0, knob.bottom() + GAP),
                vec2(knob.width() - 8.0, BUTTON),
            )
        })
        .collect();
    knobs.into_iter().chain(buttons).collect()
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    let rects = layout(tile.body);
    let current = tile.shared.meters.get(STEP_METER);
    for step in 0..STEPS {
        param_knob(ui, rects[step], tile.shared, pitch(step), palette::ACCENT);
        let button = rects[STEPS + step];
        let response = ui.interact(button, ui.scope_id().with(("gate", step)), Sense::click());
        let params = &tile.shared.params;
        let on = params.get(gate(step)) >= 0.5;
        if response.clicked() {
            params.set(gate(step), if on { 0.0 } else { 1.0 });
        }
        let painter = ui.painter();
        let fill = if on { palette::RAISED } else { palette::INSET };
        painter.rect_filled(button, palette::RADIUS_SMALL, fill);
        let playing = current >= 0.0 && current.round() as usize == step;
        if playing {
            glow::glow_rect(
                painter,
                button,
                palette::RADIUS_SMALL,
                palette::HIGHLIGHT,
                2,
            );
        }
        let edge = if playing {
            palette::HIGHLIGHT
        } else if response.hovered() {
            palette::STROKE_BRIGHT
        } else {
            palette::STROKE
        };
        painter.rect_stroke(
            button,
            palette::RADIUS_SMALL,
            Stroke::new(1.0, edge),
            StrokeKind::Inside,
        );
        let led = if on {
            palette::ACCENT
        } else {
            palette::TEXT_GHOST
        };
        painter.circle_filled(button.center(), 3.0, led);
    }
}
