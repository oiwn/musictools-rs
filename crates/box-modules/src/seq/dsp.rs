//! Seq DSP.

use box_core::pitch::semitones;
use box_core::{BlockIo, Meters, Module, ParamValues};

use super::{CLOCK, RESET, STEP_METER, STEPS, gate, pitch};

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut seq = Seq::default();
    seq.reset(sample_rate, max_block);
    Box::new(seq)
}

/// The sequencer's DSP.
#[derive(Debug, Default)]
pub struct Seq {
    step: usize,
    /// False until the first clock (and again after a reset), so that
    /// clock plays step 1 instead of moving past it.
    started: bool,
    clock_high: bool,
    reset_high: bool,
}

impl Module for Seq {
    fn reset(&mut self, _sample_rate: f32, _max_block: usize) {
        self.clear();
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        let pitches: [f32; STEPS] =
            std::array::from_fn(|step| semitones(params.get(pitch(step)).round()));
        let gates: [bool; STEPS] = std::array::from_fn(|step| params.bool(gate(step)));
        let (clock, reset) = (io.inputs[CLOCK], io.inputs[RESET]);
        let [pitch_out, gate_out, ..] = &mut *io.outputs else {
            return;
        };
        for index in 0..io.len {
            let reset_high = reset[index] >= 0.5;
            if reset_high && !self.reset_high {
                self.started = false;
            }
            self.reset_high = reset_high;

            let clock_high = clock[index] >= 0.5;
            if clock_high && !self.clock_high {
                if self.started {
                    self.step = (self.step + 1) % STEPS;
                } else {
                    self.step = 0;
                    self.started = true;
                }
            }
            self.clock_high = clock_high;

            pitch_out[index] = pitches[self.step];
            let open = self.started && clock_high && gates[self.step];
            gate_out[index] = if open { 1.0 } else { 0.0 };
        }
        let shown = if self.started { self.step as f32 } else { -1.0 };
        meters.set(STEP_METER, shown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seq::ENTRY;
    use crate::tests::{instance, run_block};

    /// A clock of `pulses` 10-sample-high, 10-sample-low periods.
    fn clock(pulses: usize) -> Vec<f32> {
        (0..pulses * 20)
            .map(|index| if index % 20 < 10 { 1.0 } else { 0.0 })
            .collect()
    }

    /// Pitch (in semitones) and gate-high count per clock period.
    fn per_period(out: &[Vec<f32>]) -> Vec<(f32, usize)> {
        out[0]
            .chunks(20)
            .zip(out[1].chunks(20))
            .map(|(pitch, gate)| {
                let high = gate.iter().filter(|&&sample| sample == 1.0).count();
                (pitch[0] * 12.0, high)
            })
            .collect()
    }

    #[test]
    fn steps_advance_and_wrap_with_exact_pitches() {
        let (mut module, shared) = instance(&ENTRY);
        for step in 0..STEPS {
            shared.params.set(pitch(step), step as f32 * 2.0 - 3.0);
        }
        let clock = clock(10);
        let silence = vec![0.0; clock.len()];
        let out = run_block(
            module.as_mut(),
            &shared,
            &[&clock, &silence],
            0b01,
            2,
            clock.len(),
        );
        let periods = per_period(&out);
        let expected: Vec<f32> = (0..10).map(|n| (n % STEPS) as f32 * 2.0 - 3.0).collect();
        for (period, (pitch, _)) in periods.iter().enumerate() {
            assert!(
                (pitch - expected[period]).abs() < 1e-4,
                "period {period}: {pitch}"
            );
        }
    }

    #[test]
    fn gates_follow_the_clock_and_skip_off_steps() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(gate(1), 0.0);
        let clock = clock(3);
        let silence = vec![0.0; clock.len()];
        let out = run_block(
            module.as_mut(),
            &shared,
            &[&clock, &silence],
            0b01,
            2,
            clock.len(),
        );
        let highs: Vec<usize> = per_period(&out).iter().map(|(_, high)| *high).collect();
        assert_eq!(highs, [10, 0, 10]);
    }

    #[test]
    fn reset_returns_to_step_one_on_the_next_clock() {
        let (mut module, shared) = instance(&ENTRY);
        for step in 0..STEPS {
            shared.params.set(pitch(step), step as f32);
        }
        let clock = clock(6);
        let mut reset = vec![0.0; clock.len()];
        // A reset pulse between the 3rd and 4th clocks.
        for sample in &mut reset[50..55] {
            *sample = 1.0;
        }
        let out = run_block(
            module.as_mut(),
            &shared,
            &[&clock, &reset],
            0b11,
            2,
            clock.len(),
        );
        let pitches: Vec<f32> = per_period(&out)
            .iter()
            .map(|(pitch, _)| pitch.round())
            .collect();
        assert_eq!(pitches, [0.0, 1.0, 2.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn pitches_snap_to_whole_semitones() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(pitch(0), 6.6);
        let clock = clock(1);
        let silence = vec![0.0; clock.len()];
        let out = run_block(module.as_mut(), &shared, &[&clock, &silence], 0b01, 2, 20);
        assert!((out[0][0] - 7.0 / 12.0).abs() < 1e-6, "{}", out[0][0]);
    }
}
