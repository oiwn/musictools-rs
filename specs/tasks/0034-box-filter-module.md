---
id: 0034-box-filter-module
status: approval
scope: [crates/box-modules/, crates/box-app/]
created: 2026-11-01
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0008-roman-dsp-biquad, 0029-box-module-folders]
---
# Task: box filter module

## Plan

- [x] `box-modules/src/filter/` (2x1), the first plugin DSP in box: wraps `roman-dsp` (0008). Knobs CUTOFF (log 20 Hz–20 kHz), Q, TYPE (LP/HP/BP/Notch); SLOPE 12/24 dB as a lamp toggle in the header.
- [x] Ports: inputs `in` and `cutoff` (pitch-style modulation: cutoff × 2^input, clamped below Nyquist), output `out`. Coefficients recompute only when the effective cutoff, Q, type or slope change (per block, or per 16 samples while `cutoff` is wired).
- [x] Demo default patch in `box-app`: Clock → Seq → Scaler → Sine, Seq gate → Env, Sine → Filter → Env → Mix4 → Output, laid out so wires read cleanly; `default_patch` test updated.
- [x] Tests: matches `roman-dsp` directly for the same settings; cutoff input of +1.0 doubles the cutoff; stays finite at extreme Q and modulation; tile layout fits; rt-audit on the demo patch.

## Manual checks

- [ ] In the app: the demo patch plays; sweeping CUTOFF and TYPE is audible with no zipper noise.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- User, 2026-11-01: "will need to create filter and this will be first plugin available in box"; answer "DSP + box module first": 0008 roman-dsp, then this module; the standalone ppc_roman plugin (0009) follows later.
- The demo patch needs Clock (0031), Mix4 (0012), Seq (0032) and Scaler (0033); if one of them isn't done yet, the demo uses what exists.

## Review

- Matches the plan inside `crates/box-modules/` (`filter/` plus the registry line, in a new "Filters" category) and `crates/box-app/` (the demo patch).
- Filter:
  - It wraps `roman_dsp::RomanFilter` (mono).
  - CUTOFF and Q glide over 30 ms. The effective cutoff (glide × 2^input, clamped to 20 Hz–0.49 × rate) is re-applied every 16 samples, and roman-dsp recomputes only when it actually changed.
  - The header's "24dB" lamp toggles the slope.
  - TYPE covers LP/HP/BP/Notch. Allpass stays in the DSP only.
- Demo patch (`patch_io::demo_patch`, opened on first launch):
  - Clock (112 BPM, 1/8) → Seq (a minor-pentatonic line, step 6 muted) → Scaler (A minor pentatonic) → Sine;
  - Sine → Env, gated by Seq → Filter (LP 1.4 kHz, Q 2) → Mix4 → Output.
  - Each row reads left to right; two short return wires drop to the next row.
  - The window grew to 1280×712.
- The old gated-sine patch stays as `default_patch` for the tests only.
- Tests:
  - the module matches roman-dsp sample for sample;
  - a cutoff input of +1 equals the filter at double the cutoff;
  - finite at Q 12, 24 dB, every type and ±40 modulation;
  - the demo compiles with every param id known and no overlaps;
  - a new rt-audit test renders the whole demo chain with zero allocations.
- Opt-in renders: `target/snapshots/box-demo.png` (demo) and `box-gallery.png` (all modules). Snapshot sizes now grow to fit the patch.
- The release app ran 6 s on the default device with no stream errors, at about 6.5 % CPU.
- The full gate is green.

## Summary

box filter module: Roman's biquad as a box tile with a pitch-style cutoff input, plus the demo default patch.

## Log

- 2026-11-01 created: box filter module
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0008-roman-dsp-biquad, 0029-box-module-folders
- 2026-11-01 scope add crates/box-modules/ — filter module over roman-dsp; demo default patch
- 2026-11-01 scope add crates/box-app/ — filter module over roman-dsp; demo default patch
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-modules/, crates/box-app/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8, crates/roman-dsp/Cargo.toml@eb2bb2c8c206, crates/roman-dsp/src/biquad.rs@90bc1bce5ad6, crates/roman-dsp/src/lib.rs@a968e62cb6cd
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
