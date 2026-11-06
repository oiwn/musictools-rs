---
id: 0008-roman-dsp-biquad
status: approval
scope: [crates/roman-dsp/, Cargo.toml]
created: 2026-10-29
source: user request 2026-10-29: Roman biquad filter plugin (reference code in task Context)
---
# Task: roman dsp biquad

## Plan

- [x] `crates/roman-dsp` skeleton: workspace member, `RomanSettings` Copy struct (filter type, cutoff Hz, Q, slope, sample rate) plus a per-sample frame struct; identity default.
- [x] Port the reference biquad from Context: drop `nih_plug` (`nih_debug_assert!` becomes `debug_assert!`, the `ChannelSamplesIter` helper goes away), keep the RBJ cookbook math.
- [x] Add bandpass and notch from the same cookbook page; keep allpass.
- [x] Slope 12/24 dB/oct: cascade one or two biquads per channel; cache coefficients so they recompute only when type, cutoff, Q, slope, or sample rate change.
- [x] House rules: `finite_or` on settings entry, allocation-free `process`, `reset` clears filter state.
- [x] Tests: identity passthrough; lowpass passes low and attenuates high; highpass mirrors it; bandpass/notch band behaviour; non-finite input stays finite; L/R independence; reset clears state; coefficients recompute only on change.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

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

## Review

- Matches the plan inside `crates/roman-dsp/`, plus the workspace manifest.
- `biquad.rs` holds the cookbook coefficients:
  - lowpass, highpass, the new bandpass (0 dB peak) and notch, and allpass;
  - out-of-range or non-finite cutoff, Q or rate are clamped (cutoff to 1 Hz–0.49 × rate, Q to 0.1–40) instead of the reference's debug asserts;
  - a `magnitude()` helper for the 0009 response curve;
  - a TDF-II `BiquadFilter`.
- `lib.rs`:
  - `FilterType` (with `from_index` for choice params), `Slope`, `RomanSettings` (sanitized on entry), a `Frame`;
  - `RomanFilter`: one channel, identity until the first `set`, recomputes only on change (counted by `coefficient_updates`), and resets the second stage when switching to 24 dB;
  - `StereoRoman`: two channels.
- Deviation: 24 dB uses two identical sections at the chosen Q, not Butterworth-staggered Qs, so its knee is softer than a textbook 4th-order filter. That's fine for v1 and noted for 0009.
- 14 tests pass, including rt-audit across process and type changes. The full gate is green.

## Summary

roman-dsp: framework-free RBJ biquad with LP/HP/BP/Notch/AP, 12/24 dB-per-oct cascade, coefficient caching, and house-rule hygiene.

## Log

- 2026-10-29 created: roman dsp biquad
- 2026-10-29 set source user request 2026-10-29: Roman biquad filter plugin (reference code in task Context)
- 2026-10-29 scope add crates/roman-dsp/ — new DSP crate and workspace membership
- 2026-10-29 scope add Cargo.toml — new DSP crate and workspace membership
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/roman-dsp/, Cargo.toml
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-app/Cargo.toml@a1e01547be2a, crates/box-app/src/audio.rs@2ee2f910d516, crates/box-app/src/browser.rs@123399bf722c, crates/box-app/src/canvas.rs@b02af8a5e564, crates/box-app/src/edit.rs@30985ff7bec6, crates/box-app/src/main.rs@c39457b81d7b, crates/box-app/src/patch_io.rs@b5d60a9d91fd, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8, crates/box-modules/Cargo.toml@5a2bc9f7141a, crates/box-modules/src/clock/dsp.rs@18971805ba7d, crates/box-modules/src/clock/mod.rs@b354b484ce0c, crates/box-modules/src/clock/view.rs@b0117a23a69e, crates/box-modules/src/env/dsp.rs@37ca3a6402b8, crates/box-modules/src/env/mod.rs@ec2c7b0cc8b2, crates/box-modules/src/env/view.rs@1ae063571467, crates/box-modules/src/format.rs@70f4856a6f0e, crates/box-modules/src/gates/dsp.rs@cda17ff8489e, crates/box-modules/src/gates/mod.rs@ff847d58e2b8, crates/box-modules/src/gates/view.rs@aefb6a948ffb, crates/box-modules/src/lib.rs@dd2701f562cf, crates/box-modules/src/mix4/dsp.rs@a00246f57fa5, crates/box-modules/src/mix4/mod.rs@184fc7915678, crates/box-modules/src/mix4/view.rs@004acf5e3d1a, crates/box-modules/src/output/dsp.rs@8cc3825ee817, crates/box-modules/src/output/mod.rs@11be8c8aa138, crates/box-modules/src/output/view.rs@e14b8bf6b6b0, crates/box-modules/src/scaler/dsp.rs@6c58f54e0972, crates/box-modules/src/scaler/mod.rs@df2fbff22fdc, crates/box-modules/src/scaler/scales.rs@226ab847393e, crates/box-modules/src/scaler/view.rs@fb2bea07c4ab, crates/box-modules/src/seq/dsp.rs@ff8fdf861e56, crates/box-modules/src/seq/mod.rs@31e3ae417a30, crates/box-modules/src/seq/view.rs@62ba5dee0bca, crates/box-modules/src/sine/dsp.rs@5cd092f72bc8, crates/box-modules/src/sine/mod.rs@8dd799221f08, crates/box-modules/src/sine/view.rs@fa64a1efe809, crates/box-modules/src/tile.rs@644ff8e7e087
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
