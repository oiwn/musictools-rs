//! NeonSlider — thin inset track, a blue-to-cyan glowing fill, and a round
//! handle with a bright rim; plus a double-thumb range variant whose fill
//! spans the thumbs. Framework-free like the rest of the kit.

use crate::{glow, palette};
use egui::epaint::{Vertex, WHITE_UV};
use egui::{Color32, Mesh, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

const TRACK_HEIGHT: f32 = 4.0;
const HANDLE_RADIUS: f32 = 7.0;
/// Horizontal room the handle needs at either end of its travel.
const HANDLE_WIDTH: f32 = 2.0 * HANDLE_RADIUS;
const ROW_HEIGHT: f32 = 22.0;

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
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click_and_drag());
    if response.double_clicked() {
        *value = default;
        response.mark_changed();
    }
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let (left, right) = track_span(rect);
        let next = pos_to_value(pos.x, left, right);
        if next != *value {
            *value = next;
            response.mark_changed();
        }
    }
    let look = Look::of(ui, &response);
    paint_single(ui.painter(), rect, *value, look);
    response
}

/// Double-thumb range slider; thumbs cannot cross.
pub fn range_slider(ui: &mut Ui, low: &mut f64, high: &mut f64, width: f32) -> Response {
    let (rect, mut response) =
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
        let next_low = if moving_low { value.min(*high) } else { *low };
        let next_high = if moving_low { *high } else { value.max(*low) };
        if next_low != *low || next_high != *high {
            *low = next_low;
            *high = next_high;
            response.mark_changed();
        }
    }

    let look = Look::of(ui, &response);
    paint_range(ui.painter(), rect, *low, *high, look);
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

/// Interaction state the painter draws.
#[derive(Clone, Copy)]
struct Look {
    hovered: bool,
    held: bool,
    enabled: bool,
}

impl Look {
    fn of(ui: &Ui, response: &Response) -> Self {
        Self {
            hovered: response.hovered(),
            held: response.is_pointer_button_down_on() || response.dragged(),
            enabled: ui.is_enabled(),
        }
    }
}

fn track_rect(rect: Rect) -> Rect {
    Rect::from_center_size(rect.center(), Vec2::new(rect.width(), TRACK_HEIGHT))
}

/// Track, then the glowing fill from `from_x` to `to_x`.
fn paint_track_and_fill(painter: &Painter, rect: Rect, from_x: f32, to_x: f32, look: Look) {
    let track = track_rect(rect);
    let radius = TRACK_HEIGHT / 2.0;
    painter.rect_filled(track, radius, palette::INSET);
    painter.rect_stroke(
        track,
        radius,
        Stroke::new(palette::HAIRLINE, palette::STROKE),
        egui::StrokeKind::Outside,
    );
    if to_x <= from_x {
        return;
    }
    let fill = Rect::from_min_max(
        Pos2::new(from_x, track.top()),
        Pos2::new(to_x, track.bottom()),
    );
    if !look.enabled {
        painter.rect_filled(fill, radius, palette::TEXT_MUTED);
        return;
    }
    let mid = [
        Pos2::new(from_x, track.center().y),
        Pos2::new(to_x, track.center().y),
    ];
    glow::glow_line_with(painter, &mid, palette::ACCENT, 2, TRACK_HEIGHT + 2.0, 0.25);
    painter.add(horizontal_gradient(fill, palette::BLUE, palette::CYAN));
}

/// A quad shading from `left` to `right` across `rect`.
fn horizontal_gradient(rect: Rect, left: Color32, right: Color32) -> Shape {
    let mut mesh = Mesh::default();
    for (pos, color) in [
        (rect.left_top(), left),
        (rect.right_top(), right),
        (rect.right_bottom(), right),
        (rect.left_bottom(), left),
    ] {
        mesh.vertices.push(Vertex {
            pos,
            uv: WHITE_UV,
            color,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    Shape::mesh(mesh)
}

/// Round handle: Surface center, bright rim, accent glow that grows with
/// interaction.
fn paint_handle(painter: &Painter, x: f32, mid_y: f32, look: Look) {
    let center = Pos2::new(x, mid_y);
    if !look.enabled {
        painter.circle_filled(center, HANDLE_RADIUS, palette::SURFACE);
        painter.circle_stroke(
            center,
            HANDLE_RADIUS - 1.0,
            Stroke::new(2.0_f32, palette::TEXT_MUTED),
        );
        return;
    }
    let layers = if look.held {
        3
    } else if look.hovered {
        2
    } else {
        1
    };
    glow::glow_circle(painter, center, HANDLE_RADIUS, palette::ACCENT, layers);
    painter.circle_filled(center, HANDLE_RADIUS, palette::SURFACE);
    painter.circle_stroke(
        center,
        HANDLE_RADIUS - 1.0,
        Stroke::new(2.0_f32, palette::CYAN.lerp_to_gamma(Color32::WHITE, 0.55)),
    );
}

fn paint_single(painter: &Painter, rect: Rect, value: f64, look: Look) {
    let x = value_x(value, rect);
    paint_track_and_fill(painter, rect, rect.left(), x, look);
    paint_handle(painter, x, rect.center().y, look);
}

fn paint_range(painter: &Painter, rect: Rect, low: f64, high: f64, look: Look) {
    let (low_x, high_x) = (value_x(low, rect), value_x(high, rect));
    paint_track_and_fill(painter, rect, low_x, high_x, look);
    paint_handle(painter, low_x, rect.center().y, look);
    paint_handle(painter, high_x, rect.center().y, look);
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
