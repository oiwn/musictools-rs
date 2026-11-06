//! Sine DSP.

use std::f32::consts::TAU;

use box_core::{BlockIo, Meters, Module, ParamValues, Smoother};

use super::{FREQ, LEVEL, PARAMS, PITCH};

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut sine = Sine::default();
    sine.reset(sample_rate, max_block);
    Box::new(sine)
}

/// The oscillator's DSP.
#[derive(Debug)]
pub struct Sine {
    phase: f32,
    inverse_rate: f32,
    nyquist: f32,
    freq: Smoother,
    level: Smoother,
    primed: bool,
}

impl Default for Sine {
    fn default() -> Self {
        Self {
            phase: 0.0,
            inverse_rate: 1.0 / 48_000.0,
            nyquist: 24_000.0,
            freq: Smoother::new(PARAMS[FREQ].default_value()),
            level: Smoother::new(PARAMS[LEVEL].default_value()),
            primed: false,
        }
    }
}

impl Module for Sine {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        let rate = sample_rate.max(1.0);
        self.inverse_rate = 1.0 / rate;
        self.nyquist = rate / 2.0;
        self.freq.set_time(rate, PARAMS[FREQ].smoothing_ms);
        self.level.set_time(rate, PARAMS[LEVEL].smoothing_ms);
        self.clear();
    }

    fn clear(&mut self) {
        self.phase = 0.0;
        self.primed = false;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, _meters: &Meters) {
        if self.primed {
            self.freq.set_target(params.get(FREQ));
            self.level.set_target(params.get(LEVEL));
        } else {
            // Start at the current settings instead of gliding from defaults.
            self.freq.snap(params.get(FREQ));
            self.level.snap(params.get(LEVEL));
            self.primed = true;
        }
        let pitched = io.is_connected(PITCH);
        let pitch = io.inputs[PITCH];
        for (index, out) in io.outputs[0].iter_mut().enumerate() {
            *out = (TAU * self.phase).sin() * self.level.next();
            let mut freq = self.freq.next();
            if pitched {
                freq = (freq * pitch[index].clamp(-10.0, 10.0).exp2()).min(self.nyquist);
            }
            self.phase += freq * self.inverse_rate;
            self.phase -= self.phase.floor();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sine::ENTRY;
    use crate::tests::{instance, run_block};

    fn rising_crossings(pitch: Option<f32>, freq: f32, seconds: usize) -> (usize, f32) {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(FREQ, freq);
        shared.params.set(LEVEL, 0.25);
        let input = vec![pitch.unwrap_or(0.0); 4000];
        let connected = u32::from(pitch.is_some());
        let mut rising = 0;
        let mut previous = 0.0_f32;
        let mut peak = 0.0_f32;
        for _ in 0..12 * seconds {
            let out = run_block(module.as_mut(), &shared, &[&input], connected, 1, 4000).remove(0);
            for sample in out {
                if previous < 0.0 && sample >= 0.0 {
                    rising += 1;
                }
                previous = sample;
                peak = peak.max(sample.abs());
            }
        }
        (rising, peak)
    }

    #[test]
    fn frequency_and_level_are_exact() {
        // One second at 48 kHz: 440 rising zero crossings (the first cycle
        // starts at zero and doesn't count).
        let (rising, peak) = rising_crossings(None, 440.0, 1);
        assert!((439..=440).contains(&rising), "{rising} crossings");
        assert!((peak - 0.25).abs() < 1e-3, "peak {peak}");
    }

    #[test]
    fn pitch_one_plays_an_octave_up() {
        let (rising, _) = rising_crossings(Some(1.0), 220.0, 1);
        assert!((439..=440).contains(&rising), "{rising} crossings");
        let (rising, _) = rising_crossings(Some(-1.0), 440.0, 1);
        assert!((219..=220).contains(&rising), "{rising} crossings");
    }

    #[test]
    fn default_freq_is_c4() {
        assert!((PARAMS[FREQ].default_value() - 261.625_58).abs() < 1e-3);
    }

    #[test]
    fn clear_restarts_the_phase() {
        let (mut module, shared) = instance(&ENTRY);
        let silence = [0.0; 100];
        run_block(module.as_mut(), &shared, &[&silence], 0, 1, 100);
        module.clear();
        let out = run_block(module.as_mut(), &shared, &[&silence[..1]], 0, 1, 1).remove(0);
        assert_eq!(out[0], 0.0);
    }
}
