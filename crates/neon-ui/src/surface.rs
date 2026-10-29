//! Surface painters — the four Dark Neon surface roles as fill plus hairline
//! stroke. Pure painting, no interaction; widgets in later tasks allocate their
//! rects and call these.

use crate::palette::{self, HAIRLINE};
use egui::{Color32, Painter, Rect, Stroke, StrokeKind};

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
pub fn inset(painter: &Painter, rect: Rect) {
    paint_surface(
        painter,
        rect,
        palette::INSET,
        palette::STROKE,
        palette::RADIUS_SMALL,
    );
}
