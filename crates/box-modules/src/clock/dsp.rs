//! Clock DSP: one f64 beat phase drives both outputs.

use box_core::{BlockIo, Meters, Module, ParamValues};

use super::{BPM, CLOCK_METER, DIV, PER_BEAT};

pub(super) fn create(sample_rate: f32, max_block: usize) -> Box<dyn Module> {
    let mut clock = Clock::default();
    clock.reset(sample_rate, max_block);
    Box::new(clock)
}

/// The clock's DSP.
#[derive(Debug)]
pub struct Clock {
    /// Position within the current beat, 0..1.
    beat: f64,
    inverse_rate: f64,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            beat: 0.0,
            inverse_rate: 1.0 / 48_000.0,
        }
    }
}

impl Module for Clock {
    fn reset(&mut self, sample_rate: f32, _max_block: usize) {
        self.inverse_rate = 1.0 / f64::from(sample_rate.max(1.0));
        self.clear();
    }

    fn clear(&mut self) {
        self.beat = 0.0;
    }

    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        let step = f64::from(params.get(BPM)) / 60.0 * self.inverse_rate;
        let per_beat = PER_BEAT[params.choice(DIV).min(PER_BEAT.len() - 1)];
        let [clock, beat, ..] = &mut *io.outputs else {
            return;
        };
        for index in 0..io.len {
            let sub = (self.beat * per_beat).fract();
            clock[index] = if sub < 0.5 { 1.0 } else { 0.0 };
            beat[index] = if self.beat < 0.5 { 1.0 } else { 0.0 };
            self.beat += step;
            if self.beat >= 1.0 {
                self.beat -= 1.0;
            }
        }
        meters.set(CLOCK_METER, clock.last().copied().unwrap_or(0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ENTRY;
    use crate::tests::{instance, run_block};

    fn rising(signal: &[f32]) -> Vec<usize> {
        (1..signal.len())
            .filter(|&index| signal[index] == 1.0 && signal[index - 1] == 0.0)
            .collect()
    }

    #[test]
    fn period_and_duty_at_120_bpm_for_every_division() {
        // 120 BPM at 48 kHz: a beat is 24 000 samples.
        for (div, per_beat) in [(0, 1), (1, 2), (2, 4), (3, 8)] {
            let (mut module, shared) = instance(&ENTRY);
            shared.params.set(BPM, 120.0);
            shared.params.set(DIV, div as f32);
            let out = run_block(module.as_mut(), &shared, &[], 0, 2, 48_000);
            let clock = &out[0];
            let period = 24_000 / per_beat;
            let edges = rising(clock);
            assert_eq!(edges.len(), 2 * per_beat - 1, "div {div}");
            for pair in edges.windows(2) {
                assert!(
                    (pair[1] - pair[0]).abs_diff(period) <= 1,
                    "div {div}: {pair:?}"
                );
            }
            let high = clock.iter().filter(|&&sample| sample == 1.0).count();
            assert!(
                high.abs_diff(24_000) <= 2 * per_beat,
                "div {div}: {high} high"
            );
        }
    }

    #[test]
    fn beat_lines_up_with_clock_on_downbeats() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(DIV, 2.0);
        let out = run_block(module.as_mut(), &shared, &[], 0, 2, 96_000);
        let clock_edges = rising(&out[0]);
        for beat_edge in rising(&out[1]) {
            assert!(clock_edges.contains(&beat_edge), "beat edge {beat_edge}");
        }
        assert_eq!(out[0][0], 1.0);
        assert_eq!(out[1][0], 1.0);
    }

    #[test]
    fn no_drift_after_ten_minutes() {
        let (mut module, shared) = instance(&ENTRY);
        shared.params.set(BPM, 137.0);
        shared.params.set(DIV, 0.0);
        let block = 4096;
        let total = 48_000 * 600;
        let mut edges = 0_usize;
        let mut last = 0.0_f32;
        let mut done = 0;
        while done < total {
            let len = block.min(total - done);
            let out = run_block(module.as_mut(), &shared, &[], 0, 2, len);
            for &sample in &out[1] {
                if sample == 1.0 && last == 0.0 {
                    edges += 1;
                }
                last = sample;
            }
            done += len;
        }
        // 137 BPM for 10 minutes is exactly 1370 beats. Counting starts from
        // a low `last`, so the beat at sample 0 counts and the one landing
        // exactly at the end does not.
        assert_eq!(edges, 1370);
    }

    #[test]
    fn outputs_stay_binary_at_the_bpm_extremes() {
        for bpm in [20.0, 300.0] {
            let (mut module, shared) = instance(&ENTRY);
            shared.params.set(BPM, bpm);
            shared.params.set(DIV, 3.0);
            let out = run_block(module.as_mut(), &shared, &[], 0, 2, 20_000);
            assert!(out.iter().flatten().all(|&s| s == 0.0 || s == 1.0));
        }
    }
}
