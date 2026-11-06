//! Filter DSP: `roman_dsp::RomanFilter` with cutoff and Q glides and a
//! modulated cutoff, updated every [`CONTROL`] samples.

use box_core::{BlockIo, Meters, Module, ParamValues, Smoother};
use roman_dsp::{FilterType, RomanFilter, RomanSettings, Slope};

use super::{CUTOFF, CUTOFF_IN, IN, PARAMS, Q, SLOPE, TYPE};

/// Samples between coefficient updates while gliding or modulated.
const CONTROL: usize = 16;

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut filter = Filter::default();
    filter.reset(sample_rate, max_block);
    Box::new(filter)
}

/// The module's DSP.
#[derive(Debug)]
pub struct Filter {
    filter: RomanFilter,
    cutoff: Smoother,
    q: Smoother,
    sample_rate: f32,
    primed: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            filter: RomanFilter::default(),
            cutoff: Smoother::new(PARAMS[CUTOFF].default_value()),
            q: Smoother::new(PARAMS[Q].default_value()),
            sample_rate: 48_000.0,
            primed: false,
        }
    }
}

impl Module for Filter {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        self.sample_rate = sample_rate.max(1.0);
        self.cutoff
            .set_time(self.sample_rate, PARAMS[CUTOFF].smoothing_ms);
        self.q.set_time(self.sample_rate, PARAMS[Q].smoothing_ms);
        self.clear();
    }

    fn clear(&mut self) {
        self.filter.reset();
        self.primed = false;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, _meters: &Meters) {
        if self.primed {
            self.cutoff.set_target(params.get(CUTOFF));
            self.q.set_target(params.get(Q));
        } else {
            self.cutoff.snap(params.get(CUTOFF));
            self.q.snap(params.get(Q));
            self.primed = true;
        }
        let filter_type = FilterType::from_index(params.choice(TYPE).min(3));
        let slope = if params.bool(SLOPE) {
            Slope::Db24
        } else {
            Slope::Db12
        };
        let modulated = io.is_connected(CUTOFF_IN);
        let (input, modulation) = (io.inputs[IN], io.inputs[CUTOFF_IN]);
        let nyquist_guard = 0.49 * self.sample_rate;
        let output = &mut *io.outputs[0];
        let mut start = 0;
        while start < io.len {
            let end = (start + CONTROL).min(io.len);
            let mut cutoff = self.cutoff.current();
            if modulated {
                cutoff *= modulation[start].clamp(-10.0, 10.0).exp2();
            }
            self.filter.set(RomanSettings {
                filter_type,
                cutoff_hz: cutoff.clamp(20.0, nyquist_guard),
                q: self.q.current(),
                slope,
                sample_rate: self.sample_rate,
            });
            for index in start..end {
                output[index] = self.filter.process(input[index]);
                self.cutoff.next();
                self.q.next();
            }
            start = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::ENTRY;
    use crate::tests::{instance, run_block};
    use roman_dsp::biquad::NEUTRAL_Q;

    fn noise(len: usize) -> Vec<f32> {
        let mut state = 0x1234_5678_u32;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state as f32 / u32::MAX as f32) * 2.0 - 1.0
            })
            .collect()
    }

    fn direct(cutoff: f32, input: &[f32]) -> Vec<f32> {
        let mut reference = RomanFilter::default();
        reference.set(RomanSettings {
            filter_type: FilterType::Lowpass,
            cutoff_hz: cutoff,
            q: NEUTRAL_Q,
            slope: Slope::Db12,
            sample_rate: 48_000.0,
        });
        input.iter().map(|&x| reference.process(x)).collect()
    }

    fn render(cutoff_input: Option<f32>, input: &[f32]) -> Vec<f32> {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(CUTOFF, 1000.0);
        shared.params.set(Q, NEUTRAL_Q);
        let modulation = vec![cutoff_input.unwrap_or(0.0); input.len()];
        let connected = 0b01 | (u32::from(cutoff_input.is_some()) << 1);
        run_block(
            module.as_mut(),
            &shared,
            &[input, &modulation],
            connected,
            1,
            input.len(),
        )
        .remove(0)
    }

    #[test]
    fn matches_roman_dsp_for_the_same_settings() {
        let input = noise(2048);
        let out = render(None, &input);
        let expected = direct(1000.0, &input);
        for (a, b) in out.iter().zip(&expected) {
            assert!((a - b).abs() < 1e-5, "{a} vs {b}");
        }
    }

    #[test]
    fn a_cutoff_input_of_one_doubles_the_cutoff() {
        let input = noise(2048);
        let out = render(Some(1.0), &input);
        let expected = direct(2000.0, &input);
        for (a, b) in out.iter().zip(&expected) {
            assert!((a - b).abs() < 1e-5, "{a} vs {b}");
        }
    }

    #[test]
    fn stays_finite_at_extremes() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(Q, 12.0);
        shared.params.set(SLOPE, 1.0);
        for filter_type in 0..4 {
            shared.params.set(TYPE, filter_type as f32);
            let input = noise(4096);
            let wild: Vec<f32> = (0..4096)
                .map(|index| if index % 2 == 0 { 40.0 } else { -40.0 })
                .collect();
            let out = run_block(module.as_mut(), &shared, &[&input, &wild], 0b11, 1, 4096);
            assert!(out[0].iter().all(|sample| sample.is_finite()));
        }
    }
}
