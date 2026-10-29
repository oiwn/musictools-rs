---
id: 0008-roman-dsp-biquad
status: draft
scope: [crates/roman-dsp/, Cargo.toml]
created: 2026-10-29
source: user request 2026-10-29: Roman biquad filter plugin (reference code in task Context)
---
# Task: roman dsp biquad

## Plan

- [ ] `crates/roman-dsp` skeleton: workspace member, `RomanSettings` Copy struct (filter type, cutoff Hz, Q, slope, sample rate) plus a per-sample frame struct; identity default.
- [ ] Port the reference biquad from Context: drop `nih_plug` (`nih_debug_assert!` becomes `debug_assert!`, the `ChannelSamplesIter` helper goes away), keep the RBJ cookbook math.
- [ ] Add bandpass and notch from the same cookbook page; keep allpass.
- [ ] Slope 12/24 dB/oct: cascade one or two biquads per channel; cache coefficients so they recompute only when type, cutoff, Q, slope, or sample rate change.
- [ ] House rules: `finite_or` on settings entry, allocation-free `process`, `reset` clears filter state.
- [ ] Tests: identity passthrough; lowpass passes low and attenuates high; highpass mirrors it; bandpass/notch band behaviour; non-finite input stays finite; L/R independence; reset clears state; coefficients recompute only on change.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Context

- Reference implementation from the user's earlier project (nih_plug-based), stored verbatim below by request. This file exceeds the default word budget because of the code block; accepted.
- Cookbook: http://shepazu.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
- Design shows a Low Pass / High Pass / Band Pass / Notch select and a 24 dB/Oct slope option, which drives the mode list and the cascade.

```rust
use nih_plug::buffer::ChannelSamplesIter;
use nih_plug::debug::*;
use std::f32::consts::TAU;
/// Biquad filter definition
/// http://shepazu.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html

pub const NEUTRAL_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;

#[derive(Debug, Copy, Clone)]
pub struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

pub struct BiquadFilter {
    pub coefficients: BiquadCoefficients,
    s1: f32,
    s2: f32,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self {
            coefficients: BiquadCoefficients::identity(),
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl BiquadCoefficients {
    pub fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    pub fn lowpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        nih_debug_assert!(sample_rate > 0.0);
        nih_debug_assert!(frequency > 0.0);
        nih_debug_assert!(frequency < sample_rate / 2.0);
        nih_debug_assert!(q > 0.0);

        let omega0 = TAU * (frequency / sample_rate);
        let cos_omega0 = omega0.cos();
        let alpha = omega0.sin() / (2.0 * q);

        // prenormalize with a0
        let a0 = 1.0 + alpha;
        let b0 = ((1.0 - cos_omega0) / 2.0) / a0;
        let b1 = (1.0 - cos_omega0) / a0;
        let b2 = ((1.0 - cos_omega0) / 2.0) / a0;
        let a1 = (-2.0 * cos_omega0) / a0;
        let a2 = (1.0 - alpha) / a0;

        Self { b0, b1, b2, a1, a2 }
    }

    pub fn highpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        nih_debug_assert!(sample_rate > 0.0);
        nih_debug_assert!(frequency > 0.0);
        nih_debug_assert!(frequency < sample_rate / 2.0);
        nih_debug_assert!(q > 0.0);

        let omega0 = TAU * (frequency / sample_rate);
        let cos_omega0 = omega0.cos();
        let alpha = omega0.sin() / (2.0 * q);

        // prenormalize with a0
        let a0 = 1.0 + alpha;
        let b0 = ((1.0 + cos_omega0) / 2.0) / a0;
        let b1 = -(1.0 + cos_omega0) / a0;
        let b2 = ((1.0 + cos_omega0) / 2.0) / a0;
        let a1 = (-2.0 * cos_omega0) / a0;
        let a2 = (1.0 - alpha) / a0;

        BiquadCoefficients { b0, b1, b2, a1, a2 }
    }

    /// Allpass filter
    pub fn allpass(sample_rate: f32, frequency: f32, q: f32) -> Self {
        nih_debug_assert!(sample_rate > 0.0);
        nih_debug_assert!(frequency > 0.0);
        nih_debug_assert!(frequency < sample_rate / 2.0);
        nih_debug_assert!(q > 0.0);

        let omega0 = TAU * (frequency / sample_rate);
        let cos_omega0 = omega0.cos();
        let alpha = omega0.sin() / (2.0 * q);

        // We'll prenormalize everything with a0
        let a0 = 1.0 + alpha;
        let b0 = (1.0 - alpha) / a0;
        let b1 = (-2.0 * cos_omega0) / a0;
        let b2 = (1.0 + alpha) / a0;
        let a1 = (-2.0 * cos_omega0) / a0;
        let a2 = (1.0 - alpha) / a0;

        BiquadCoefficients { b0, b1, b2, a1, a2 }
    }
}

impl BiquadFilter {
    pub fn process(&mut self, sample: f32) -> f32 {
        let result = self.coefficients.b0 * sample + self.s1;

        self.s1 =
            self.coefficients.b1 * sample - self.coefficients.a1 * result + self.s2;
        self.s2 = self.coefficients.b2 * sample - self.coefficients.a2 * result;

        result
    }

    #[allow(dead_code)]
    pub fn process_on_place(&mut self, channel_samples: &mut ChannelSamplesIter) {
        for sample in channel_samples {
            let smp = *sample;
            let result = self.coefficients.b0 * smp;

            self.s1 =
                self.coefficients.b1 * smp - self.coefficients.a1 * result + self.s2;
            self.s2 = self.coefficients.b2 * smp - self.coefficients.a2 * result;

            *sample = result;
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}
```

## Summary

roman-dsp: framework-free RBJ biquad with LP/HP/BP/Notch/AP, 12/24 dB-per-oct cascade, coefficient caching, and house-rule hygiene.

## Log

- 2026-10-29 created: roman dsp biquad
- 2026-10-29 set source user request 2026-10-29: Roman biquad filter plugin (reference code in task Context)
- 2026-10-29 scope add crates/roman-dsp/ — new DSP crate and workspace membership
- 2026-10-29 scope add Cargo.toml — new DSP crate and workspace membership
