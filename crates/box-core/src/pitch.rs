//! The pitch signal convention: a pitch port carries octaves, with 0.0 at
//! C4 (middle C, 261.63 Hz) and +1.0 one octave up, like V/oct. Sequencers
//! output it, scalers quantize it, oscillators turn it into frequency.

/// Frequency of C4, the pitch origin.
pub const C4_HZ: f32 = 261.625_58;

/// Frequency of `pitch` octaves above `base_hz`.
#[must_use]
pub fn pitch_to_hz(base_hz: f32, pitch: f32) -> f32 {
    base_hz * pitch.exp2()
}

/// Pitch (octaves) of `hz` relative to `base_hz`.
#[must_use]
pub fn hz_to_pitch(base_hz: f32, hz: f32) -> f32 {
    (hz / base_hz).log2()
}

/// Pitch of `count` semitones.
#[must_use]
pub fn semitones(count: f32) -> f32 {
    count / 12.0
}

/// Pitch of `count` cents.
#[must_use]
pub fn cents(count: f32) -> f32 {
    count / 1200.0
}

/// Semitones in `pitch`.
#[must_use]
pub fn to_semitones(pitch: f32) -> f32 {
    pitch * 12.0
}

/// Cents in `pitch`.
#[must_use]
pub fn to_cents(pitch: f32) -> f32 {
    pitch * 1200.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_is_440_hz() {
        let a4 = pitch_to_hz(C4_HZ, semitones(9.0));
        assert!((a4 - 440.0).abs() < 1e-2, "{a4}");
    }

    #[test]
    fn conversions_invert() {
        for pitch in [-2.0_f32, -0.25, 0.0, 0.5, 1.0, 3.0] {
            let back = hz_to_pitch(C4_HZ, pitch_to_hz(C4_HZ, pitch));
            assert!((back - pitch).abs() < 1e-5, "{pitch} -> {back}");
            assert!((semitones(to_semitones(pitch)) - pitch).abs() < 1e-6);
            assert!((cents(to_cents(pitch)) - pitch).abs() < 1e-6);
        }
        assert_eq!(pitch_to_hz(100.0, 1.0), 200.0);
    }
}
