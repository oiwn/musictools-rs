//! NeonMeter — L/R bars on an Inset track: gradient fill cyan → yellow → red
//! toward clip, a decaying peak-hold tick, and a red clip latch that a click
//! resets. Framework-free: callers pass plain dB levels.

use crate::{palette, surface};
use egui::{
    Align2, Color32, FontId, Mesh, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
};

const ROW_HEIGHT: f32 = 10.0;
const ROW_GAP: f32 = 5.0;
const LABEL_COLUMN: f32 = 14.0;
/// dB at the left edge of the track.
pub const FLOOR_DB: f64 = -60.0;
/// Levels at or above this latch the clip marker.
pub const CLIP_DB: f64 = -0.1;
/// Peak-hold decay in dB per second.
const PEAK_DECAY_DB: f64 = 24.0;

/// Per-meter persistent state: peak-hold ticks and clip latches per channel.
/// Peaks start at the meter floor, not at zero.
#[derive(Clone, Copy)]
pub struct MeterState {
    pub peaks: [f64; 2],
    pub clip: [bool; 2],
}

impl Default for MeterState {
    fn default() -> Self {
        Self {
            peaks: [FLOOR_DB; 2],
            clip: [false; 2],
        }
    }
}

/// Fraction of the track covered by a dB level, clamped to 0..=1. Non-finite
/// levels other than +inf read as silence.
#[must_use]
pub fn db_to_fraction(db: f64) -> f64 {
    if db.is_nan() {
        0.0
    } else {
        ((db - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0)
    }
}

/// True when a level latches the clip marker.
#[must_use]
pub fn is_clipped(db: f64) -> bool {
    db >= CLIP_DB
}

/// Level-to-color mapping: cyan in the safe range, blending to yellow and
/// then red as the level approaches clip.
#[must_use]
pub fn level_color(db: f64) -> Color32 {
    let yellow_from = -18.0;
    let red_from = -6.0;
    if db <= yellow_from {
        palette::CYAN
    } else if db <= red_from {
        blend(
            palette::CYAN,
            palette::YELLOW,
            (db - yellow_from) / (red_from - yellow_from),
        )
    } else {
        blend(palette::YELLOW, palette::RED, (db - red_from) / (-red_from))
    }
}

fn blend(from: Color32, to: Color32, t: f64) -> Color32 {
    let t = t.clamp(0.0, 1.0) as f32;
    Color32::from_rgb(
        (from.r() as f32 + (to.r() as f32 - from.r() as f32) * t).round() as u8,
        (from.g() as f32 + (to.g() as f32 - from.g() as f32) * t).round() as u8,
        (from.b() as f32 + (to.b() as f32 - from.b() as f32) * t).round() as u8,
    )
}

/// Advance the peak-hold and clip state one frame; pure apart from `dt`.
#[must_use]
pub fn advance_state(state: MeterState, levels_db: [f64; 2], dt: f64) -> MeterState {
    let mut next = state;
    for (channel, level) in levels_db.iter().enumerate() {
        next.peaks[channel] = if level.is_finite() && *level > next.peaks[channel] {
            *level
        } else {
            (next.peaks[channel] - PEAK_DECAY_DB * dt).max(FLOOR_DB)
        };
        if is_clipped(*level) {
            next.clip[channel] = true;
        }
    }
    next
}

/// Draw the stereo meter. `levels_db` are dBFS values (0 is full scale).
/// Clicking anywhere on the meter resets peaks and clip latches.
pub fn meter(ui: &mut Ui, id: egui::Id, levels_db: [f64; 2]) -> Response {
    let height = 2.0 * ROW_HEIGHT + ROW_GAP;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());

    let state = ui
        .ctx()
        .data(|data| data.get_temp::<MeterState>(id))
        .unwrap_or_default();
    let dt = ui.input(|input| input.unstable_dt).clamp(0.0, 0.1) as f64;
    let state = if response.clicked() {
        MeterState {
            peaks: levels_db,
            clip: [false; 2],
        }
    } else {
        advance_state(state, levels_db, dt)
    };
    ui.ctx().data_mut(|data| data.insert_temp(id, state));

    paint(ui.painter(), rect, levels_db, state);
    response
}

