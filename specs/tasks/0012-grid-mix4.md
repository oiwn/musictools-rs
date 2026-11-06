---
id: 0012-grid-mix4
status: approval
scope: [crates/box-modules/]
created: 2026-10-29
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0029-box-module-folders]
---
# Task: grid mix4

## Plan

- [x] `box-modules/src/mix4/` (4x1): four mono inputs `1`–`4` (left), stereo outputs `l` and `r` (right) that patch straight into Output.
- [x] Per channel a LEVEL and a PAN knob (8 small knobs, 352 of the 380 px body), both smoothed; constant-power pan law (−3 dB at center).
- [x] Tests: the pan law (hard left, center, hard right), four channels sum, an unconnected channel adds nothing, finite output with hot inputs, the layout fits the 4x1 body.

## Manual checks

- [ ] In the app: two sources into Mix4 → Output; level and pan move them across the stereo field.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Re-planned 2026-11-01 for box (user: "mix4 with pan"): mono inputs with level and pan replace the old stereo-input, mute/solo and master-trim plan; box signals are mono per port. The id and title stay.

## Review

- Matches the plan inside `crates/box-modules/` (`mix4/` plus the registry line, in a new "Mixing" category).
- Params run LVL 1, PAN 1, … LVL 4, PAN 4 in knob order:
  - level is 0–1 on a skewed knob, reading in dB (default −3 dB);
  - pan reads `L40` / `C` / `R100`.
- Both are smoothed per channel. An unconnected channel skips its math but keeps its glides moving.
- Tests cover the pan law (center −3 dB, hard sides, constant power across the sweep), summing with one unconnected channel full of junk, and finite output from 1e30 inputs. The registry test checks the 4x1 layout fits.
- The full gate is green.

## Summary

box mix4 module: four mono channels with level and constant-power pan into a stereo pair.

## Log

- 2026-10-29 created: grid mix4
- 2026-10-29 set source user request 2026-10-29: grid module (parked until grid foundation)
- 2026-10-29 set depends 0014-grid-foundation
- 2026-10-30 set depends 0024-box-module-library
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0029-box-module-folders
- 2026-11-01 scope add crates/box-modules/ — mix4 module folder and registry line
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-modules/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-app/Cargo.toml@a1e01547be2a, crates/box-app/src/audio.rs@2ee2f910d516, crates/box-app/src/browser.rs@123399bf722c, crates/box-app/src/canvas.rs@3554c0123882, crates/box-app/src/edit.rs@30985ff7bec6, crates/box-app/src/main.rs@c39457b81d7b, crates/box-app/src/patch_io.rs@b5d60a9d91fd, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
