---
id: 0029-box-module-folders
status: approval
scope: [crates/box-modules/, crates/box-core/src/]
created: 2026-11-01
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0024-box-module-library]
---
# Task: box module folders

## Plan

- [x] Folder per module: move sine, gates, env and output into `crates/box-modules/src/<name>/` with `mod.rs` (descriptor, `ENTRY`, `KIND`), `dsp.rs` (the `Module` impl; never imports egui) and `view.rs` (tile view and layout). Registration stays one line in `MODULES`; tests move with their code.
- [x] `box_core::pitch`: `C4_HZ`, `pitch_to_hz(base, pitch)`, `semitones(n)` / `cents(n)` → pitch, and back; documented convention (pitch is octaves, 0.0 = C4).
- [x] Sine gets a `pitch` input: `freq = FREQ × 2^pitch` (unconnected reads 0, so nothing changes for existing patches); FREQ default moves to C4 so sequenced notes are in tune.
- [x] `specs/box.md`: a Pitch section (octaves, C4 = 0, gates/clocks stay 0/1) and the add-a-module recipe rewritten for the folder layout.
- [x] Tests: pitch helpers invert, and A4 (pitch 9/12) is 440 Hz; Sine an octave up with pitch 1.0; everything from 0024 still passes after the move.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Next box chain (user, 2026-11-01): "each module should be in own module, i think we don't need sdk since i'm doing it for myself mostly" → folder per module inside the one `box-modules` crate (answer: "Folder per module"), compiled in, no plugin SDK.
- Pitch is the convention the sequencer (0032), scaler (0033) and filter cutoff input (0034) share; it needs no new `PortKind`.
- Suggested order of the chain: 0029 → 0026 → 0030 → 0031 → 0012 → 0032 → 0033 → 0008 → 0034.

## Review

- Matches the plan inside scope:
  - Four folders, each with `mod.rs`, `dsp.rs` and `view.rs`.
  - The crate-level helpers became `tile.rs`, so they don't clash with each module's `view.rs`.
  - The formatters moved to an egui-free `format.rs`.
  - Header lamps are a shared `tile::lamp` helper.
- Sine clamps its pitch input to ±10 octaves and its frequency below Nyquist. An unconnected input leaves it exactly as before, apart from the new C4 default.
- `specs/box.md` has the Pitch convention and the folder recipe.
- `_typos.toml` (new) allows the user's verbatim approval quote ("implmenet") in the task logs.
- 21 box-modules tests pass, plus the new box-core pitch tests. The full gate is green.

## Summary

box module folders: each module in its own folder (descriptor, DSP, view), plus the pitch convention and a pitch input on Sine.

## Log

- 2026-11-01 created: box module folders
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0024-box-module-library
- 2026-11-01 scope add crates/box-modules/ — module folders and the pitch helpers
- 2026-11-01 scope add crates/box-core/src/ — module folders and the pitch helpers
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-modules/, crates/box-core/src/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: crates/box-app/Cargo.toml@a1e01547be2a, crates/box-app/src/audio.rs@2ee2f910d516, crates/box-app/src/canvas.rs@451bea6503f0, crates/box-app/src/main.rs@4c3a7094f14b, crates/box-app/src/patch_io.rs@b5d60a9d91fd, crates/box-core/Cargo.toml@da582a43791d, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
