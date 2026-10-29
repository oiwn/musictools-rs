//! NeonTabs — a segmented control: the active segment sits on Surface with an
//! accent underline glow. `tabs` lays the segments out horizontally;
//! `tabs_vertical` stacks them, for controls sitting beside tall columns.

use crate::{glow, palette, surface};
use egui::{Align2, FontId, Painter, Pos2, Rect, Response, Sense, Ui, Vec2};

const HEIGHT: f32 = 22.0;
const UNDERLINE: f32 = 2.0;

/// Segment index for an x position inside a `segments`-wide `rect`.
#[must_use]
pub fn segment_at(x: f32, rect: Rect, segments: usize) -> usize {
    if segments == 0 {
        return 0;
    }
    let t = ((x - rect.left()) / rect.width()).clamp(0.0, 1.0_f32 - f32::EPSILON);
    (t * segments as f32) as usize
}

/// Segment index for a y offset inside a `segments`-tall `extent`.
#[must_use]
pub fn segment_at_y(y: f32, extent: f32, segments: usize) -> usize {
    if segments == 0 {
        return 0;
    }
    let t = (y / extent).clamp(0.0, 1.0_f32 - f32::EPSILON);
    (t * segments as f32) as usize
}

/// Width a `tabs` control occupies for `labels` — the same formula `tabs`
/// uses, so editors can precompute row centering.
#[must_use]
pub fn tabs_width(labels: &[&str]) -> f32 {
    (labels.iter().map(|l| l.len() as f32).sum::<f32>() * 6.5 + 40.0).max(120.0)
}

/// Width a `tabs_vertical` control occupies for `labels`: the longest label
/// plus padding, floored so short stacks still read as a column.
#[must_use]
pub fn tabs_vertical_width(labels: &[&str]) -> f32 {
    let longest = labels
        .iter()
        .map(|l| l.len() as f32)
        .fold(0.0_f32, f32::max);
    (longest * 6.5 + 16.0).max(64.0)
}

/// Segmented tabs over `labels`; `current` is the selected index.
pub fn tabs(ui: &mut Ui, current: &mut usize, labels: &[&str]) -> Response {
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(tabs_width(labels), HEIGHT), Sense::click());

    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        let next = segment_at(pos.x, rect, labels.len()).min(labels.len() - 1);
        if next != *current {
            *current = next;
            response.mark_changed();
        }
    }

    paint(ui.painter(), rect, *current, labels, ui.is_enabled());
    response
}

/// Stacked segmented tabs filling `height` — the vertical companion to
/// [`tabs`], sized to stand beside a full knob column.
pub fn tabs_vertical(ui: &mut Ui, current: &mut usize, labels: &[&str], height: f32) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(tabs_vertical_width(labels), height),
        Sense::click(),
    );

    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        let next =
            segment_at_y(pos.y - rect.top(), rect.height(), labels.len()).min(labels.len() - 1);
        if next != *current {
            *current = next;
            response.mark_changed();
        }
    }

    paint_vertical(ui.painter(), rect, *current, labels, ui.is_enabled());
    response
}

fn paint(painter: &Painter, rect: Rect, current: usize, labels: &[&str], enabled: bool) {
    let seg_width = rect.width() / labels.len() as f32;
    for (index, label) in labels.iter().enumerate() {
        let seg = Rect::from_min_size(
            Pos2::new(rect.left() + index as f32 * seg_width, rect.top()),
            Vec2::new(seg_width, rect.height()),
        );
        paint_segment(painter, seg, index, current, label, Bar::Below, enabled);
    }
}

/// Where a segment's accent bar sits.
enum Bar {
    Below,
    Leading,
}

fn paint_segment(
    painter: &Painter,
    seg: Rect,
    index: usize,
    current: usize,
    label: &str,
    bar: Bar,
    enabled: bool,
) {
    // A disabled stack still shows its stored selection, but ghosted: no
    // Surface fill, no accent bar, no highlight text.
    let active = index == current && enabled;
    if active {
        surface::surface(painter, seg);
        let bar_rect = match bar {
            Bar::Below => Rect::from_min_size(
                Pos2::new(seg.left(), seg.bottom() - UNDERLINE),
                Vec2::new(seg.width(), UNDERLINE),
            ),
            Bar::Leading => Rect::from_min_size(seg.left_top(), Vec2::new(UNDERLINE, seg.height())),
        };
        glow::glow_rect(painter, bar_rect, 0.0, palette::ACCENT, 2);
        painter.rect_filled(bar_rect, 0.0, palette::ACCENT);
    }
    painter.text(
        seg.center(),
        Align2::CENTER_CENTER,
        label.to_uppercase(),
        FontId::proportional(10.0),
        if !enabled {
            palette::TEXT_GHOST
        } else if active {
            palette::TEXT_PRIMARY
        } else {
            palette::TEXT_MUTED
        },
    );
}

fn paint_vertical(painter: &Painter, rect: Rect, current: usize, labels: &[&str], enabled: bool) {
    let seg_height = rect.height() / labels.len() as f32;
    for (index, label) in labels.iter().enumerate() {
        let seg = Rect::from_min_size(
            Pos2::new(rect.left(), rect.top() + index as f32 * seg_height),
            Vec2::new(rect.width(), seg_height),
        );
        paint_segment(painter, seg, index, current, label, Bar::Leading, enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_at_maps_positions_to_indices() {
        let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(90.0, 20.0));
        assert_eq!(segment_at(0.0, rect, 3), 0);
        assert_eq!(segment_at(30.0, rect, 3), 1);
        assert_eq!(segment_at(89.9, rect, 3), 2);
    }

    #[test]
    fn segment_at_clamps_outside_positions() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 0.0), Vec2::new(60.0, 20.0));
        assert_eq!(segment_at(-5.0, rect, 3), 0);
        assert_eq!(segment_at(500.0, rect, 3), 2);
        assert_eq!(segment_at(50.0, rect, 0), 0);
    }

    #[test]
    fn segment_at_y_maps_offsets_to_indices() {
        assert_eq!(segment_at_y(0.0, 86.0, 3), 0);
        assert_eq!(segment_at_y(29.0, 86.0, 3), 1);
        assert_eq!(segment_at_y(85.9, 86.0, 3), 2);
        assert_eq!(segment_at_y(-5.0, 86.0, 2), 0);
        assert_eq!(segment_at_y(500.0, 86.0, 2), 1);
        assert_eq!(segment_at_y(10.0, 86.0, 0), 0);
    }

    #[test]
    fn tabs_vertical_width_covers_the_longest_label() {
        assert!(tabs_vertical_width(&["SINE", "TRIANGLE"]) >= 8.0 * 6.5);
        assert_eq!(tabs_vertical_width(&["A", "B"]), 64.0);
    }
}
