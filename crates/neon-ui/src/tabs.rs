//! NeonTabs — a segmented control on a Surface strip with hairline dividers.
//! The active segment takes a deep-blue gradient and a glowing accent bar
//! (below for `tabs`, leading for `tabs_vertical`); other segments light on
//! hover and sink on press. Disabled strips stay readable: the active segment
//! turns grey instead of vanishing.

use crate::{glow, palette, surface, text};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

const HEIGHT: f32 = 26.0;
const BAR: f32 = 2.0;

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
    (labels.iter().map(|l| l.len() as f32).sum::<f32>() * 7.5 + 22.0 * labels.len() as f32)
        .max(120.0)
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

/// Horizontal tabs; `current` is the selected index.
pub fn tabs(ui: &mut Ui, current: &mut usize, labels: &[&str]) -> Response {
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(tabs_width(labels), HEIGHT), Sense::click());
    let pick = |pos: Pos2| segment_at(pos.x, rect, labels.len()).min(labels.len() - 1);
    select_on_click(&mut response, current, pick);
    let input = SegmentInput::read(ui, &response, pick);
    paint(
        ui.painter(),
        rect,
        *current,
        labels,
        input,
        Axis::Horizontal,
    );
    response
}

/// Vertically stacked tabs filling `height`; `current` is the selected index.
pub fn tabs_vertical(ui: &mut Ui, current: &mut usize, labels: &[&str], height: f32) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(tabs_vertical_width(labels), height),
        Sense::click(),
    );
    let pick = |pos: Pos2| {
        segment_at_y(pos.y - rect.top(), rect.height(), labels.len()).min(labels.len() - 1)
    };
    select_on_click(&mut response, current, pick);
    let input = SegmentInput::read(ui, &response, pick);
    paint(ui.painter(), rect, *current, labels, input, Axis::Vertical);
    response
}

fn select_on_click(response: &mut Response, current: &mut usize, pick: impl Fn(Pos2) -> usize) {
    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        let next = pick(pos);
        if next != *current {
            *current = next;
            response.mark_changed();
        }
    }
}

/// Which segment the pointer is over, and whether it is held there.
#[derive(Clone, Copy)]
struct SegmentInput {
    hovered: Option<usize>,
    pressed: bool,
    enabled: bool,
}

impl SegmentInput {
    fn read(ui: &Ui, response: &Response, pick: impl Fn(Pos2) -> usize) -> Self {
        let enabled = ui.is_enabled();
        Self {
            hovered: response.hover_pos().filter(|_| enabled).map(pick),
            pressed: enabled && response.is_pointer_button_down_on(),
            enabled,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Axis {
    Horizontal,
    Vertical,
}

fn paint(
    painter: &Painter,
    rect: Rect,
    current: usize,
    labels: &[&str],
    input: SegmentInput,
    axis: Axis,
) {
    if labels.is_empty() {
        return;
    }
    surface::surface(painter, rect);
    let count = labels.len() as f32;
    let segment = |index: usize| match axis {
        Axis::Horizontal => {
            let width = rect.width() / count;
            Rect::from_min_size(
                Pos2::new(rect.left() + index as f32 * width, rect.top()),
                Vec2::new(width, rect.height()),
            )
        }
        Axis::Vertical => {
            let height = rect.height() / count;
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.top() + index as f32 * height),
                Vec2::new(rect.width(), height),
            )
        }
    };

    // Dividers between segments, skipped next to the active one.
    for index in 1..labels.len() {
        if index == current || index == current + 1 {
            continue;
        }
        let seg = segment(index);
        let (from, to) = match axis {
            Axis::Horizontal => (
                Pos2::new(seg.left(), seg.top() + 5.0),
                Pos2::new(seg.left(), seg.bottom() - 5.0),
            ),
            Axis::Vertical => (
                Pos2::new(seg.left() + 8.0, seg.top()),
                Pos2::new(seg.right() - 8.0, seg.top()),
            ),
        };
        painter.line_segment([from, to], Stroke::new(palette::HAIRLINE, palette::STROKE));
    }

    for (index, label) in labels.iter().enumerate() {
        let seg = segment(index).shrink(1.0);
        let active = index == current;
        let hovered = input.hovered == Some(index);
        if active && input.enabled {
            paint_active(painter, seg, axis);
        } else if active {
            // Disabled: keep the selection visible, without the accent.
            painter.rect_filled(seg, palette::RADIUS_SMALL, palette::RAISED);
        } else if hovered && input.pressed {
            surface::inset(painter, seg);
        } else if hovered {
            surface::raised_lit(painter, seg);
        }

        // `add_enabled_ui` already fades disabled strips; Secondary keeps
        // their labels readable after that fade.
        let color = if active || hovered {
            palette::TEXT_PRIMARY
        } else {
            palette::TEXT_SECONDARY
        };
        text::tracked(
            painter,
            seg.center(),
            Align2::CENTER_CENTER,
            &label.to_uppercase(),
            FontId::proportional(10.5),
            color,
            0.6,
        );
    }
}

/// Active segment: deep-blue gradient plus the glowing accent bar.
fn paint_active(painter: &Painter, seg: Rect, axis: Axis) {
    surface::gradient_rect(
        painter,
        seg,
        palette::RADIUS_SMALL,
        deep_blue(0.7),
        deep_blue(0.84),
    );
    let bar = match axis {
        Axis::Horizontal => Rect::from_min_size(
            Pos2::new(seg.left(), seg.bottom() - BAR),
            Vec2::new(seg.width(), BAR),
        ),
        Axis::Vertical => Rect::from_min_size(seg.left_top(), Vec2::new(BAR, seg.height())),
    };
    glow::glow_rect(painter, bar, 0.0, palette::ACCENT, 2);
    painter.rect_filled(bar, 0.0, palette::ACCENT);
}

/// Blue sunk toward Inset by `depth` (0 = Blue, 1 = Inset).
fn deep_blue(depth: f32) -> Color32 {
    palette::BLUE.lerp_to_gamma(palette::INSET, depth)
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
