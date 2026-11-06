//! Scaler tile: ROOT and SCALE knobs, an octave strip marking the scale's
//! degrees at their cents (so microtonal steps show where they fall), and
//! the scale name in the header.

use egui::{Align2, FontId, Rect, Stroke, Ui, pos2, vec2};
use neon_ui::{glow, palette, surface};

use super::dsp::DEGREE_METER;
use super::scales::SCALES;
use super::{ROOT, SCALE};
use crate::tile::{KNOB, Tile, param_knob};

/// Room left of the knobs for the input nub.
const NUB_CLEARANCE: f32 = 8.0;
/// Octave strip height.
const STRIP: f32 = 22.0;

/// The two knobs, then the octave strip.
pub(super) fn layout(body: Rect) -> Vec<Rect> {
    let knob = KNOB.footprint();
    let top = body.center().y - knob.y / 2.0;
    let root = Rect::from_min_size(pos2(body.left() + NUB_CLEARANCE, top), knob);
    let scale = Rect::from_min_size(pos2(root.right(), top), knob);
    let strip = Rect::from_min_max(
        pos2(scale.right() + 6.0, body.center().y - STRIP / 2.0),
        pos2(body.right() - NUB_CLEARANCE, body.center().y + STRIP / 2.0),
    );
    vec![root, scale, strip]
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    let rects = layout(tile.body);
    param_knob(ui, rects[0], tile.shared, ROOT, palette::ACCENT);
    param_knob(ui, rects[1], tile.shared, SCALE, palette::ACCENT_ALT);

    let params = &tile.shared.params;
    let scale = &SCALES[(params.get(SCALE).max(0.0) as usize).min(SCALES.len() - 1)];
    let strip = rects[2];
    let painter = ui.painter();
    surface::inset(painter, strip);
    // Faint 12-tone grid, then the scale's degrees at their cents.
    for semitone in 1..12 {
        let x = strip.left() + strip.width() * semitone as f32 / 12.0;
        painter.line_segment(
            [pos2(x, strip.top() + 4.0), pos2(x, strip.bottom() - 4.0)],
            Stroke::new(1.0, palette::STROKE),
        );
    }
    let playing = tile.shared.meters.get(DEGREE_METER);
    for &degree in scale.cents {
        let x = strip.left() + strip.width() * degree / 1200.0;
        let lit = (degree - playing).abs() < 0.5;
        let color = if lit {
            palette::HIGHLIGHT
        } else {
            palette::ACCENT_ALT
        };
        let mark = Rect::from_center_size(
            pos2(x + 1.5, strip.center().y),
            vec2(3.0, strip.height() - 6.0),
        );
        if lit {
            glow::glow_rect(painter, mark, 1.0, color, 1);
        }
        painter.rect_filled(mark, 1.0, color);
    }

    let header = tile.header_extra;
    if header.width() > 20.0 {
        ui.painter_at(header).text(
            pos2(header.right(), header.center().y),
            Align2::RIGHT_CENTER,
            scale.name,
            FontId::proportional(10.0),
            palette::TEXT_SECONDARY,
        );
    }
}
