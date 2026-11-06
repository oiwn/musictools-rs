//! Mix4 tile: a LEVEL and a PAN knob per channel, in one row.

use egui::{Rect, Ui};
use neon_ui::palette;

use super::{CHANNELS, level, pan};
use crate::tile::{Tile, knob_row, param_knob};

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    knob_row(body, 2 * CHANNELS)
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    let rects = layout(tile.body);
    for channel in 0..CHANNELS {
        param_knob(
            ui,
            rects[2 * channel],
            tile.shared,
            level(channel),
            palette::ACCENT,
        );
        param_knob(
            ui,
            rects[2 * channel + 1],
            tile.shared,
            pan(channel),
            palette::ACCENT_ALT,
        );
    }
}
