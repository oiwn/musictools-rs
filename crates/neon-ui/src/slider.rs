//! NeonSlider — inset track, accent fill up to the handle, raised handle;
//! plus a double-thumb range variant. Framework-free like the rest of the kit.

use crate::{glow, palette, surface};
use egui::{Painter, Pos2, Rect, Response, Sense, Ui, Vec2};

const TRACK_HEIGHT: f32 = 6.0;
const HANDLE_WIDTH: f32 = 10.0;
const HANDLE_HEIGHT: f32 = 16.0;
const ROW_HEIGHT: f32 = 22.0;
const FILL_INSET: f32 = 2.0;

/// Map a pointer x within the handle travel span to a normalized value.
#[must_use]
pub fn pos_to_value(x: f32, left: f32, right: f32) -> f64 {
    if right <= left {
        return 0.0;
    }
    (((x - left) / (right - left)).clamp(0.0, 1.0)) as f64
}

/// True when `x` is at least as close to `low_x` as to `high_x`.
#[must_use]
pub fn nearest_low(x: f32, low_x: f32, high_x: f32) -> bool {
    (x - low_x).abs() <= (x - high_x).abs()
}

/// Single-value slider. `value` is normalized 0..=1; double-click restores
/// `default`.
pub fn slider(ui: &mut Ui, value: &mut f64, default: f64, width: f32) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click_and_drag());
    if response.double_clicked() {
        *value = default;
    }
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let (left, right) = track_span(rect);
        *value = pos_to_value(pos.x, left, right);
    }
    paint_single(ui.painter(), rect, *value, response.hovered());
    response
}

/// Double-thumb range slider; thumbs cannot cross.
pub fn range_slider(ui: &mut Ui, low: &mut f64, high: &mut f64, width: f32) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click_and_drag());
    let (left, right) = track_span(rect);

    if (response.drag_started() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let low_x = value_x(*low, rect);
        let high_x = value_x(*high, rect);
        let moving_low = nearest_low(pos.x, low_x, high_x);
        ui.ctx()
            .data_mut(|data| data.insert_temp(response.id, moving_low));
    }
    if response.dragged()
        && let Some(pos) = response.interact_pointer_pos()
    {
        let moving_low = ui
            .ctx()
            .data(|data| data.get_temp::<bool>(response.id))
            .unwrap_or(true);
        let value = pos_to_value(pos.x, left, right);
        if moving_low {
            *low = value.min(*high);
        } else {
            *high = value.max(*low);
        }
    }

    paint_range(ui.painter(), rect, *low, *high, response.hovered());
    response
}

/// Handle travel span: the track minus half a handle on each side.
fn track_span(rect: Rect) -> (f32, f32) {
    (
        rect.left() + HANDLE_WIDTH / 2.0,
        rect.right() - HANDLE_WIDTH / 2.0,
    )
}

/// Pointer x of a normalized value inside `rect`.
fn value_x(value: f64, rect: Rect) -> f32 {
    let (left, right) = track_span(rect);
    left + value as f32 * (right - left)
}

fn track_rect(rect: Rect) -> Rect {
    let mid_y = rect.center().y;
    Rect::from_min_max(
        Pos2::new(rect.left(), mid_y - TRACK_HEIGHT / 2.0),
        Pos2::new(rect.right(), mid_y + TRACK_HEIGHT / 2.0),
    )
}

fn paint_track_and_fill(painter: &Painter, rect: Rect, fill_to: f32) {
    let track = track_rect(rect);
    surface::inset(painter, track);
    let mid_y = rect.center().y;
    let fill = Rect::from_min_max(
        Pos2::new(
            track.left() + FILL_INSET,
            mid_y - (TRACK_HEIGHT - FILL_INSET) / 2.0,
        ),
        Pos2::new(fill_to, mid_y + (TRACK_HEIGHT - FILL_INSET) / 2.0),
    );
    painter.rect_filled(fill, palette::RADIUS_SMALL, palette::ACCENT);
}

fn paint_handle(painter: &Painter, x: f32, mid_y: f32, hovered: bool) {
    let rect = Rect::from_center_size(Pos2::new(x, mid_y), Vec2::new(HANDLE_WIDTH, HANDLE_HEIGHT));
    if hovered {
        glow::glow_rect(painter, rect, palette::RADIUS_SMALL, palette::ACCENT, 2);
    }
    surface::raised(painter, rect);
}

fn paint_single(painter: &Painter, rect: Rect, value: f64, hovered: bool) {
    paint_track_and_fill(painter, rect, value_x(value, rect));
    paint_handle(painter, value_x(value, rect), rect.center().y, hovered);
}

fn paint_range(painter: &Painter, rect: Rect, low: f64, high: f64, hovered: bool) {
    paint_track_and_fill(painter, rect, value_x(high, rect));
    paint_handle(painter, value_x(low, rect), rect.center().y, hovered);
    paint_handle(painter, value_x(high, rect), rect.center().y, hovered);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_to_value_maps_the_span_linearly_and_clamps() {
        assert!((pos_to_value(0.0, 0.0, 100.0) - 0.0).abs() < 1e-9);
        assert!((pos_to_value(50.0, 0.0, 100.0) - 0.5).abs() < 1e-9);
        assert!((pos_to_value(100.0, 0.0, 100.0) - 1.0).abs() < 1e-9);
        assert_eq!(pos_to_value(-10.0, 0.0, 100.0), 0.0);
        assert_eq!(pos_to_value(110.0, 0.0, 100.0), 1.0);
        assert_eq!(pos_to_value(50.0, 40.0, 40.0), 0.0);
    }

    #[test]
    fn nearest_low_breaks_ties_toward_low() {
        assert!(nearest_low(10.0, 0.0, 20.0));
        assert!(nearest_low(10.0, 0.0, 20.0));
        assert!(!nearest_low(16.0, 0.0, 20.0));
        assert!(!nearest_low(21.0, 0.0, 20.0));
    }
}
