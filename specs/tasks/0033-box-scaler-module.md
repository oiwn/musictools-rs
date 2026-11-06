---
id: 0033-box-scaler-module
status: approval
scope: [crates/box-modules/]
created: 2026-11-01
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0029-box-module-folders]
---
# Task: box scaler module

## Plan

- [x] `box-modules/src/scaler/` (2x1): ROOT (C…B) and SCALE (choice) knobs; the body shows the scale name and a 12-slot keyboard strip lighting the scale's notes (microtonal degrees drawn at their cents position).
- [x] `scaler/scales.rs`: each scale is a name plus degrees in cents within the octave (0 first, all below 1200), so microtonal scales work.
  - Diatonic and common: the seven major modes, harmonic minor, melodic minor, major and minor pentatonic, blues, whole tone, both diminished scales, chromatic.
  - Exotic 12-TET: Hungarian minor, double harmonic, Phrygian dominant, Persian, Neapolitan major and minor, enigmatic, Hirajoshi, In-sen, Iwato, Kumoi, Egyptian, Pelog (12-TET approximation).
  - Microtonal: Maqam Rast and Bayati (quarter tones), Slendro and Pelog in gamelan cents, 19-TET and 24-TET chromatic.
- [x] DSP: input `pitch`, output `pitch`; snap each sample to the nearest scale degree relative to ROOT, across octaves (wraps at the octave boundary in both directions). Only recompute the target when the input moves, so a held note costs one compare per sample.
- [x] Tests: every table is sorted, starts at 0 and stays below 1200; each degree maps to itself; a pitch halfway between two degrees picks the nearer; ROOT transposes the scale; quarter-tone degrees come out exact; negative pitches and octave wrap; non-finite input stays finite.

## Manual checks

- [ ] In the app: Seq → Scaler → Sine plays in key; switching to an exotic and a microtonal scale changes the line audibly.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- User, 2026-11-01: "scaler (including exotic scales, like in bitwig 6)". Bitwig 6's full scale list isn't published (its exotic tunings go through Micropitch), so this catalog is our own; custom scales and Scala `.scl` import are in `ideas.md`.
- Pitch convention from 0029 (octaves, C4 = 0): cents = pitch × 1200.

## Review

- Matches the plan inside `crates/box-modules/` (`scaler/` plus the registry line, in a new "Pitch" category).
- Catalog: 35 scales, all listed in the plan. The equal temperaments (12, 19, 24) are built by a const fn. `NAMES` feeds the SCALE choice param.
- `quantize` finds the nearest degree within the octave. The next octave's root closes the top gap, and ties go to the lower degree. The DSP caches the last input and output, so a held note costs one compare, and a settings change forces a recompute.
- Tile:
  - The SCALE knob reads `5 / 35`, and the scale name sits in the header.
  - The body strip shows a faint 12-tone grid with the scale's degrees marked at their cents (quarter tones and gamelan steps fall between the lines). The degree playing now glows yellow.
- Tests:
  - the tables (sorted, start at 0, below 1200, unique names);
  - every degree maps to itself in three octaves;
  - nearest-degree choice, root transposition, quarter tones, negative pitches and octave wrap, non-finite input;
  - a C-major ramp lands only on scale degrees;
  - a scale change requantizes a held note.
- The full gate is green.

## Summary

box scaler module: quantizes pitch to a root and scale, including exotic and microtonal scales.

## Log

- 2026-11-01 created: box scaler module
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0029-box-module-folders
- 2026-11-01 scope add crates/box-modules/ — scaler module folder, scale catalog, registry line
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-modules/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-app/Cargo.toml@a1e01547be2a, crates/box-app/src/audio.rs@2ee2f910d516, crates/box-app/src/browser.rs@123399bf722c, crates/box-app/src/canvas.rs@b02af8a5e564, crates/box-app/src/edit.rs@30985ff7bec6, crates/box-app/src/main.rs@c39457b81d7b, crates/box-app/src/patch_io.rs@b5d60a9d91fd, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
