//! Env DSP: linear segments, sample-accurate gate edges.

use box_core::{BlockIo, Meters, Module, ParamValues};

use super::{ATTACK, DECAY, GATE, IN, LEVEL_METER, RELEASE, SUSTAIN};

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut env = Env::default();
    env.reset(sample_rate, max_block);
    Box::new(env)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// The envelope's DSP.
#[derive(Debug)]
pub struct Env {
    stage: Stage,
    level: f32,
    gate: bool,
    samples_per_ms: f32,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            stage: Stage::Idle,
            level: 0.0,
            gate: false,
            samples_per_ms: 48.0,
        }
    }
}

impl Env {
    /// Per-sample step that covers `span` in `ms`.
    fn step(&self, span: f32, ms: f32) -> f32 {
        span / (ms * self.samples_per_ms).max(1.0)
    }
}

impl Module for Env {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        self.samples_per_ms = sample_rate.max(1.0) / 1000.0;
        self.clear();
    }

    fn clear(&mut self) {
        self.stage = Stage::Idle;
        self.level = 0.0;
        self.gate = false;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        let sustain = params.get(SUSTAIN);
        let attack = self.step(1.0, params.get(ATTACK));
        let decay = self.step(1.0 - sustain, params.get(DECAY));
        let release = self.step(1.0, params.get(RELEASE));
        let shape_input = io.is_connected(IN);
        let (input, gate) = (io.inputs[IN], io.inputs[GATE]);
        for (index, out) in io.outputs[0].iter_mut().enumerate() {
            let high = gate[index] >= 0.5;
            if high && !self.gate {
                self.stage = Stage::Attack;
            } else if !high && self.gate {
                self.stage = Stage::Release;
            }
            self.gate = high;
            match self.stage {
                Stage::Idle => self.level = 0.0,
                Stage::Attack => {
                    self.level += attack;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.stage = Stage::Decay;
                    }
                }
                Stage::Decay => {
                    self.level -= decay;
                    if self.level <= sustain {
                        self.level = sustain;
                        self.stage = Stage::Sustain;
                    }
                }
                Stage::Sustain => self.level = sustain,
                Stage::Release => {
                    self.level -= release;
                    if self.level <= 0.0 {
                        self.level = 0.0;
                        self.stage = Stage::Idle;
                    }
                }
            }
            *out = if shape_input {
                input[index] * self.level
            } else {
                self.level
            };
        }
        meters.set(LEVEL_METER, self.level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::ENTRY;
    use crate::tests::{instance, run_block};

    fn render(gate: &[f32]) -> Vec<f32> {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(ATTACK, 1.0);
        shared.params.set(DECAY, 10.0);
        shared.params.set(SUSTAIN, 0.5);
        shared.params.set(RELEASE, 10.0);
        let silence = vec![0.0; gate.len()];
        // `in` unconnected: the output is the envelope itself.
        run_block(
            module.as_mut(),
            &shared,
            &[&silence, gate],
            0b10,
            1,
            gate.len(),
        )
        .remove(0)
    }

    #[test]
    fn stages_hit_their_targets() {
        let mut gate = vec![1.0; 2000];
        gate.extend(vec![0.0; 1000]);
        let out = render(&gate);
        // Attack: 1 ms at 48 kHz.
        assert!((out[47] - 1.0).abs() < 1e-4, "{}", out[47]);
        assert!(out[23] > 0.45 && out[23] < 0.55);
        // Decay to sustain over 10 ms.
        assert!((out[48 + 480] - 0.5).abs() < 1e-3, "{}", out[528]);
        assert_eq!(out[1999], 0.5);
        // Release from 0.5 takes half the release time.
        assert!(out[2000 + 230] > 0.0);
        assert_eq!(out[2000 + 250], 0.0);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn a_rising_edge_retriggers_from_the_current_level() {
        let mut gate = vec![1.0; 1000];
        gate.extend(vec![0.0; 100]);
        gate.extend(vec![1.0; 100]);
        let out = render(&gate);
        let before = out[1099];
        assert!(before > 0.0 && before < 0.5);
        assert!(out[1100] > before, "attack resumes upward");
        // From ~0.29 the 1 ms attack reaches the top in about 34 samples.
        let peak = out[1100..]
            .iter()
            .fold(0.0_f32, |peak, &sample| peak.max(sample));
        assert_eq!(peak, 1.0);
        assert!(out[1100..1140].contains(&1.0));
    }

    #[test]
    fn a_connected_input_is_shaped() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(SUSTAIN, 1.0);
        shared.params.set(ATTACK, 0.5);
        let input = vec![0.5; 200];
        let gate = vec![1.0; 200];
        let out = run_block(module.as_mut(), &shared, &[&input, &gate], 0b11, 1, 200).remove(0);
        assert!((out[199] - 0.5).abs() < 1e-6, "{}", out[199]);
    }
}
