//! roman-dsp — Roman's biquad filter, free of any plugin framework.
//!
//! [`RomanFilter`] is one channel: lowpass, highpass, bandpass, notch or
//! allpass at 12 dB/oct (one biquad) or 24 dB/oct (two in series).
//! Coefficients are recomputed only when the settings change.
//! [`StereoRoman`] runs two channels from the same settings. Used by the
//! box Filter module now and the `ppc_roman` plugin later.

pub mod biquad;

use biquad::{BiquadCoefficients, BiquadFilter, NEUTRAL_Q};
use musictools_core::finite_or;

/// Filter response.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterType {
    #[default]
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Allpass,
}

impl FilterType {
    /// The type for a choice index (Lowpass, Highpass, Bandpass, Notch,
    /// Allpass); out-of-range indices clamp to the last.
    #[must_use]
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Lowpass,
            1 => Self::Highpass,
            2 => Self::Bandpass,
            3 => Self::Notch,
            _ => Self::Allpass,
        }
    }
}

/// Steepness: one or two cascaded biquads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Slope {
    #[default]
    Db12,
    Db24,
}

/// Everything that sets the coefficients.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RomanSettings {
    pub filter_type: FilterType,
    pub cutoff_hz: f32,
    pub q: f32,
    pub slope: Slope,
    pub sample_rate: f32,
}

impl Default for RomanSettings {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Lowpass,
            cutoff_hz: 1000.0,
            q: NEUTRAL_Q,
            slope: Slope::Db12,
            sample_rate: 48_000.0,
        }
    }
}

impl RomanSettings {
    /// The settings with non-finite fields replaced by defaults.
    #[must_use]
    pub fn sanitized(self) -> Self {
        let fallback = Self::default();
        Self {
            cutoff_hz: finite_or(self.cutoff_hz, fallback.cutoff_hz),
            q: finite_or(self.q, fallback.q),
            sample_rate: finite_or(self.sample_rate, fallback.sample_rate),
            ..self
        }
    }

    /// One section's coefficients for these settings.
    #[must_use]
    pub fn coefficients(&self) -> BiquadCoefficients {
        let (rate, cutoff, q) = (self.sample_rate, self.cutoff_hz, self.q);
        match self.filter_type {
            FilterType::Lowpass => BiquadCoefficients::lowpass(rate, cutoff, q),
            FilterType::Highpass => BiquadCoefficients::highpass(rate, cutoff, q),
            FilterType::Bandpass => BiquadCoefficients::bandpass(rate, cutoff, q),
            FilterType::Notch => BiquadCoefficients::notch(rate, cutoff, q),
            FilterType::Allpass => BiquadCoefficients::allpass(rate, cutoff, q),
        }
    }

    /// Sections in series for the slope.
    #[must_use]
    pub fn stages(&self) -> usize {
        match self.slope {
            Slope::Db12 => 1,
            Slope::Db24 => 2,
        }
    }
}

/// One channel of the filter. Starts as an identity (passthrough) until
/// the first [`RomanFilter::set`].
#[derive(Clone, Debug, Default)]
pub struct RomanFilter {
    settings: Option<RomanSettings>,
    stages: [BiquadFilter; 2],
    active: usize,
    updates: u64,
}

impl RomanFilter {
    /// Applies `settings`, recomputing coefficients only when they changed.
    pub fn set(&mut self, settings: RomanSettings) {
        let settings = settings.sanitized();
        if self.settings == Some(settings) {
            return;
        }
        let coefficients = settings.coefficients();
        for stage in &mut self.stages {
            stage.coefficients = coefficients;
        }
        // A newly added second stage starts from silence.
        if settings.stages() > self.active {
            self.stages[1].reset();
        }
        self.active = settings.stages();
        self.settings = Some(settings);
        self.updates += 1;
    }

    /// Filters one sample; non-finite input is treated as silence.
    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        let mut value = finite_or(sample, 0.0);
        if self.settings.is_none() {
            return value;
        }
        for stage in &mut self.stages[..self.active] {
            value = stage.process(value);
        }
        finite_or(value, 0.0)
    }

    /// Clears the filter memory, keeping the settings.
    pub fn reset(&mut self) {
        for stage in &mut self.stages {
            stage.reset();
        }
    }

    /// How many times the coefficients were recomputed.
    #[must_use]
    pub fn coefficient_updates(&self) -> u64 {
        self.updates
    }

    /// The current settings, if any were set.
    #[must_use]
    pub fn settings(&self) -> Option<RomanSettings> {
        self.settings
    }
}

/// One stereo sample.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Frame {
    pub left: f32,
    pub right: f32,
}

/// Two independent channels sharing one set of settings.
#[derive(Clone, Debug, Default)]
pub struct StereoRoman {
    left: RomanFilter,
    right: RomanFilter,
}

impl StereoRoman {
    /// Applies `settings` to both channels.
    pub fn set(&mut self, settings: RomanSettings) {
        self.left.set(settings);
        self.right.set(settings);
    }

    /// Filters one stereo frame.
    #[inline]
    pub fn process(&mut self, frame: Frame) -> Frame {
        Frame {
            left: self.left.process(frame.left),
            right: self.right.process(frame.right),
        }
    }

    /// Clears both channels' memory.
    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    const RATE: f32 = 48_000.0;

    fn settings(filter_type: FilterType, cutoff_hz: f32, slope: Slope) -> RomanSettings {
        RomanSettings {
            filter_type,
            cutoff_hz,
            q: NEUTRAL_Q,
            slope,
            sample_rate: RATE,
        }
    }

