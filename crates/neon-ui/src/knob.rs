//! NeonKnob — the suite's knob: pure geometry and drag math, a painter, and
//! the interactive widget. Framework-free: the widget holds a plain normalized
//! value and returns the `egui::Response`.

use crate::{glow, palette, surface, text};
use egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
};

/// Knob size classes from the style guide.
#[derive(Clone, Copy, Debug, Default)]
pub enum KnobSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl KnobSize {
    /// Body diameter in px; the widget footprint adds readout and label rows.
    #[must_use]
    pub fn diameter(self) -> f32 {
        match self {
            Self::Small => 36.0,
            Self::Medium => 48.0,
            Self::Large => 64.0,
        }
    }

    /// Widget footprint: the diameter plus the readout and label rows, with
    /// horizontal padding for the arc glow.
    #[must_use]
    pub fn footprint(self) -> Vec2 {
        Vec2::new(self.diameter() + 8.0, self.diameter() + 38.0)
    }
}

/// Arc sweep: 270 degrees from bottom-left to bottom-right.
const SWEEP: f32 = 1.5 * std::f32::consts::PI;
/// Arc start angle in screen radians (y grows down): 135 degrees points down-left.
const START_ANGLE: f32 = 0.75 * std::f32::consts::PI;
/// Snap pull distance in normalized value units.
const SNAP_EPSILON: f64 = 0.02;
/// Full value sweep per dragged pixel at normal speed.
const DRAG_SPEED: f64 = 1.0 / 150.0;
/// Shift divides the drag speed by this factor.
const FINE_FACTOR: f64 = 4.0;

/// Knob configuration beyond the plain value.
pub struct KnobParams<'a> {
    pub size: KnobSize,
    pub accent: Color32,
    /// Normalized points the value snaps to; drawn as the tick ring.
    pub snap_points: &'a [f64],
    /// Value restored on double-click.
    pub default: f64,
    /// Draw the readout on an inset pill (for editable entry) instead of
    /// plain text.
    pub pill: bool,
}

impl Default for KnobParams<'_> {
    fn default() -> Self {
        Self {
            size: KnobSize::Medium,
            accent: palette::ACCENT,
            snap_points: &[],
            default: 0.5,
            pill: false,
        }
    }
}

/// Interaction state the painter draws.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct KnobLook {
    hovered: bool,
    /// Pointer held on the knob (pressed or dragging).
    held: bool,
    enabled: bool,
}

/// Width of the track and value arcs.
const ARC_WIDTH: f32 = 4.0;

/// New normalized value after a vertical drag (up increases). Clamps to 0..=1.
#[must_use]
pub fn drag_value(value: f64, delta_y: f64, fine: bool) -> f64 {
    let speed = if fine {
        DRAG_SPEED / FINE_FACTOR
    } else {
        DRAG_SPEED
    };
    (value - delta_y * speed).clamp(0.0, 1.0)
}

/// Index of the snap point within `SNAP_EPSILON` of `value`, if any.
#[must_use]
pub fn snap_index(value: f64, snap_points: &[f64]) -> Option<usize> {
    snap_points
        .iter()
        .position(|&point| (point - value).abs() <= SNAP_EPSILON)
}

/// Screen angle (radians, y down) for a normalized value.
#[must_use]
pub fn value_angle(value: f64) -> f32 {
    START_ANGLE + value.clamp(0.0, 1.0) as f32 * SWEEP
}

/// Polyline along an arc from `start` to `end` (radians, y down).
#[must_use]
pub fn arc_points(center: Pos2, radius: f32, start: f32, end: f32, segments: usize) -> Vec<Pos2> {
    (0..=segments)
        .map(|step| {
            let t = step as f32 / segments as f32;
            let angle = start + (end - start) * t;
            center + radius * Vec2::new(angle.cos(), angle.sin())
        })
        .collect()
}

/// Knob body: a fan mesh from a `Panel` center out to a rim lit from the
/// top-left (lifted `Raised`) and falling into shadow (`Inset`) bottom-right,
/// which reads as a dome.
pub fn body_mesh(center: Pos2, radius: f32, segments: usize) -> Shape {
    let mut mesh = egui::Mesh::default();
    mesh.vertices.reserve(segments + 2);
    mesh.indices.reserve(segments * 3);
    let uv = egui::epaint::WHITE_UV;
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv,
        color: palette::PANEL,
    });
    let lit = surface::lift(palette::RAISED);
    let light = Vec2::new(-1.0, -1.0).normalized();
    for step in 0..=segments {
        let angle = std::f32::consts::TAU * step as f32 / segments as f32;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let t = (dir.dot(light) + 1.0) / 2.0;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + radius * dir,
            uv,
            color: palette::INSET.lerp_to_gamma(lit, t),
        });
    }
    for step in 0..segments {
        mesh.indices
            .extend_from_slice(&[0, step as u32 + 1, step as u32 + 2]);
    }
    Shape::Mesh(std::sync::Arc::new(mesh))
}

