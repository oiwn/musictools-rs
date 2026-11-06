//! Filter tile: CUTOFF, Q and TYPE knobs; a clickable 24 dB lamp in the
//! header toggles the slope.

use egui::{Align2, FontId, Rect, Sense, Ui, pos2, vec2};
use neon_ui::palette;

use super::{CUTOFF, Q, SLOPE, TYPE};
use crate::tile::{KNOB, Tile, lamp, param_knob};

/// Room left of the knobs for the two input nubs.
const NUB_CLEARANCE: f32 = 12.0;

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    let knob = KNOB.footprint();
    let top = body.center().y - knob.y / 2.0;
    (0..3)
        .map(|index| {
            Rect::from_min_size(
                pos2(
                    body.left() + NUB_CLEARANCE + knob.x * index as f32 + 6.0 * index as f32,
                    top,
                ),
                knob,
            )
        })
        .collect()
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    let rects = layout(tile.body);
    param_knob(ui, rects[0], tile.shared, CUTOFF, palette::ACCENT);
    param_knob(ui, rects[1], tile.shared, Q, palette::ACCENT);
    param_knob(ui, rects[2], tile.shared, TYPE, palette::ACCENT_ALT);

    let header = tile.header_extra;
    if header.width() < 40.0 {
        return;
    }
    let toggle = Rect::from_min_max(pos2(header.right() - 40.0, header.top()), header.max);
    let response = ui.interact(toggle, ui.scope_id().with("slope"), Sense::click());
    let params = &tile.shared.params;
    let steep = params.get(SLOPE) >= 0.5;
    if response.clicked() {
        params.set(SLOPE, if steep { 0.0 } else { 1.0 });
    }
    let lamp_center = pos2(toggle.left() + 5.0, toggle.center().y);
    lamp(ui, lamp_center, 3.0, steep, palette::ACCENT);
    let text = if response.hovered() {
        palette::TEXT_PRIMARY
    } else if steep {
        palette::TEXT_SECONDARY
    } else {
        palette::TEXT_MUTED
    };
    ui.painter().text(
        lamp_center + vec2(8.0, 0.0),
        Align2::LEFT_CENTER,
        "24dB",
        FontId::proportional(9.0),
        text,
    );
}