    /// Steady-state RMS gain of `filter` for a sine at `freq`.
    fn gain(filter: &mut RomanFilter, freq: f32) -> f32 {
        filter.reset();
        let samples = 48_000;
        let mut energy_in = 0.0_f64;
        let mut energy_out = 0.0_f64;
        for index in 0..samples {
            let x = (TAU * freq * index as f32 / RATE).sin();
            let y = filter.process(x);
            if index > samples / 2 {
                energy_in += f64::from(x * x);
                energy_out += f64::from(y * y);
            }
        }
        (energy_out / energy_in).sqrt() as f32
    }

    #[test]
    fn identity_passes_through_until_set() {
        let mut filter = RomanFilter::default();
        for x in [0.5, -1.0, 0.25] {
            assert_eq!(filter.process(x), x);
        }
    }

    #[test]
    fn lowpass_passes_low_and_cuts_high() {
        let mut filter = RomanFilter::default();
        filter.set(settings(FilterType::Lowpass, 1000.0, Slope::Db12));
        assert!(gain(&mut filter, 100.0) > 0.98);
        assert!(gain(&mut filter, 10_000.0) < 0.02);
    }

    #[test]
    fn highpass_mirrors_it() {
        let mut filter = RomanFilter::default();
        filter.set(settings(FilterType::Highpass, 1000.0, Slope::Db12));
        assert!(gain(&mut filter, 10_000.0) > 0.98);
        assert!(gain(&mut filter, 100.0) < 0.02);
    }

    #[test]
    fn bandpass_and_notch_act_on_their_band() {
        let mut bandpass = RomanFilter::default();
        bandpass.set(RomanSettings {
            q: 4.0,
            ..settings(FilterType::Bandpass, 1000.0, Slope::Db12)
        });
        assert!((gain(&mut bandpass, 1000.0) - 1.0).abs() < 0.02);
        assert!(gain(&mut bandpass, 4000.0) < 0.15);
        let mut notch = RomanFilter::default();
        notch.set(settings(FilterType::Notch, 1000.0, Slope::Db12));
        assert!(gain(&mut notch, 1000.0) < 0.02);
        assert!(gain(&mut notch, 8000.0) > 0.9);
    }

    #[test]
    fn the_24_db_slope_cuts_twice_as_hard() {
        let mut gentle = RomanFilter::default();
        gentle.set(settings(FilterType::Lowpass, 1000.0, Slope::Db12));
        let mut steep = RomanFilter::default();
        steep.set(settings(FilterType::Lowpass, 1000.0, Slope::Db24));
        let (g12, g24) = (gain(&mut gentle, 8000.0), gain(&mut steep, 8000.0));
        assert!((g24 - g12 * g12).abs() < 1e-3, "{g12} {g24}");
    }

    #[test]
    fn non_finite_input_stays_finite() {
        let mut filter = RomanFilter::default();
        filter.set(settings(FilterType::Highpass, 200.0, Slope::Db24));
        for x in [f32::NAN, f32::INFINITY, 1.0, f32::NEG_INFINITY] {
            assert!(filter.process(x).is_finite());
        }
        filter.set(RomanSettings {
            cutoff_hz: f32::NAN,
            q: f32::INFINITY,
            ..settings(FilterType::Bandpass, 1.0, Slope::Db12)
        });
        assert!(filter.process(1.0).is_finite());
    }

    #[test]
    fn left_and_right_are_independent() {
        let mut stereo = StereoRoman::default();
        stereo.set(settings(FilterType::Lowpass, 500.0, Slope::Db12));
        let mut solo = RomanFilter::default();
        solo.set(settings(FilterType::Lowpass, 500.0, Slope::Db12));
        for index in 0..256 {
            let x = (index as f32 * 0.37).sin();
            let out = stereo.process(Frame {
                left: x,
                right: 0.0,
            });
            assert_eq!(out.left, solo.process(x));
            assert_eq!(out.right, 0.0);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut filter = RomanFilter::default();
        filter.set(settings(FilterType::Lowpass, 300.0, Slope::Db24));
        for _ in 0..64 {
            filter.process(1.0);
        }
        filter.reset();
        assert_eq!(filter.process(0.0), 0.0);
    }

    #[test]
    fn coefficients_recompute_only_on_change() {
        let mut filter = RomanFilter::default();
        let base = settings(FilterType::Lowpass, 1000.0, Slope::Db12);
        filter.set(base);
        filter.set(base);
        filter.set(base);
        assert_eq!(filter.coefficient_updates(), 1);
        filter.set(RomanSettings {
            cutoff_hz: 1200.0,
            ..base
        });
        filter.set(RomanSettings {
            slope: Slope::Db24,
            cutoff_hz: 1200.0,
            ..base
        });
        assert_eq!(filter.coefficient_updates(), 3);
    }

    #[cfg(feature = "rt-audit")]
    #[test]
    fn process_and_set_allocate_nothing() {
        let mut filter = StereoRoman::default();
        let (_, allocations) = musictools_core::rt_audit::audit(|| {
            for index in 0..4096 {
                if index % 256 == 0 {
                    filter.set(settings(
                        FilterType::from_index(index / 256),
                        200.0 + index as f32,
                        Slope::Db24,
                    ));
                }
                let x = (index as f32 * 0.01).sin();
                filter.process(Frame { left: x, right: x });
            }
        });
        assert_eq!(allocations, 0);
    }
}
