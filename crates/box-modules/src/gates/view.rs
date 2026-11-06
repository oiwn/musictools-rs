//! Gates tile: RATE and WIDTH knobs, a header lamp that follows the gate.

use egui::{Rect, Ui};
use neon_ui::palette;

use super::{GATE_METER, RATE, WIDTH};
use crate::tile::{Tile, header_lamp_spot, knob_row, knob_view, lamp};

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    knob_row(body, 2)
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    knob_view(ui, tile, &[RATE, WIDTH], palette::HIGHLIGHT);
    if let Some(center) = header_lamp_spot(tile) {
        let on = tile.shared.meters.get(GATE_METER) >= 0.5;
        lamp(ui, center, 3.0, on, palette::HIGHLIGHT);
    }
}
