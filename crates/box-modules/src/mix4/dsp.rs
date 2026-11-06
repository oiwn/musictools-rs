//! Mix4 DSP.

use std::f32::consts::FRAC_PI_2;

use box_core::{BlockIo, Meters, Module, ParamValues, Smoother};

use super::{CHANNELS, PARAMS, level, pan};

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut mix = Mix4::default();
    mix.reset(sample_rate, max_block);
    Box::new(mix)
}

/// Constant-power gains `(left, right)` for a pan of −1..=1.
#[must_use]
pub fn pan_gains(pan: f32) -> (f32, f32) {
    let angle = (pan.clamp(-1.0, 1.0) + 1.0) * 0.5 * FRAC_PI_2;
    (angle.cos(), angle.sin())
}

/// The mixer's DSP: smoothed level and pan per channel.
#[derive(Debug)]
pub struct Mix4 {
    levels: [Smoother; CHANNELS],
    pans: [Smoother; CHANNELS],
    primed: bool,
}

impl Default for Mix4 {
    fn default() -> Self {
        Self {
            levels: std::array::from_fn(|channel| {
                Smoother::new(PARAMS[level(channel)].default_value())
            }),
            pans: std::array::from_fn(|channel| {
                Smoother::new(PARAMS[pan(channel)].default_value())
            }),
            primed: false,
        }
    }
}

impl Module for Mix4 {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        for channel in 0..CHANNELS {
            self.levels[channel].set_time(sample_rate, PARAMS[level(channel)].smoothing_ms);
            self.pans[channel].set_time(sample_rate, PARAMS[pan(channel)].smoothing_ms);
        }
        self.clear();
    }

    fn clear(&mut self) {
        self.primed = false;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, _meters: &Meters) {
        for channel in 0..CHANNELS {
            let (level_value, pan_value) = (params.get(level(channel)), params.get(pan(channel)));
            if self.primed {
                self.levels[channel].set_target(level_value);
                self.pans[channel].set_target(pan_value);
            } else {
                self.levels[channel].snap(level_value);
                self.pans[channel].snap(pan_value);
            }
        }
        self.primed = true;
        let connected: [bool; CHANNELS] = std::array::from_fn(|channel| io.is_connected(channel));
        let inputs = io.inputs;
        let [left, right, ..] = &mut *io.outputs else {
            return;
        };
        left.fill(0.0);
        right.fill(0.0);
        for channel in 0..CHANNELS {
            let (levels, pans) = (&mut self.levels[channel], &mut self.pans[channel]);
            if !connected[channel] {
                // Keep the glides moving so a later wire starts in place.
                for _ in 0..io.len {
                    levels.next();
                    pans.next();
                }
                continue;
            }
            let input = inputs[channel];
            for index in 0..io.len {
                let gain = levels.next();
                let (to_left, to_right) = pan_gains(pans.next());
                let sample = input[index] * gain;
                left[index] += sample * to_left;
                right[index] += sample * to_right;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mix4::ENTRY;
    use crate::tests::{instance, run_block};

    #[test]
    fn pan_law_is_constant_power() {
        let (l, r) = pan_gains(0.0);
        assert!((l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((r - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert_eq!(pan_gains(-1.0).0, 1.0);
        assert!(pan_gains(-1.0).1.abs() < 1e-6);
        assert!(pan_gains(1.0).0.abs() < 1e-6);
        for step in -10..=10 {
            let (l, r) = pan_gains(step as f32 / 10.0);
            assert!((l * l + r * r - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn four_channels_sum_and_unconnected_ones_add_nothing() {
        let (mut module, shared) = instance(&ENTRY);
        for channel in 0..CHANNELS {
            shared.params.set(level(channel), 1.0);
        }
        shared.params.set(pan(0), -1.0);
        shared.params.set(pan(1), 1.0);
        let ones = vec![1.0; 64];
        let halves = vec![0.5; 64];
        let loud = vec![9.0; 64];
        // Channels 1 and 2 wired; 3 holds junk but is unconnected; 4 is silent.
        let out = run_block(
            module.as_mut(),
            &shared,
            &[&ones, &halves, &loud, &halves],
            0b1011,
            2,
            64,
        );
        let center = 0.5 * std::f32::consts::FRAC_1_SQRT_2;
        assert!((out[0][63] - (1.0 + center)).abs() < 1e-5, "{}", out[0][63]);
        assert!((out[1][63] - (0.5 + center)).abs() < 1e-5, "{}", out[1][63]);
    }

    #[test]
    fn hot_inputs_stay_finite() {
        let (mut module, shared) = instance(&ENTRY);
        let hot = vec![1.0e30; 32];
        let out = run_block(
            module.as_mut(),
            &shared,
            &[&hot, &hot, &hot, &hot],
            0b1111,
            2,
            32,
        );
        assert!(out.iter().flatten().all(|sample| sample.is_finite()));
    }
}
