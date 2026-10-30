//! Text helpers: letter-spaced (tracked) caps for labels and titles.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, FontId, Galley, Painter, Pos2, Rect};
use std::sync::Arc;

/// Lay out `text` with `spacing` px of extra letter spacing.
#[must_use]
pub fn tracked_galley(
    painter: &Painter,
    text: &str,
    font: FontId,
    color: Color32,
    spacing: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: font,
            color,
            extra_letter_spacing: spacing,
            ..Default::default()
        },
    );
    painter.layout_job(job)
}

/// Paint tracked text anchored like `Painter::text`; returns its rect.
pub fn tracked(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    text: &str,
    font: FontId,
    color: Color32,
    spacing: f32,
) -> Rect {
    let galley = tracked_galley(painter, text, font, color, spacing);
    let rect = align.anchor_size(pos, galley.size());
    painter.galley(rect.min, galley, color);
    rect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_widens_the_galley() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter();
            let font = FontId::proportional(11.0);
            let plain = tracked_galley(painter, "BOGDAN", font.clone(), Color32::WHITE, 0.0);
            let wide = tracked_galley(painter, "BOGDAN", font, Color32::WHITE, 3.0);
            assert!(wide.size().x > plain.size().x + 10.0);
        });
        // Headless: no renderer consumes the font atlas upload.
        output.textures_delta.clear();
    }
}
