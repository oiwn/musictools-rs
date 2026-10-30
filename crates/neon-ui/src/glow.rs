//! Glow — layered strokes and fills of one color at falling alpha. No blur
//! passes: the halo reads as light bleeding outward, not egui shadow blur.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind};

/// Default layer count for a subtle neon halo.
pub const LAYERS: usize = 3;

/// Width and alpha for each glow layer, innermost first: widths grow outward,
/// alphas fall strictly. Pure math; the painting helpers below consume it.
#[must_use]
pub fn ladder(layers: usize, base_width: f32, base_alpha: f32) -> Vec<(f32, f32)> {
    (0..layers)
        .map(|index| {
            let step = index as f32;
            (
                base_width * (1.0 + step),
                base_alpha * 0.5_f32.powi(index as i32),
            )
        })
        .collect()
}

/// Halo around a rounded rect: concentric strokes straddling the edge, alpha
/// falling outward. Paint before the element it highlights.
pub fn glow_rect(painter: &Painter, rect: Rect, radius: f32, color: Color32, layers: usize) {
    for (width, alpha) in ladder(layers, 1.0, 0.45) {
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(width, with_alpha(color, alpha)),
            StrokeKind::Middle,
        );
    }
}

/// Glow along a polyline: curve indicators, accent underlines.
pub fn glow_line(painter: &Painter, points: &[Pos2], color: Color32, layers: usize) {
    glow_line_with(painter, points, color, layers, 1.0, 0.45);
}

/// [`glow_line`] scaled for thick strokes: layer widths grow from
/// `base_width` and alphas fall from `base_alpha`. Paint before the stroke.
pub fn glow_line_with(
    painter: &Painter,
    points: &[Pos2],
    color: Color32,
    layers: usize,
    base_width: f32,
    base_alpha: f32,
) {
    for (width, alpha) in ladder(layers, base_width, base_alpha) {
        painter.add(egui::Shape::line(
            points.to_vec(),
            Stroke::new(width, with_alpha(color, alpha)),
        ));
    }
}

/// Radial glow behind a filled circle: LED state dots, focus rings. Paint
/// before the solid dot so the halo stays behind it.
pub fn glow_circle(painter: &Painter, center: Pos2, radius: f32, color: Color32, layers: usize) {
    for (width, alpha) in ladder(layers, 1.0, 0.4) {
        painter.circle_filled(center, radius + width, with_alpha(color, alpha));
    }
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glow_ladder_widths_grow_and_alphas_fall_strictly() {
        let layers = ladder(LAYERS, 1.0, 0.45);
        assert_eq!(layers.len(), LAYERS);
        for pair in layers.windows(2) {
            assert!(pair[1].0 > pair[0].0, "widths must grow: {pair:?}");
            assert!(pair[1].1 < pair[0].1, "alphas must fall: {pair:?}");
        }
        assert!(layers.iter().all(|&(width, _)| width > 0.0));
        assert!(
            layers
                .iter()
                .all(|&(_, alpha)| (0.0..=1.0).contains(&alpha))
        );
    }

    #[test]
    fn glow_ladder_single_layer_is_base_values() {
        assert_eq!(ladder(1, 2.0, 0.5), vec![(2.0, 0.5)]);
    }

    #[test]
    fn glow_ladder_zero_layers_is_empty() {
        assert!(ladder(0, 1.0, 0.4).is_empty());
    }

    #[test]
    fn with_alpha_sets_the_alpha_channel_only() {
        // ecolor stores premultiplied colors, so the RGB channels come back
        // scaled by the alpha: rgba(10, 20, 30, 0.5) is stored as (5, 10, 15).
        let color = Color32::from_rgb(10, 20, 30);
        let faded = with_alpha(color, 0.5);
        assert_eq!(
            (faded.r(), faded.g(), faded.b(), faded.a()),
            (5, 10, 15, 128)
        );
    }
}
