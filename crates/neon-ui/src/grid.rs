//! Graph-grid painters — linear, log-frequency, and spectrum grids at
//! Ghost-alpha hairlines, for the inset graph wells. Pure math plus painting;
//! the tick functions are unit-tested.

use crate::palette;
use egui::{Color32, Painter, Pos2, Rect, Stroke};
/// 1-2-5 ticks per decade within `min..=max` (exclusive of out-of-range).
#[must_use]
pub fn decade_ticks(min: f32, max: f32) -> Vec<f32> {
    let mut ticks = Vec::new();
    if min <= 0.0 || max <= min {
        return ticks;
    }
    for exponent in min.log10().floor() as i32..=max.log10().ceil() as i32 {
        for mantissa in [1.0, 2.0, 5.0] {
            let freq = mantissa * 10.0_f32.powi(exponent);
            if freq >= min && freq <= max {
                ticks.push(freq);
            }
        }
    }
    ticks
}

/// dB grid lines from `floor` to `ceil` (inclusive) at `step`.
#[must_use]
pub fn db_ticks(floor: i32, ceil: i32, step: i32) -> Vec<i32> {
    if step <= 0 || ceil < floor {
        return Vec::new();
    }
    (floor..=ceil).step_by(step as usize).collect()
}

/// X position of a frequency on a log axis spanning `rect`.
#[must_use]
pub fn freq_to_x(freq: f32, rect: Rect, min: f32, max: f32) -> f32 {
    if min <= 0.0 || max <= min || freq <= 0.0 {
        return rect.left();
    }
    let t = (freq.log2() - min.log2()) / (max.log2() - min.log2());
    rect.left() + t.clamp(0.0, 1.0) * rect.width()
}

/// Y position of a dB level on a linear axis spanning `rect`.
#[must_use]
pub fn db_to_y(db: f32, rect: Rect, floor: f32, ceil: f32) -> f32 {
    if ceil <= floor {
        return rect.bottom();
    }
    let t = ((db - floor) / (ceil - floor)).clamp(0.0, 1.0);
    rect.bottom() - t * rect.height()
}

/// Evenly spaced hairline grid: vertical lines every `step_x`, horizontal
/// every `step_y`.
pub fn paint_linear(painter: &Painter, rect: Rect, step_x: f32, step_y: f32) {
    let color = line_color();
    if step_x > 0.0 {
        let mut x = rect.left() + step_x;
        while x < rect.right() {
            grid_line(
                painter,
                Pos2::new(x, rect.top()),
                Pos2::new(x, rect.bottom()),
                color,
            );
            x += step_x;
        }
    }
    if step_y > 0.0 {
        let mut y = rect.top() + step_y;
        while y < rect.bottom() {
            grid_line(
                painter,
                Pos2::new(rect.left(), y),
                Pos2::new(rect.right(), y),
                color,
            );
            y += step_y;
        }
    }
}

/// Log-frequency grid: a hairline per 1-2-5 tick between `min` and `max`.
pub fn paint_log_freq(painter: &Painter, rect: Rect, min: f32, max: f32) {
    let color = line_color();
    for freq in decade_ticks(min, max) {
        let x = freq_to_x(freq, rect, min, max);
        grid_line(
            painter,
            Pos2::new(x, rect.top()),
            Pos2::new(x, rect.bottom()),
            color,
        );
    }
}

/// Spectrum grid: log-frequency verticals plus dB horizontals.
pub fn paint_spectrum(
    painter: &Painter,
    rect: Rect,
    f_min: f32,
    f_max: f32,
    db_floor: i32,
    db_ceiling: i32,
) {
    paint_log_freq(painter, rect, f_min, f_max);
    let color = line_color();
    for db in db_ticks(db_floor, db_ceiling, 6) {
        let y = db_to_y(db as f32, rect, db_floor as f32, db_ceiling as f32);
        grid_line(
            painter,
            Pos2::new(rect.left(), y),
            Pos2::new(rect.right(), y),
            color,
        );
    }
}

fn grid_line(painter: &Painter, from: Pos2, to: Pos2, color: Color32) {
    painter.line_segment([from, to], Stroke::new(1.0_f32, color));
}

fn line_color() -> Color32 {
    let ghost = palette::TEXT_GHOST;
    Color32::from_rgba_unmultiplied(ghost.r(), ghost.g(), ghost.b(), 45)
}

/// Map a series of (x-fraction, value) points into `rect` — helper for curve
/// overlays the editors draw on top of these grids.
#[must_use]
pub fn curve_point(t: f32, value: f32, rect: Rect) -> Pos2 {
    let clamped_t = t.clamp(0.0, 1.0);
    Pos2::new(
        rect.left() + clamped_t * rect.width(),
        rect.bottom() - value.clamp(0.0, 1.0) * rect.height(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Vec2;

    #[test]
    fn decade_ticks_follow_the_125_pattern() {
        let ticks = decade_ticks(20.0, 20_000.0);
        for expected in [
            20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 5000.0, 10_000.0, 20_000.0,
        ] {
            assert!(ticks.contains(&expected), "missing {expected} in {ticks:?}");
        }
        assert!(!ticks.contains(&30.0));
        assert!(!ticks.contains(&300.0));
        assert!(ticks.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn decade_ticks_reject_degenerate_ranges() {
        assert!(decade_ticks(0.0, 100.0).is_empty());
        assert!(decade_ticks(100.0, 100.0).is_empty());
    }

    #[test]
    fn db_ticks_step_from_floor_to_ceiling() {
        assert_eq!(
            db_ticks(-60, 0, 6),
            vec![-60, -54, -48, -42, -36, -30, -24, -18, -12, -6, 0]
        );
        assert!(db_ticks(0, -60, 6).is_empty());
        assert!(db_ticks(-60, 0, 0).is_empty());
    }

    #[test]
    fn freq_to_x_maps_log_axis_endpoints() {
        let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 10.0));
        assert!((freq_to_x(20.0, rect, 20.0, 20_000.0) - 0.0).abs() < 1e-4);
        assert!((freq_to_x(20_000.0, rect, 20.0, 20_000.0) - 100.0).abs() < 1e-4);
        let center = (20.0_f32 * 20_000.0).sqrt();
        assert!((freq_to_x(center, rect, 20.0, 20_000.0) - 50.0).abs() < 1e-3);
        assert!((freq_to_x(1.0, rect, 20.0, 20_000.0) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn db_to_y_maps_ceiling_to_top() {
        let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(100.0, 40.0));
        assert!((db_to_y(0.0, rect, -60.0, 0.0) - 0.0).abs() < 1e-4);
        assert!((db_to_y(-60.0, rect, -60.0, 0.0) - 40.0).abs() < 1e-4);
        assert!((db_to_y(-30.0, rect, -60.0, 0.0) - 20.0).abs() < 1e-4);
        assert_eq!(db_to_y(9.0, rect, 0.0, 0.0), 40.0);
    }

    #[test]
    fn curve_point_clamps_into_the_rect() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(100.0, 50.0));
        assert_eq!(curve_point(0.5, 0.5, rect), Pos2::new(60.0, 35.0));
        assert_eq!(curve_point(-1.0, 2.0, rect), Pos2::new(10.0, 10.0));
        assert_eq!(curve_point(2.0, -1.0, rect), Pos2::new(110.0, 60.0));
    }
}