/// Knob with default parameters (`Medium`, cyan accent, no snap points).
pub fn knob(ui: &mut Ui, value: &mut f64, label: &str, fmt: &dyn Fn(f64) -> String) -> Response {
    knob_with(ui, value, label, fmt, &KnobParams::default())
}

/// Knob with full configuration. `value` is normalized 0..=1.
pub fn knob_with(
    ui: &mut Ui,
    value: &mut f64,
    label: &str,
    fmt: &dyn Fn(f64) -> String,
    params: &KnobParams<'_>,
) -> Response {
    let (rect, mut response) =
        ui.allocate_exact_size(params.size.footprint(), Sense::click_and_drag());

    // Double-click or ctrl/⌘-click restores the default. The modifier path
    // is the reliable one under hosts whose window events mangle click pairs.
    if response.double_clicked() || (response.clicked() && ui.input(|input| input.modifiers.ctrl)) {
        *value = params.default;
        response.mark_changed();
    }
    let mut active_snap = None;
    if response.dragged() {
        let fine = ui.input(|input| input.modifiers.shift);
        *value = drag_value(*value, response.drag_delta().y as f64, fine);
        active_snap = snap_index(*value, params.snap_points);
        if let Some(index) = active_snap {
            *value = params.snap_points[index];
        }
        response.mark_changed();
    }

    let look = KnobLook {
        hovered: response.hovered(),
        held: response.is_pointer_button_down_on() || response.dragged(),
        enabled: ui.is_enabled(),
    };
    paint(
        ui.painter(),
        rect,
        *value,
        label,
        fmt,
        params,
        active_snap,
        look,
    );
    response
}

#[allow(clippy::too_many_arguments)]
fn paint(
    painter: &Painter,
    rect: Rect,
    value: f64,
    label: &str,
    fmt: &dyn Fn(f64) -> String,
    params: &KnobParams<'_>,
    active_snap: Option<usize>,
    look: KnobLook,
) {
    let diameter = params.size.diameter();
    let outer = diameter / 2.0;
    let center = pos2(rect.center().x, rect.top() + outer);
    let arc_radius = outer - 3.0;
    let body_radius = 0.66 * outer;
    // `add_enabled_ui` already fades a disabled knob, so Muted (not Ghost)
    // keeps it readable instead of dimming twice.
    let accent = if look.enabled {
        params.accent
    } else {
        palette::TEXT_MUTED
    };

    // Track, then the value arc over its halo; the halo grows with
    // interaction.
    let end = value_angle(value);
    let track = arc_points(center, arc_radius, START_ANGLE, START_ANGLE + SWEEP, 48);
    painter.add(Shape::line(track, Stroke::new(ARC_WIDTH, palette::STROKE)));
    let arc = arc_points(center, arc_radius, START_ANGLE, end, 48);
    if look.enabled {
        let alpha = if look.held {
            0.5
        } else if look.hovered {
            0.35
        } else {
            0.2
        };
        glow::glow_line_with(painter, &arc, accent, 2, ARC_WIDTH + 2.0, alpha);
    }
    painter.add(Shape::line(arc, Stroke::new(ARC_WIDTH, accent)));

    // Body: lit dome, bezel hairline, and a soft shadow just inside it.
    if look.enabled && (look.hovered || look.held) {
        glow::glow_circle(painter, center, body_radius, accent, 1);
    }
    painter.add(body_mesh(center, body_radius, 40));
    painter.circle_stroke(
        center,
        body_radius,
        Stroke::new(palette::HAIRLINE, palette::STROKE_BRIGHT),
    );
    painter.circle_stroke(
        center,
        body_radius - 1.5,
        Stroke::new(1.5_f32, Color32::from_black_alpha(60)),
    );

    // Tick ring: one tick per snap point; the active snap lights Yellow.
    for (index, point) in params.snap_points.iter().enumerate() {
        let angle = value_angle(*point);
        let tick_outer = body_radius - 2.0;
        let tick_inner = tick_outer - 3.0;
        let color = if Some(index) == active_snap {
            palette::HIGHLIGHT
        } else {
            palette::STROKE_BRIGHT
        };
        let dir = Vec2::new(angle.cos(), angle.sin());
        painter.line_segment(
            [center + tick_inner * dir, center + tick_outer * dir],
            Stroke::new(1.5_f32, color),
        );
    }

    // Pointer: primary at rest, accent while held.
    let dir = Vec2::new(end.cos(), end.sin());
    let pointer = if look.enabled && look.held {
        accent
    } else if look.enabled {
        palette::TEXT_PRIMARY
    } else {
        palette::TEXT_MUTED
    };
    painter.line_segment(
        [
            center + 0.3 * body_radius * dir,
            center + 0.82 * body_radius * dir,
        ],
        Stroke::new(2.0_f32, pointer),
    );

    // Readout (plain text, or a pill when editable), then the tracked label.
    let readout_center = pos2(center.x, rect.top() + diameter + 12.0);
    if params.pill {
        let galley = painter.layout(
            fmt(value),
            FontId::monospace(10.0),
            palette::TEXT_PRIMARY,
            f32::INFINITY,
        );
        let pill = Rect::from_center_size(readout_center, galley.size() + Vec2::new(12.0, 5.0));
        surface::inset(painter, pill);
        painter.galley(
            pos2(
                pill.center().x - galley.size().x / 2.0,
                pill.center().y - galley.size().y / 2.0,
            ),
            galley,
            palette::TEXT_PRIMARY,
        );
    } else {
        painter.text(
            readout_center,
            Align2::CENTER_CENTER,
            fmt(value),
            FontId::proportional(11.0),
            palette::TEXT_PRIMARY,
        );
    }
    text::tracked(
        painter,
        pos2(center.x, rect.top() + diameter + 24.0),
        Align2::CENTER_TOP,
        &label.to_uppercase(),
        FontId::proportional(10.0),
        palette::TEXT_SECONDARY,
        0.8,
    );
}

