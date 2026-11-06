---
id: 0032-box-sequencer-module
status: approval
scope: [crates/box-modules/, crates/box-app/src/canvas.rs]
created: 2026-11-01
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0029-box-module-folders]
---
# Task: box sequencer module

## Plan

- [x] `box-modules/src/seq/` (4x2): 8 steps, each a pitch knob (−24…+24 semitones, snapping to whole semitones) above an on/off gate toggle, with a lamp on the current step.
- [x] Ports: inputs `clock` and `reset` (left), outputs `pitch` (octaves, per `box_core::pitch`) and `gate` (right).
- [x] DSP: advance on each rising `clock` edge and wrap after step 8; a rising `reset` returns to step 1 on the next clock; `gate` follows `clock` while the current step is on; `pitch` holds the step's value between clocks. The current step goes to a meter for the lamp.
- [x] Tests: steps advance and wrap; reset; gate width follows the clock and is silent on off steps; pitch per step in exact semitones; every widget fits the 4x2 body (layout math).

## Manual checks

- [ ] In the app: Clock → Seq → Sine pitch, Seq gate → Env plays the 8-step line; knobs and toggles change it while playing.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- User, 2026-11-01: "sequencer"; size answer "8 steps". Pitch convention from 0029; clock usually from 0031. Scaling the steps to a key is the scaler's job (0033).
- The gate toggles need a small on/off widget: reuse `neon_ui::button::toggle`/`switch` if it fits the cell, otherwise draw a lamp button in the view.

## Review

- Matches the plan inside `crates/box-modules/` (`seq/` plus the registry line, in a new "Sequencing" category).
- 16 params:
  - 8 step pitches (−24…+24, rounded to whole semitones in the DSP and the readout `+7 st`);
  - 8 gate bools.
- The kit's `toggle`/`switch` don't fit under a 44 px knob, so the view draws its own lamp buttons. A cyan LED means the step is on; the current step's button glows yellow.
- DSP:
  - The first clock after start or reset plays step 1. A rising `reset` makes the next clock start over.
  - `gate` = clock high and step on.
  - The step meter reads −1 before the first clock.
- Tests: advance and wrap with exact pitches, gate width and off steps, reset between clocks, semitone snapping. The registry test checks the 4x2 layout.
- Added `snapshot_module_gallery` (opt-in, `target/snapshots/box-gallery.png`) to box-app's canvas tests, which renders every module for layout review. Scope was widened for it.
- The full gate is green.

## Summary

box sequencer module: an 8-step pitch and gate sequencer driven by a clock input.

## Log

- 2026-11-01 created: box sequencer module
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0029-box-module-folders
- 2026-11-01 scope add crates/box-modules/ — sequencer module folder and registry line
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-modules/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-app/Cargo.toml@a1e01547be2a, crates/box-app/src/audio.rs@2ee2f910d516, crates/box-app/src/browser.rs@123399bf722c, crates/box-app/src/canvas.rs@3554c0123882, crates/box-app/src/edit.rs@30985ff7bec6, crates/box-app/src/main.rs@c39457b81d7b, crates/box-app/src/patch_io.rs@b5d60a9d91fd, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8
- 2026-11-01 scope add crates/box-app/src/canvas.rs — opt-in module gallery render to review new tile layouts
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