fn paint(painter: &Painter, rect: Rect, levels_db: [f64; 2], state: MeterState) {
    let labels = ["L", "R"];
    let track_left = rect.left() + LABEL_COLUMN;
    for (channel, label) in labels.iter().enumerate() {
        let row_top = rect.top() + channel as f32 * (ROW_HEIGHT + ROW_GAP);
        painter.text(
            Pos2::new(rect.left(), row_top + ROW_HEIGHT / 2.0),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(9.0),
            palette::TEXT_MUTED,
        );
        let track = Rect::from_min_size(
            Pos2::new(track_left, row_top),
            Vec2::new(rect.right() - track_left, ROW_HEIGHT),
        );
        surface::inset(painter, track);

        let level = if levels_db[channel].is_finite() {
            db_to_fraction(levels_db[channel])
        } else {
            0.0
        };
        let fill = Rect::from_min_size(
            track.left_top(),
            Vec2::new(track.width() * level as f32, track.height()),
        );
        gradient_fill(painter, fill, track.height());

        let peak = db_to_fraction(state.peaks[channel]) as f32;
        if peak > 0.0 {
            let x = track.left() + track.width() * peak;
            painter.line_segment(
                [Pos2::new(x, track.top()), Pos2::new(x, track.bottom())],
                Stroke::new(1.0_f32, level_color(state.peaks[channel])),
            );
        }
        if state.clip[channel] {
            painter.circle_filled(
                Pos2::new(track.right() - 5.0, track.center().y),
                2.5,
                palette::RED,
            );
        }
    }
}

/// Horizontal cyan → yellow → red gradient across the fill, as a vertex-colored
/// mesh with stops at 0, 0.75, and 1 of the full-track loudness scale.
fn gradient_fill(painter: &Painter, fill: Rect, height: f32) {
    if !fill.is_positive() || fill.width() <= 0.5 || height <= 0.0 {
        return;
    }
    let stops = [
        (0.0_f32, palette::CYAN),
        (0.75, palette::YELLOW),
        (1.0, palette::RED),
    ];
    let mut mesh = Mesh::default();
    let uv = egui::epaint::WHITE_UV;
    for pair in stops.windows(2) {
        let (t0, left) = pair[0];
        let (t1, right) = pair[1];
        let x0 = fill.left() + fill.width() * t0;
        let x1 = (fill.left() + fill.width() * t1).min(fill.right());
        if x1 <= x0 {
            continue;
        }
        let base = mesh.vertices.len() as u32;
        mesh.vertices.extend_from_slice(&[
            egui::epaint::Vertex {
                pos: Pos2::new(x0, fill.top()),
                uv,
                color: left,
            },
            egui::epaint::Vertex {
                pos: Pos2::new(x1, fill.top()),
                uv,
                color: right,
            },
            egui::epaint::Vertex {
                pos: Pos2::new(x1, fill.bottom()),
                uv,
                color: right,
            },
            egui::epaint::Vertex {
                pos: Pos2::new(x0, fill.bottom()),
                uv,
                color: left,
            },
        ]);
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    painter.add(Shape::Mesh(std::sync::Arc::new(mesh)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_to_fraction_maps_the_meter_range() {
        assert_eq!(db_to_fraction(FLOOR_DB), 0.0);
        assert!((db_to_fraction(-30.0) - 0.5).abs() < 1e-9);
        assert_eq!(db_to_fraction(0.0), 1.0);
        assert_eq!(db_to_fraction(-100.0), 0.0);
        assert_eq!(db_to_fraction(3.0), 1.0);
        assert_eq!(db_to_fraction(f64::NAN), 0.0);
    }

    #[test]
    fn level_color_moves_cyan_to_yellow_to_red() {
        assert_eq!(level_color(-40.0), palette::CYAN);
        assert_eq!(level_color(-18.0), palette::CYAN);
        let near = level_color(-10.0);
        assert!(near.g() > level_color(-18.0).g() || near.r() > level_color(-18.0).r());
        assert_eq!(level_color(0.0), palette::RED);
        let almost = level_color(-3.0);
        assert!(almost.r() > almost.b());
    }

    #[test]
    fn clip_latches_at_the_threshold() {
        assert!(!is_clipped(-0.2));
        assert!(is_clipped(-0.1));
        assert!(is_clipped(0.0));
        assert!(is_clipped(1.0));
    }

    #[test]
    fn peak_holds_then_decays_and_clip_latches() {
        let state = advance_state(MeterState::default(), [-6.0, -20.0], 0.016);
        assert!((state.peaks[0] - -6.0).abs() < 1e-9);
        assert!(!state.clip[0]);

        let latched = advance_state(state, [-30.0, -30.0], 0.016);
        assert!(latched.peaks[0] < -6.0);
        assert!(!latched.clip[0]);

        let clipped = advance_state(latched, [-30.0, 0.0], 0.016);
        assert!(clipped.clip[1]);
        assert!(clipped.peaks[1] >= 0.0);
    }

    #[test]
    fn non_finite_levels_hold_silence() {
        let state = advance_state(MeterState::default(), [f64::NAN, f64::NEG_INFINITY], 0.1);
        assert_eq!(state.peaks[0], FLOOR_DB);
        assert_eq!(state.peaks[1], FLOOR_DB);
        assert!(!state.clip[0] && !state.clip[1]);
    }
}