fn pos2(x: f32, y: f32) -> Pos2 {
    Pos2::new(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_angle_sweeps_the_full_range_from_start() {
        assert!((value_angle(0.0) - START_ANGLE).abs() < 1e-6);
        assert!((value_angle(1.0) - (START_ANGLE + SWEEP)).abs() < 1e-6);
        assert!(value_angle(0.25) < value_angle(0.5));
        assert!(value_angle(0.5) < value_angle(0.75));
    }

    #[test]
    fn value_angle_clamps_out_of_range_values() {
        assert!((value_angle(-0.5) - value_angle(0.0)).abs() < 1e-6);
        assert!((value_angle(1.5) - value_angle(1.0)).abs() < 1e-6);
    }

    #[test]
    fn arc_points_hits_both_endpoints() {
        let center = Pos2::new(10.0, 10.0);
        let points = arc_points(center, 5.0, 0.0, std::f32::consts::FRAC_PI_2, 8);
        assert_eq!(points.len(), 9);
        assert!((points[0].x - 15.0).abs() < 1e-4);
        assert!((points[0].y - 10.0).abs() < 1e-4);
        assert!((points[8].x - 10.0).abs() < 1e-4);
        assert!((points[8].y - 15.0).abs() < 1e-4);
    }

    #[test]
    fn drag_value_moves_up_on_upward_drag_and_clamps() {
        assert!((drag_value(0.5, -75.0, false) - 1.0).abs() < 1e-9);
        assert!((drag_value(0.5, 75.0, false) - 0.0).abs() < 1e-9);
        assert_eq!(drag_value(0.0, 50.0, false), 0.0);
        assert_eq!(drag_value(1.0, -50.0, false), 1.0);
    }

    #[test]
    fn drag_value_fine_mode_is_slower() {
        let normal = drag_value(0.5, -30.0, false);
        let fine = drag_value(0.5, -30.0, true);
        assert!(fine < normal);
        assert!((fine - (0.5 + 30.0 / 600.0)).abs() < 1e-9);
    }

    #[test]
    fn snap_index_finds_only_nearby_points() {
        let points = [0.25, 0.5, 0.75];
        assert_eq!(snap_index(0.26, &points), Some(0));
        assert_eq!(snap_index(0.51, &points), Some(1));
        assert_eq!(snap_index(0.4, &points), None);
        assert_eq!(snap_index(0.5, &[]), None);
    }

    #[test]
    fn body_mesh_covers_the_full_circle() {
        let counts = match body_mesh(Pos2::ZERO, 10.0, 16) {
            Shape::Mesh(mesh) => (mesh.vertices.len(), mesh.indices.len()),
            other => panic!("expected a mesh, got {other:?}"),
        };
        assert_eq!(counts, (18, 48));
    }
}
