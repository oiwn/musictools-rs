//! Dark Neon palette — the color source of truth for every editor in the suite.
//!
//! The values are normative in `specs/design.md`; where a rendered editor and
//! the spec disagree, the sketch (`specs/design-sketch.png`) is canonical.

use egui::Color32;

// Backgrounds, in depth order: window down to wells.
/// Window background, the darkest layer.
pub const BG: Color32 = Color32::from_rgb(0x0E, 0x14, 0x1B);
/// Panels and cards holding sections.
pub const PANEL: Color32 = Color32::from_rgb(0x1A, 0x22, 0x2C);
/// Controls: buttons, closed selects, slider tracks.
pub const SURFACE: Color32 = Color32::from_rgb(0x23, 0x2C, 0x36);
/// Hover fills, icon buttons, knob bodies.
pub const RAISED: Color32 = Color32::from_rgb(0x2D, 0x39, 0x45);
/// Graph wells and value readouts sunk into a panel.
pub const INSET: Color32 = Color32::from_rgb(0x12, 0x19, 0x21);

// Accents, used sparingly; one accent leads per plugin.
/// Primary accent: active states, meter fills, focus rings.
pub const CYAN: Color32 = Color32::from_rgb(0x39, 0xD9, 0xFF);
/// Secondary accent: alternate curves and labels.
pub const BLUE: Color32 = Color32::from_rgb(0x3A, 0x6B, 0xFF);
/// Dry/mix-side marks, secondary readouts, warnings.
pub const ORANGE: Color32 = Color32::from_rgb(0xFF, 0x98, 0x00);
/// Clip, ceiling, destructive.
pub const RED: Color32 = Color32::from_rgb(0xF3, 0x4D, 0x57);
/// Highlight ticks, snap indicators.
pub const YELLOW: Color32 = Color32::from_rgb(0xE6, 0xC3, 0x5C);

// Text greys, from primary down to ghost.
/// Primary text color.
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF6, 0xED, 0xF6);
/// Secondary text: labels, captions.
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x9A, 0xA5, 0xB1);
/// Muted text: sub-labels.
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x63, 0x70, 0x7E);
/// Ghost text: grids, disabled.
pub const TEXT_GHOST: Color32 = Color32::from_rgb(0x40, 0x4A, 0x55);

// Hairlines: panel frames and grid lines.
/// Panel hairline.
pub const STROKE: Color32 = Color32::from_rgb(0x26, 0x31, 0x3D);
/// Brighter hairline for grids and window frames.
pub const STROKE_BRIGHT: Color32 = Color32::from_rgb(0x31, 0x3E, 0x4B);

// Semantic aliases. The leading accent is per plugin; cyan is the default.
/// The suite-default leading accent.
pub const ACCENT: Color32 = CYAN;
/// Alternate accent for the secondary curve or label.
pub const ACCENT_ALT: Color32 = BLUE;
/// Warning mark.
pub const WARNING: Color32 = ORANGE;
/// Clip mark.
pub const CLIP: Color32 = RED;
/// Highlight or snap mark.
pub const HIGHLIGHT: Color32 = YELLOW;

// Layout canon.
/// Corner radius for panels, cards, and controls (2-3 px per the style guide).
pub const RADIUS: f32 = 3.0;
/// Tighter radius for inset wells.
pub const RADIUS_SMALL: f32 = 2.0;
/// Hairline stroke width.
pub const HAIRLINE: f32 = 1.0;
/// Spacing grid.
pub const SPACING: f32 = 8.0;

const _: () = assert!(RADIUS_SMALL <= RADIUS);

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_hex(color: Color32, hex: u32) {
        assert_eq!(
            (color.r(), color.g(), color.b()),
            ((hex >> 16) as u8, (hex >> 8) as u8, hex as u8),
            "color does not match #{hex:06X}"
        );
    }

    #[test]
    fn palette_matches_design_spec_hex_values() {
        assert_hex(BG, 0x0E141B);
        assert_hex(PANEL, 0x1A222C);
        assert_hex(SURFACE, 0x232C36);
        assert_hex(RAISED, 0x2D3945);
        assert_hex(INSET, 0x121921);
        assert_hex(CYAN, 0x39D9FF);
        assert_hex(BLUE, 0x3A6BFF);
        assert_hex(ORANGE, 0xFF9800);
        assert_hex(RED, 0xF34D57);
        assert_hex(YELLOW, 0xE6C35C);
        assert_hex(TEXT_PRIMARY, 0xF6EDF6);
        assert_hex(TEXT_SECONDARY, 0x9AA5B1);
        assert_hex(TEXT_MUTED, 0x63707E);
        assert_hex(TEXT_GHOST, 0x404A55);
        assert_hex(STROKE, 0x26313D);
        assert_hex(STROKE_BRIGHT, 0x313E4B);
    }

    #[test]
    fn semantic_aliases_point_at_their_accent_colors() {
        assert_eq!(ACCENT, CYAN);
        assert_eq!(ACCENT_ALT, BLUE);
        assert_eq!(WARNING, ORANGE);
        assert_eq!(CLIP, RED);
        assert_eq!(HIGHLIGHT, YELLOW);
    }

    #[test]
    fn corner_radii_stay_within_design_bounds() {
        assert!((2.0..=3.0).contains(&RADIUS));
        assert!((2.0..=3.0).contains(&RADIUS_SMALL));
    }
}
