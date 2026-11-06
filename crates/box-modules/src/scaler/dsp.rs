//! Scaler DSP: per-sample quantization, recomputed only when the input or
//! settings move.

use box_core::{BlockIo, Meters, Module, ParamValues};

use super::scales::{SCALES, quantize};
use super::{PITCH_IN, ROOT, SCALE};

/// Meter: the latest output's cents above the root, within the octave,
/// for the tile's strip.
pub(super) const DEGREE_METER: usize = 0;

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut scaler = Scaler::default();
    scaler.reset(sample_rate, max_block);
    Box::new(scaler)
}

/// The quantizer's DSP.
#[derive(Debug)]
pub struct Scaler {
    last_in: f32,
    last_out: f32,
    last_settings: (usize, usize),
}

impl Default for Scaler {
    fn default() -> Self {
        Self {
            last_in: f32::NAN,
            last_out: 0.0,
            last_settings: (usize::MAX, usize::MAX),
        }
    }
}

impl Module for Scaler {
    fn reset(&mut self, _sample_rate: f32, _max_block: usize) {
        self.clear();
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        let root = params.choice(ROOT).min(11);
        let scale_index = params.choice(SCALE).min(SCALES.len() - 1);
        if (root, scale_index) != self.last_settings {
            self.last_settings = (root, scale_index);
            self.last_in = f32::NAN;
        }
        let scale = &SCALES[scale_index];
        let input = io.inputs[PITCH_IN];
        for (out, &pitch) in io.outputs[0].iter_mut().zip(input) {
            // A held note costs one compare (NaN never equals, so the
            // first sample after a settings change recomputes).
            if pitch != self.last_in {
                self.last_in = pitch;
                self.last_out = quantize(pitch, root as f32, scale);
            }
            *out = self.last_out;
        }
        let above_root = (self.last_out * 1200.0 - root as f32 * 100.0).rem_euclid(1200.0);
        meters.set(DEGREE_METER, above_root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaler::ENTRY;
    use crate::tests::{instance, run_block};

    #[test]
    fn a_ramp_lands_only_on_scale_degrees() {
        let (mut module, shared) = instance(&ENTRY);
        // C major.
        let ramp: Vec<f32> = (0..1200).map(|index| index as f32 / 1200.0).collect();
        let out = run_block(module.as_mut(), &shared, &[&ramp], 1, 1, ramp.len()).remove(0);
        let allowed = [0, 2, 4, 5, 7, 9, 11, 12];
        for value in out {
            let semis = value * 12.0;
            assert!((semis - semis.round()).abs() < 1e-4, "{semis}");
            assert!(allowed.contains(&(semis.round() as i32)), "{semis}");
        }
    }

    #[test]
    fn changing_scale_requantizes_a_held_note() {
        let (mut module, shared) = instance(&ENTRY);
        // 60 cents above C: C in C major, C# in the chromatic scale.
        let held = vec![0.6 / 12.0; 16];
        let first = run_block(module.as_mut(), &shared, &[&held], 1, 1, 16).remove(0);
        assert!(first[15].abs() < 1e-6, "{}", first[15]);
        let chromatic = super::super::scales::NAMES
            .iter()
            .position(|name| *name == "Chromatic")
            .unwrap();
        shared.params.set(SCALE, chromatic as f32);
        let second = run_block(module.as_mut(), &shared, &[&held], 1, 1, 16).remove(0);
        assert!((second[0] - 1.0 / 12.0).abs() < 1e-6);
    }
}
