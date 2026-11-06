//! Sine tile: FREQ and LEVEL knobs.

use egui::{Rect, Ui};
use neon_ui::palette;

use super::{FREQ, LEVEL};
use crate::tile::{Tile, knob_row, knob_view};

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    knob_row(body, 2)
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    knob_view(ui, tile, &[FREQ, LEVEL], palette::ACCENT);
}
