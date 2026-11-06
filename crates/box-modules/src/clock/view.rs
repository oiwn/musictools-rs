//! Clock tile: BPM and DIV knobs, a header lamp that blinks with `clock`.

use egui::{Rect, Ui};
use neon_ui::palette;

use super::{BPM, CLOCK_METER, DIV};
use crate::tile::{Tile, header_lamp_spot, knob_row, knob_view, lamp};

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    knob_row(body, 2)
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    knob_view(ui, tile, &[BPM, DIV], palette::HIGHLIGHT);
    if let Some(center) = header_lamp_spot(tile) {
        let on = tile.shared.meters.get(CLOCK_METER) >= 0.5;
        lamp(ui, center, 3.0, on, palette::HIGHLIGHT);
    }
}
