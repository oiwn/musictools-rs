//! Gates DSP.

use box_core::{BlockIo, Meters, Module, ParamValues};

use super::{GATE_METER, RATE, WIDTH};

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut gates = Gates::default();
    gates.reset(sample_rate, max_block);
    Box::new(gates)
}

/// The gate generator's DSP. Each period starts high for `width` of it;
/// edges land on exact samples.
#[derive(Debug)]
pub struct Gates {
    phase: f64,
    inverse_rate: f64,
}

impl Default for Gates {
    fn default() -> Self {
        Self {
            phase: 0.0,
            inverse_rate: 1.0 / 48_000.0,
        }
    }
}

impl Module for Gates {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        self.inverse_rate = 1.0 / f64::from(sample_rate.max(1.0));
        self.clear();
    }

    fn clear(&mut self) {
        self.phase = 0.0;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        let step = f64::from(params.get(RATE)) * self.inverse_rate;
        let width = f64::from(params.get(WIDTH));
        for out in io.outputs[0].iter_mut() {
            *out = if self.phase < width { 1.0 } else { 0.0 };
            self.phase += step;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
        meters.set(GATE_METER, io.outputs[0].last().copied().unwrap_or(0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::ENTRY;
    use crate::tests::{instance, run_block};

    #[test]
    fn gate_period_and_duty_match() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(RATE, 10.0);
        shared.params.set(WIDTH, 0.25);
        let out = run_block(module.as_mut(), &shared, &[], 0, 1, 9600).remove(0);
        assert!(out.iter().all(|&sample| sample == 0.0 || sample == 1.0));
        let high = out.iter().filter(|&&sample| sample == 1.0).count();
        assert!((2399..=2401).contains(&high), "{high} high samples");
        let rising: Vec<usize> = (1..out.len())
            .filter(|&index| out[index] == 1.0 && out[index - 1] == 0.0)
            .collect();
        assert_eq!(rising.len(), 1);
        assert!((4799..=4801).contains(&rising[0]), "{rising:?}");
        assert_eq!(out[0], 1.0);
    }
}
