//! Linear param glide, advanced per sample without touching atomics.

/// Ramps from the current value to a target over a fixed time. Modules set
/// the target once per block from the param snapshot and call [`next`]
/// per sample.
///
/// [`next`]: Smoother::next
#[derive(Clone, Copy, Debug)]
pub struct Smoother {
    current: f32,
    target: f32,
    step: f32,
    remaining: u32,
    ramp_samples: u32,
}

impl Smoother {
    /// A smoother resting at `value` with no ramp time yet.
    #[must_use]
    pub const fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            step: 0.0,
            remaining: 0,
            ramp_samples: 0,
        }
    }

    /// Sets the ramp time from a sample rate and milliseconds.
    pub fn set_time(&mut self, sample_rate: f32, ms: f32) {
        let samples = (sample_rate.max(0.0) * ms.max(0.0) / 1000.0).round();
        self.ramp_samples = if samples.is_finite() {
            samples as u32
        } else {
            0
        };
    }

    /// Starts a ramp toward `target`; a repeated target keeps the ramp going.
    pub fn set_target(&mut self, target: f32) {
        if !target.is_finite() || target == self.target {
            return;
        }
        self.target = target;
        if self.ramp_samples == 0 {
            self.snap(target);
            return;
        }
        self.remaining = self.ramp_samples;
        self.step = (target - self.current) / self.ramp_samples as f32;
    }

    /// Jumps straight to `value`.
    pub fn snap(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.remaining = 0;
        self.step = 0.0;
    }

    /// The next sample's value.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f32 {
        if self.remaining > 0 {
            self.remaining -= 1;
            self.current = if self.remaining == 0 {
                self.target
            } else {
                self.current + self.step
            };
        }
        self.current
    }

    /// The current value without advancing.
    #[must_use]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// True while a ramp is running.
    #[must_use]
    pub fn is_smoothing(&self) -> bool {
        self.remaining > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_reaches_target_exactly() {
        let mut smoother = Smoother::new(0.0);
        smoother.set_time(1000.0, 10.0);
        smoother.set_target(1.0);
        let values: Vec<f32> = (0..10).map(|_| smoother.next()).collect();
        assert!((values[0] - 0.1).abs() < 1e-6);
        assert_eq!(values[9], 1.0);
        assert!(!smoother.is_smoothing());
        assert_eq!(smoother.next(), 1.0);
    }

    #[test]
    fn zero_time_snaps() {
        let mut smoother = Smoother::new(0.0);
        smoother.set_target(0.5);
        assert_eq!(smoother.next(), 0.5);
    }

    #[test]
    fn non_finite_targets_are_ignored() {
        let mut smoother = Smoother::new(0.25);
        smoother.set_target(f32::NAN);
        assert_eq!(smoother.next(), 0.25);
    }
}
