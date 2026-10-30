//! Surface painters — the four Dark Neon surface roles as fill plus hairline
//! stroke, their top-lit gradient variants, and the gradient primitive behind
//! them. Pure painting, no interaction; widgets allocate their rects and call
//! these.

use crate::palette::{self, HAIRLINE};
use egui::epaint::{CornerRadiusF32, RoundedRect, Vertex, WHITE_UV, tessellator::path};
use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

/// How far toward white the top edge of a lit surface is lifted.
const LIFT: f32 = 0.06;

fn paint_surface(painter: &Painter, rect: Rect, fill: Color32, stroke: Color32, radius: f32) {
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(HAIRLINE, stroke),
        StrokeKind::Inside,
    );
}

/// Paint a panel: the card that holds sections.
pub fn panel(painter: &Painter, rect: Rect) {
    paint_surface(
        painter,
        rect,
        palette::PANEL,
        palette::STROKE,
        palette::RADIUS,
    );
}

/// Paint a control surface: buttons, closed selects, slider tracks.
pub fn surface(painter: &Painter, rect: Rect) {
    paint_surface(
        painter,
        rect,
        palette::SURFACE,
        palette::STROKE,
        palette::RADIUS,
    );
}

/// Paint a raised surface: hover fills, icon buttons, knob bodies.
pub fn raised(painter: &Painter, rect: Rect) {
    paint_surface(
        painter,
        rect,
        palette::RAISED,
        palette::STROKE_BRIGHT,
        palette::RADIUS_SMALL,
    );
}

/// Paint an inset well: graph wells and value readouts sunk into a panel.
/// A soft shadow along the top inner edge sells the depth.
pub fn inset(painter: &Painter, rect: Rect) {
    paint_surface(
        painter,
        rect,
        palette::INSET,
        palette::STROKE,
        palette::RADIUS_SMALL,
    );
    inner_shadow(painter, rect);
}

/// Panel lit from above: a faint lift at the top fading to Panel.
pub fn panel_lit(painter: &Painter, rect: Rect) {
    paint_lit(
        painter,
        rect,
        palette::PANEL,
        palette::STROKE,
        palette::RADIUS,
    );
}

/// Control surface lit from above.
pub fn surface_lit(painter: &Painter, rect: Rect) {
    paint_lit(
        painter,
        rect,
        palette::SURFACE,
        palette::STROKE,
        palette::RADIUS,
    );
}

/// Raised surface lit from above: hover fills.
pub fn raised_lit(painter: &Painter, rect: Rect) {
    paint_lit(
        painter,
        rect,
        palette::RAISED,
        palette::STROKE_BRIGHT,
        palette::RADIUS,
    );
}

/// `color` lifted toward white by the lit-surface amount.
#[must_use]
pub fn lift(color: Color32) -> Color32 {
    color.lerp_to_gamma(Color32::WHITE, LIFT)
}

fn paint_lit(painter: &Painter, rect: Rect, base: Color32, stroke: Color32, radius: f32) {
    gradient_rect(painter, rect, radius, lift(base), base);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(HAIRLINE, stroke),
        StrokeKind::Inside,
    );
}

/// Rounded rect filled with a vertical gradient from `top` to `bottom`.
/// The mesh has hard edges; callers stroke a hairline over it, as every
/// surface does, which anti-aliases the outline.
pub fn gradient_rect(painter: &Painter, rect: Rect, radius: f32, top: Color32, bottom: Color32) {
    painter.add(gradient_mesh(rect, radius, top, bottom));
}

/// The gradient fill as a shape: a triangle fan from the rect center over
/// the rounded outline, each vertex colored by its height.
pub fn gradient_mesh(rect: Rect, radius: f32, top: Color32, bottom: Color32) -> Shape {
    let mut outline = Vec::new();
    path::rounded_rectangle(
        &mut outline,
        RoundedRect::new(rect, CornerRadiusF32::same(radius)),
    );
    let color_at = |pos: Pos2| {
        let t = if rect.height() > 0.0 {
            ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        top.lerp_to_gamma(bottom, t)
    };

    let mut mesh = Mesh::default();
    mesh.vertices.reserve(outline.len() + 1);
    mesh.indices.reserve(outline.len() * 3);
    let center = rect.center();
    mesh.vertices.push(Vertex {
        pos: center,
        uv: WHITE_UV,
        color: color_at(center),
    });
    for &pos in &outline {
        mesh.vertices.push(Vertex {
            pos,
            uv: WHITE_UV,
            color: color_at(pos),
        });
    }
    let count = outline.len() as u32;
    for index in 0..count {
        mesh.indices
            .extend_from_slice(&[0, index + 1, (index + 1) % count + 1]);
    }
    Shape::mesh(mesh)
}

/// Soft shadow inside the top edge of `rect`: a few hairlines whose alpha
/// falls away from the edge.
fn inner_shadow(painter: &Painter, rect: Rect) {
    for (step, alpha) in [(1.0_f32, 90_u8), (2.0, 50), (3.0, 22)] {
        let y = rect.top() + step;
        if y >= rect.bottom() {
            break;
        }
        painter.line_segment(
            [
                Pos2::new(rect.left() + palette::RADIUS_SMALL, y),
                Pos2::new(rect.right() - palette::RADIUS_SMALL, y),
            ],
            Stroke::new(HAIRLINE, Color32::from_black_alpha(alpha)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh_of(shape: Shape) -> Mesh {
        match shape {
            Shape::Mesh(mesh) => (*mesh).clone(),
            other => panic!("expected a mesh, got {other:?}"),
        }
    }

    #[test]
    fn gradient_mesh_runs_from_top_color_to_bottom_color() {
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(40.0, 20.0));
        let mesh = mesh_of(gradient_mesh(rect, 0.0, Color32::WHITE, Color32::BLACK));
        let top = mesh.vertices.iter().find(|v| v.pos.y == 0.0).unwrap();
        let bottom = mesh.vertices.iter().find(|v| v.pos.y == 20.0).unwrap();
        assert_eq!(top.color, Color32::WHITE);
        assert_eq!(bottom.color, Color32::BLACK);
    }

    #[test]
    fn gradient_mesh_fan_closes_the_outline() {
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(40.0, 20.0));
        let mesh = mesh_of(gradient_mesh(rect, 4.0, palette::PANEL, palette::BG));
        let rim = mesh.vertices.len() as u32 - 1;
        assert_eq!(mesh.indices.len() as u32, rim * 3);
        // The last triangle wraps back to the first outline vertex.
        assert_eq!(mesh.indices[mesh.indices.len() - 1], 1);
        assert!(mesh.is_valid());
    }

    #[test]
    fn lift_brightens_without_changing_alpha() {
        let lifted = lift(palette::SURFACE);
        assert!(lifted.r() > palette::SURFACE.r());
        assert!(lifted.g() > palette::SURFACE.g());
        assert!(lifted.b() > palette::SURFACE.b());
        assert_eq!(lifted.a(), 255);
    }
}
