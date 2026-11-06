//! Output tile: a LEVEL knob and a peak dot.

use egui::{Rect, Ui, pos2, vec2};
use neon_ui::{glow, meter, palette};

use super::{LEVEL, PEAK};
use crate::tile::{KNOB, Tile, param_knob};

/// Gap between the left edge (where the input nubs sit) and the knob, so
/// the nubs never cover its readout.
const NUB_CLEARANCE: f32 = 10.0;

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    let size = KNOB.footprint();
    let knob = Rect::from_min_size(
        pos2(body.left() + NUB_CLEARANCE, body.center().y - size.y / 2.0),
        size,
    );
    let lamp = Rect::from_center_size(
        pos2(
            (knob.right() + body.right()) / 2.0,
            knob.top() + KNOB.diameter() / 2.0,
        ),
        vec2(16.0, 16.0),
    );
    vec![knob, lamp]
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    let rects = layout(tile.body);
    param_knob(ui, rects[0], tile.shared, LEVEL, palette::WARNING);
    let peak = tile.shared.meters.get(PEAK);
    let db = if peak > 0.0 {
        f64::from(20.0 * peak.log10())
    } else {
        -120.0
    };
    let center = rects[1].center();
    let painter = ui.painter();
    if db > -60.0 {
        let color = meter::level_color(db);
        glow::glow_circle(painter, center, 5.0, color, 2);
        painter.circle_filled(center, 5.0, color);
    } else {
        painter.circle_filled(center, 5.0, palette::INSET);
        painter.circle_stroke(center, 5.0, egui::Stroke::new(1.0, palette::STROKE_BRIGHT));
    }
}
