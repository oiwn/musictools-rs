//! The RBJ cookbook biquad.
//!
//! <http://shepazu.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html>

use std::f32::consts::TAU;

/// Q of a maximally flat (Butterworth) second-order section.
pub const NEUTRAL_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// Lowest and highest Q the coefficient builders accept.
pub const Q_RANGE: (f32, f32) = (0.1, 40.0);

/// Normalized biquad coefficients (`a0` divided out).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

/// The cookbook's shared terms for a cutoff and Q.
struct Terms {
    cos: f32,
    alpha: f32,
}

impl Terms {
    /// Clamps the cutoff inside (1 Hz, 0.49 × rate) and Q into
    /// [`Q_RANGE`]; non-finite values fall back to safe ones.
    fn new(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let sample_rate = musictools_core::finite_or(sample_rate, 48_000.0).max(1.0);
        let frequency =
            musictools_core::finite_or(frequency, 1000.0).clamp(1.0, 0.49 * sample_rate);
        let q = musictools_core::finite_or(q, NEUTRAL_Q).clamp(Q_RANGE.0, Q_RANGE.1);
        let omega = TAU * frequency / sample_rate;
        Self {
            cos: omega.cos(),
            alpha: omega.sin() / (2.0 * q),
        }
    }
}

impl BiquadCoefficients {
    /// Passes the signal through unchanged.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    /// Divides everything by `a0`.
    fn normalized(b: [f32; 3], a: [f32; 3]) -> Self {
        let a0 = a[0];
        Self {
            b0: b[0] / a0,
            b1: b[1] / a0,
            b2: b[2] / a0,
            a1: a[1] / a0,
            a2: a[2] / a0,
        }
    }

    /// Second-order lowpass.
    #[must_use]
    pub fn lowpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let Terms { cos, alpha } = Terms::new(sample_rate, frequency, q);
        Self::normalized(
            [(1.0 - cos) / 2.0, 1.0 - cos, (1.0 - cos) / 2.0],
            [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
        )
    }

    /// Second-order highpass.
    #[must_use]
    pub fn highpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let Terms { cos, alpha } = Terms::new(sample_rate, frequency, q);
        Self::normalized(
            [(1.0 + cos) / 2.0, -(1.0 + cos), (1.0 + cos) / 2.0],
            [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
        )
    }

    /// Bandpass with 0 dB at the center frequency.
    #[must_use]
    pub fn bandpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let Terms { cos, alpha } = Terms::new(sample_rate, frequency, q);
        Self::normalized([alpha, 0.0, -alpha], [1.0 + alpha, -2.0 * cos, 1.0 - alpha])
    }

    /// Notch (band reject) at the center frequency.
    #[must_use]
    pub fn notch(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let Terms { cos, alpha } = Terms::new(sample_rate, frequency, q);
        Self::normalized(
            [1.0, -2.0 * cos, 1.0],
            [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
        )
    }

    /// Allpass: flat magnitude, phase turns around the frequency.
    #[must_use]
    pub fn allpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        let Terms { cos, alpha } = Terms::new(sample_rate, frequency, q);
        Self::normalized(
            [1.0 - alpha, -2.0 * cos, 1.0 + alpha],
            [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
        )
    }

    /// Magnitude response at `frequency` (linear gain), for drawing
    /// response curves off the audio thread.
    #[must_use]
    pub fn magnitude(&self, sample_rate: f32, frequency: f32) -> f32 {
        let omega = TAU * frequency / sample_rate;
        let (cos1, sin1) = (omega.cos(), omega.sin());
        let (cos2, sin2) = ((2.0 * omega).cos(), (2.0 * omega).sin());
        let num_re = self.b0 + self.b1 * cos1 + self.b2 * cos2;
        let num_im = -(self.b1 * sin1 + self.b2 * sin2);
        let den_re = 1.0 + self.a1 * cos1 + self.a2 * cos2;
        let den_im = -(self.a1 * sin1 + self.a2 * sin2);
        ((num_re * num_re + num_im * num_im) / (den_re * den_re + den_im * den_im)).sqrt()
    }
}

/// One biquad section in transposed direct form II.
#[derive(Clone, Copy, Debug)]
pub struct BiquadFilter {
    pub coefficients: BiquadCoefficients,
    s1: f32,
    s2: f32,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self {
            coefficients: BiquadCoefficients::identity(),
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl BiquadFilter {
    /// Filters one sample.
    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        let c = &self.coefficients;
        let result = c.b0 * sample + self.s1;
        self.s1 = c.b1 * sample - c.a1 * result + self.s2;
        self.s2 = c.b2 * sample - c.a2 * result;
        result
    }

    /// Clears the filter memory.
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_has_unit_magnitude() {
        let identity = BiquadCoefficients::identity();
        for freq in [20.0, 1000.0, 20_000.0] {
            assert!((identity.magnitude(48_000.0, freq) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn bandpass_peaks_at_zero_db_and_notch_cuts() {
        let bandpass = BiquadCoefficients::bandpass(48_000.0, 1000.0, 2.0);
        assert!((bandpass.magnitude(48_000.0, 1000.0) - 1.0).abs() < 1e-3);
        assert!(bandpass.magnitude(48_000.0, 100.0) < 0.1);
        let notch = BiquadCoefficients::notch(48_000.0, 1000.0, 2.0);
        assert!(notch.magnitude(48_000.0, 1000.0) < 1e-3);
        assert!((notch.magnitude(48_000.0, 100.0) - 1.0).abs() < 0.02);
    }

    #[test]
    fn allpass_is_flat() {
        let allpass = BiquadCoefficients::allpass(48_000.0, 2000.0, 0.7);
        for freq in [50.0, 500.0, 2000.0, 15_000.0] {
            assert!((allpass.magnitude(48_000.0, freq) - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn out_of_range_settings_are_clamped() {
        let wild = BiquadCoefficients::lowpass(48_000.0, 1.0e9, 0.0);
        let edge = BiquadCoefficients::lowpass(48_000.0, 0.49 * 48_000.0, Q_RANGE.0);
        assert_eq!(wild, edge);
        let nan = BiquadCoefficients::highpass(f32::NAN, f32::NAN, f32::NAN);
        assert!(nan.magnitude(48_000.0, 1000.0).is_finite());
    }
}
