//! The Dark Neon egui theme.
//!
//! [`dark_neon_visuals`] is what editors install with `ctx.set_visuals` (the
//! plugins do it in their nice-plug-egui build callback; the showcase at
//! startup). egui keeps text styles and spacing on `Style`, not `Visuals`, so
//! this covers fills, widget colors, selection, corner radii, and shadows;
//! typography is set per-painter by the kit's widgets.

use crate::palette::{self, HAIRLINE};
use egui::{Color32, CornerRadius, Stroke, Visuals};

/// The Dark Neon `Visuals`: window/panel/inset fills, the widget-state ladder
/// over the surface colors, cyan selection and links, 2-3 px corners, and no
/// drop shadows — depth comes from the surface ladder, not blur.
#[must_use]
pub fn dark_neon_visuals() -> Visuals {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = palette::BG;
    visuals.window_fill = palette::PANEL;
    visuals.extreme_bg_color = palette::INSET;
    visuals.faint_bg_color = palette::PANEL;
    visuals.text_edit_bg_color = Some(palette::INSET);
    visuals.code_bg_color = palette::INSET;

    visuals.widgets.noninteractive.bg_fill = palette::PANEL;
    visuals.widgets.noninteractive.bg_stroke = hairline(palette::STROKE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(HAIRLINE, palette::TEXT_MUTED);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::from(palette::RADIUS_SMALL);

    visuals.widgets.inactive.bg_fill = palette::SURFACE;
    visuals.widgets.inactive.bg_stroke = hairline(palette::STROKE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(HAIRLINE, palette::TEXT_SECONDARY);
    visuals.widgets.inactive.corner_radius = CornerRadius::from(palette::RADIUS_SMALL);

    visuals.widgets.hovered.bg_fill = palette::RAISED;
    visuals.widgets.hovered.bg_stroke = hairline(palette::STROKE_BRIGHT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(HAIRLINE, palette::TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = CornerRadius::from(palette::RADIUS);

    visuals.widgets.active.bg_fill = palette::ACCENT;
    visuals.widgets.active.fg_stroke = Stroke::new(HAIRLINE, palette::BG);
    visuals.widgets.active.corner_radius = CornerRadius::from(palette::RADIUS);

    visuals.widgets.open.bg_fill = palette::PANEL;
    visuals.widgets.open.bg_stroke = hairline(palette::STROKE);
    visuals.widgets.open.fg_stroke = Stroke::new(HAIRLINE, palette::TEXT_PRIMARY);
    visuals.widgets.open.corner_radius = CornerRadius::from(palette::RADIUS);

    visuals.selection.bg_fill = palette::ACCENT.linear_multiply(0.35);
    visuals.selection.stroke = Stroke::new(HAIRLINE, palette::ACCENT);
    visuals.hyperlink_color = palette::ACCENT;

    visuals.override_text_color = Some(palette::TEXT_PRIMARY);
    visuals.warn_fg_color = palette::WARNING;
    visuals.error_fg_color = palette::CLIP;

    visuals.window_corner_radius = CornerRadius::from(palette::RADIUS);
    visuals.window_stroke = hairline(palette::STROKE_BRIGHT);
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.menu_corner_radius = CornerRadius::from(palette::RADIUS);
    visuals.popup_shadow = egui::Shadow::NONE;

    visuals
}

fn hairline(color: Color32) -> Stroke {
    Stroke::new(HAIRLINE, color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette;

    #[test]
    fn dark_neon_visuals_constructs_with_the_neon_fills() {
        let visuals = dark_neon_visuals();
        assert_eq!(visuals.panel_fill, palette::BG);
        assert_eq!(visuals.window_fill, palette::PANEL);
        assert_eq!(visuals.extreme_bg_color, palette::INSET);
        assert_eq!(visuals.faint_bg_color, palette::PANEL);
        assert_eq!(visuals.override_text_color, Some(palette::TEXT_PRIMARY));
        assert_eq!(visuals.widgets.noninteractive.bg_fill, palette::PANEL);
        assert_eq!(visuals.widgets.inactive.bg_fill, palette::SURFACE);
        assert_eq!(visuals.widgets.hovered.bg_fill, palette::RAISED);
        assert_eq!(visuals.widgets.active.bg_fill, palette::ACCENT);
        assert_eq!(visuals.selection.stroke.color, palette::ACCENT);
        assert_eq!(visuals.hyperlink_color, palette::ACCENT);
        assert_eq!(visuals.warn_fg_color, palette::WARNING);
        assert_eq!(visuals.error_fg_color, palette::CLIP);
    }

    #[test]
    fn dark_neon_visuals_shadows_are_off() {
        let visuals = dark_neon_visuals();
        assert_eq!(visuals.window_shadow, egui::Shadow::NONE);
        assert_eq!(visuals.popup_shadow, egui::Shadow::NONE);
    }

    #[test]
    fn dark_neon_visuals_is_dark_mode() {
        assert!(dark_neon_visuals().dark_mode);
    }
}
