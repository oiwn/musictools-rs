//! Output DSP.

use box_core::{BlockIo, Meters, Module, ParamValues, Smoother};

use super::{LEVEL, PARAMS, PEAK};

/// Peak hold falls by this factor per second.
const PEAK_FALL_PER_SECOND: f32 = 0.05;

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut output = Output::default();
    output.reset(sample_rate, max_block);
    Box::new(output)
}

/// The sink's DSP.
#[derive(Debug)]
pub struct Output {
    level: Smoother,
    peak: f32,
    sample_rate: f32,
    primed: bool,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            level: Smoother::new(PARAMS[LEVEL].default_value()),
            peak: 0.0,
            sample_rate: 48_000.0,
            primed: false,
        }
    }
}

impl Module for Output {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        self.sample_rate = sample_rate.max(1.0);
        self.level
            .set_time(self.sample_rate, PARAMS[LEVEL].smoothing_ms);
        self.clear();
    }

    fn clear(&mut self) {
        self.peak = 0.0;
        self.primed = false;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        if self.primed {
            self.level.set_target(params.get(LEVEL));
        } else {
            self.level.snap(params.get(LEVEL));
            self.primed = true;
        }
        let right = if io.is_connected(1) { 1 } else { 0 };
        let (left_in, right_in) = (io.inputs[0], io.inputs[right]);
        let [left_out, right_out, ..] = &mut *io.outputs else {
            return;
        };
        let mut peak = 0.0_f32;
        for index in 0..io.len {
            let gain = self.level.next();
            let l = left_in[index].clamp(-1.0, 1.0) * gain;
            let r = right_in[index].clamp(-1.0, 1.0) * gain;
            left_out[index] = l;
            right_out[index] = r;
            peak = peak.max(l.abs()).max(r.abs());
        }
        let fall = PEAK_FALL_PER_SECOND.powf(io.len as f32 / self.sample_rate);
        self.peak = peak.max(self.peak * fall);
        meters.set(PEAK, self.peak);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::ENTRY;
    use crate::tests::{instance, run_block};

    #[test]
    fn output_stays_finite_and_below_its_level() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(LEVEL, 0.25);
        let loud = vec![4.0; 256];
        let quiet = vec![-0.5; 256];
        let out = run_block(module.as_mut(), &shared, &[&loud, &quiet], 0b11, 2, 256);
        assert!(out[0].iter().all(|&sample| (sample - 0.25).abs() < 1e-6));
        assert!(out[1].iter().all(|&sample| (sample + 0.125).abs() < 1e-6));
        assert!((shared.meters.get(PEAK) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn right_follows_left_when_unconnected() {
        let (mut module, shared) = instance(&ENTRY);
        let left = vec![0.5; 16];
        let silence = vec![0.0; 16];
        let out = run_block(module.as_mut(), &shared, &[&left, &silence], 0b01, 2, 16);
        assert_eq!(out[0], out[1]);
        assert!(out[1][0] > 0.0);
    }
}
