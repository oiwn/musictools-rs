---
id: 0031-box-clock-module
status: approval
scope: [crates/box-modules/]
created: 2026-11-01
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0029-box-module-folders]
---
# Task: box clock module

## Plan

- [x] `box-modules/src/clock/` (1x1): BPM (20–300, default 120) and DIV (a stepped choice: 1/4, 1/8, 1/16, 1/32) knobs; outputs `clock` (50 % gate at the division) and `beat` (quarter notes, 50 %); a lamp in the header that blinks with `clock`.
- [x] DSP: one f64 phase per output, so edges land on exact samples and don't drift over long runs; a BPM or DIV change keeps the phase continuous.
- [x] Tests: period and duty at 120 BPM for every division; `beat` lines up with `clock` on downbeats; no drift after ten minutes of samples; finite at the BPM extremes.

## Manual checks

- [ ] In the app: Clock → Env gate → Output ticks in time; turning BPM and DIV changes it smoothly.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- User, 2026-11-01: "clock (bpm)". Free-running in the standalone app; following host tempo and transport belongs to the plugin (0025, noted in `ideas.md`).
- Gates stays as the free-rate gate source.

## Review

- Matches the plan inside `crates/box-modules/` (`clock/` plus the registry line, in the Sources category).
- One f64 beat phase drives both outputs instead of one phase per output: `clock` is the fractional part of `beat × per_beat`, so the two can never drift apart and a BPM or DIV change stays continuous.
- DIV is a choice param on a stepped knob.
- Tests:
  - period and duty for all four divisions at 120 BPM;
  - beat edges are always clock edges;
  - 10 minutes at 137 BPM gives exactly 1370 beats;
  - outputs stay 0/1 at 20 and 300 BPM.
- The browser test now checks the "Sources" search by category instead of a fixed list, since Clock joined that category.
- The full gate is green.

## Summary

box clock module: a BPM clock with a division knob and clock and beat gate outputs.

## Log

- 2026-11-01 created: box clock module
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0029-box-module-folders
- 2026-11-01 scope add crates/box-modules/ — clock module folder and registry line
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-modules/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-app/Cargo.toml@a1e01547be2a, crates/box-app/src/audio.rs@2ee2f910d516, crates/box-app/src/browser.rs@c49eb87cc630, crates/box-app/src/canvas.rs@3554c0123882, crates/box-app/src/edit.rs@30985ff7bec6, crates/box-app/src/main.rs@c39457b81d7b, crates/box-app/src/patch_io.rs@b5d60a9d91fd, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
