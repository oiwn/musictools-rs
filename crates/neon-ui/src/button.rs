//! NeonButton, NeonToggle, and IconButton — Surface fills, hairline strokes,
//! an LED state dot on toggles, and a subtle accent glow on hover.

use crate::{glow, palette, surface};
use egui::{Align2, FontId, Pos2, Response, Sense, Ui, Vec2};

/// Push button with an ALL-CAPS label; check `Response::clicked`.
pub fn button(ui: &mut Ui, label: &str) -> Response {
    let galley = ui.painter().layout(
        label.to_uppercase(),
        FontId::proportional(10.0),
        palette::TEXT_PRIMARY,
        f32::INFINITY,
    );
    let (rect, response) =
        ui.allocate_exact_size(galley.size() + Vec2::new(24.0, 12.0), Sense::click());
    paint_body(ui, rect, response.hovered());
    paint_text(ui, rect.center(), galley);
    response
}

/// Toggle with an LED dot: accent when on, Ghost when off.
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let galley = ui.painter().layout(
        label.to_uppercase(),
        FontId::proportional(10.0),
        palette::TEXT_SECONDARY,
        f32::INFINITY,
    );
    let led = 6.0;
    let size = Vec2::new(10.0 + led + 8.0 + galley.size().x + 10.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
    }

    paint_body(ui, rect, response.hovered());
    let led_center = Pos2::new(rect.left() + 10.0 + led / 2.0, rect.center().y);
    let led_color = if *on {
        palette::ACCENT
    } else {
        palette::TEXT_GHOST
    };
    if *on {
        glow::glow_circle(ui.painter(), led_center, led / 2.0, palette::ACCENT, 2);
    }
    ui.painter().circle_filled(led_center, led / 2.0, led_color);
    ui.painter().galley(
        Pos2::new(
            rect.left() + 10.0 + led + 8.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        palette::TEXT_SECONDARY,
    );
    response
}

/// Square icon button: Raised fill, Ghost glyph, accent on hover.
pub fn icon_button(ui: &mut Ui, glyph: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
    if response.hovered() {
        glow::glow_rect(
            ui.painter(),
            rect,
            palette::RADIUS_SMALL,
            palette::ACCENT,
            2,
        );
    }
    surface::raised(ui.painter(), rect);
    let color = if response.hovered() {
        palette::ACCENT
    } else {
        palette::TEXT_GHOST
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(13.0),
        color,
    );
    response
}

fn paint_body(ui: &Ui, rect: egui::Rect, hovered: bool) {
    if hovered {
        glow::glow_rect(ui.painter(), rect, palette::RADIUS, palette::ACCENT, 2);
        surface::raised(ui.painter(), rect);
    } else {
        surface::surface(ui.painter(), rect);
    }
}

fn paint_text(ui: &Ui, center: Pos2, galley: std::sync::Arc<egui::Galley>) {
    ui.painter().galley(
        Pos2::new(
            center.x - galley.size().x / 2.0,
            center.y - galley.size().y / 2.0,
        ),
        galley,
        palette::TEXT_PRIMARY,
    );
}
