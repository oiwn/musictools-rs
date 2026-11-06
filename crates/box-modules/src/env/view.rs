//! Env tile: four ADSR knobs; the envelope curve and a level dot sit in
//! the free header strip.

use egui::{Pos2, Rect, Ui, pos2};
use neon_ui::{glow, palette};

use super::{ATTACK, DECAY, LEVEL_METER, RELEASE, SUSTAIN};
use crate::tile::{Tile, knob_row, knob_view};

pub(super) fn layout(body: Rect) -> Vec<Rect> {
    knob_row(body, 4)
}

/// The ADSR outline across `rect`; segment widths follow the square root
/// of each time so short and long settings both stay readable.
fn curve(rect: Rect, attack: f32, decay: f32, sustain: f32, release: f32) -> [Pos2; 5] {
    let hold = 0.35 * (attack.sqrt() + decay.sqrt() + release.sqrt()).max(1.0);
    let widths = [attack.sqrt(), decay.sqrt(), hold, release.sqrt()];
    let total: f32 = widths.iter().sum::<f32>().max(f32::EPSILON);
    let x = |sum: f32| rect.left() + rect.width() * sum / total;
    let y = |level: f32| rect.bottom() - rect.height() * level;
    let a = widths[0];
    let d = a + widths[1];
    let s = d + widths[2];
    [
        pos2(x(0.0), y(0.0)),
        pos2(x(a), y(1.0)),
        pos2(x(d), y(sustain)),
        pos2(x(s), y(sustain)),
        pos2(x(total), y(0.0)),
    ]
}

pub(super) fn view(ui: &mut Ui, tile: &Tile<'_>) {
    knob_view(
        ui,
        tile,
        &[ATTACK, DECAY, SUSTAIN, RELEASE],
        palette::ACCENT_ALT,
    );
    let area = tile.header_extra.shrink2(egui::vec2(4.0, 4.0));
    if area.width() < 16.0 || area.height() < 4.0 {
        return;
    }
    let params = &tile.shared.params;
    let points = curve(
        area,
        params.get(ATTACK),
        params.get(DECAY),
        params.get(SUSTAIN),
        params.get(RELEASE),
    );
    let painter = ui.painter_at(tile.header_extra);
    glow::glow_line(&painter, &points, palette::ACCENT_ALT, 1);
    painter.add(egui::Shape::line(
        points.to_vec(),
        egui::Stroke::new(1.5, palette::ACCENT_ALT),
    ));
    let level = tile.shared.meters.get(LEVEL_METER).clamp(0.0, 1.0);
    if level > 0.0 {
        let y = area.bottom() - area.height() * level;
        painter.circle_filled(pos2(area.left() - 2.0, y), 2.0, palette::TEXT_PRIMARY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_spans_its_rect() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 10.0));
        let points = curve(rect, 5.0, 200.0, 0.5, 300.0);
        assert_eq!(points[0], pos2(0.0, 10.0));
        assert_eq!(points[4].x, 100.0);
        assert_eq!(points[1].y, 0.0);
        assert_eq!(points[2].y, 5.0);
    }
}
