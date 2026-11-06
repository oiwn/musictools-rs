---
id: 0027-box-app
status: done
scope: [crates/box-app/]
created: 2026-10-31
source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
depends: [0024-box-module-library]
---
# Task: box app

## Plan

- [x] `crates/box-app` binary: eframe 0.37 window on the neon visuals and fonts, plus a cpal output stream on the default device; the engine runs in the cpal callback under the same realtime rules as a host (no alloc, no locks); the UI talks to it only through the 0023 queues and the `ParamStore`s.
- [x] Canvas (view only): 96 px cell grid with pan; tiles drawn at their cells with a box-owned frame (header strip with power and label, port nubs on the edges) around each module's view; wires as beziers between nubs.
- [x] Live module views: knobs and toggles write the `ParamStore`, so sound follows the UI; the power toggle bypasses a module.
- [x] Move tiles (user, 2026-10-31: "Allow moving tiles too"): drag a tile by its header, snap to cells, a drop that would overlap another tile snaps back. Moving changes only the patch cell, never the graph.
- [x] Default patch on first launch: Gates → Env (gate), Sine → Env (in), Env → Output, so it plays straight away.
- [x] Patch save and load as JSON: an optional path argument (default `box-patch.json` in the working directory), ⌘S saves and ⌘O reloads it; no file dialog. An unknown module kind loads as a placeholder tile instead of failing.
- [x] Status line: device, sample rate, block size, engine load (% of the block budget).
- [x] Tests: cell ↔ pixel math, tile frame and port positions per descriptor size, overlap refusal, default patch compiles, JSON round trip.

## Manual checks

- [x] `cargo run -p box-app --release`: the default patch plays a gated sine, knobs change the sound live, save/reload restores it. — user: "Wow it looks good and feels good!" / save/reload: "It worked, close"

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Fourth task of the box chain (goals and decisions in 0014). User, 2026-10-31: the grid "should be able to start as app without host plugins (easy to debug)"; audio through cpal; 96 px cells.
- New workspace dependencies: `cpal` and `eframe` (already a neon-ui dev-dependency at 0.37), `serde_json`. Editing (add, move, wire) is 0026; this task only shows and plays a patch.

## Review

- Matches the plan inside `crates/box-app/`: `main.rs` (app, shortcuts, status line), `audio.rs` (cpal stream), `canvas.rs` (grid, frames, wires, nubs, pan, tile moves) and `patch_io.rs` (default patch, save/load).
- Audio: an f32 stream on the default device. The callback only calls `Engine::process` into preallocated scratch, interleaves the result and records its load in atomics. A mono device gets L+R averaged.
- If no device opens, the UI still runs and the status line says why. Non-f32 devices are refused.
- Canvas:
  - Box draws the frame: a power lamp (click to bypass), a tracked label, and an inset dim while bypassed. Port nubs show their name on hover.
  - Wires are glowing beziers that leave each nub along its side.
  - Dragging a header moves the tile live, and the drop snaps to the nearest cell. A drop onto another tile snaps back with a hint.
  - Dragging empty space pans.
- ⌘O reloads with `forget_instances()`, so a reloaded patch starts from its saved params.
- Verified:
  - 11 tests pass.
  - The release build ran for 6 s on the default output device with no stream errors, at about 4 % CPU.
  - An opt-in offscreen render (`cargo test -p box-app -- --ignored snapshot` → `target/snapshots/box-canvas.png`) shows all four tiles, the Env curve and the wires. It needs a GPU, so it stays out of the default suite. The test-only `egui_kittest` and `image` deps come in with it.
- Not checked by me: listening to the sound and the feel of the knobs. That's the manual check.

## Summary

box app: a standalone eframe + cpal app that shows a patch on the cell grid with live module tiles and plays it.

## Log

- 2026-10-31 created: box app
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 set depends 0024-box-module-library
- 2026-10-31 scope add crates/box-app/ — standalone eframe + cpal app
- 2026-10-31 draft approved: let's plan to implement basic version of grid to have an idea — Full chain as drafted; Allow moving tiles too; plan approved (user, 2026-10-31)
- 2026-10-31 advance draft → ready
- 2026-10-31 scope approved: crates/box-app/
- 2026-10-31 advance ready → in-progress/implement
- 2026-10-31 baseline: crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@cf56491f7121, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@6571c7427ca0, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8, crates/box-modules/Cargo.toml@5a2bc9f7141a, crates/box-modules/src/env.rs@58106b9e9593, crates/box-modules/src/gates.rs@318234aac1e2, crates/box-modules/src/lib.rs@1e4576af9f58, crates/box-modules/src/output.rs@69d37b6ca921, crates/box-modules/src/sine.rs@ba248baed010, crates/box-modules/src/view.rs@d9390f27c93c
- 2026-10-31 advance in-progress/implement → in-progress/verify
- 2026-10-31 advance in-progress/verify → in-progress/review
- 2026-10-31 advance in-progress/review → approval
- 2026-11-01 approved: Wow it looks good and feels good! — It worked, close (user, 2026-11-01)
- 2026-11-01 advance approval → done
