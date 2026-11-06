---
id: 0026-box-canvas-editor
status: approval
scope: [crates/box-app/]
created: 2026-10-30
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0027-box-app]
---
# Task: box canvas editor

## Plan

- [x] Select and delete: clicking a tile header selects it (highlighted frame); Delete or Backspace removes the tile and its wires. Adding modules is the browser task (0030); moving tiles landed in 0027.
- [x] Patching: drag from a port nub to another port to wire it (a live wire follows the pointer, valid targets light up; output → input only), click a wire to remove it; a wire that would close a cycle is refused with a short hint.
- [x] Every edit goes through the 0023 compile and swap, so audio keeps playing while you patch.
- [x] Tests: hit-testing ports and wires under pan, output → input only, cycle refusal, deleting a tile drops its wires, edit → patch round trip; offscreen render of a live wire.

## Manual checks

- [ ] In the app: delete the default patch's wires and rewire Gates → Env ← Sine → Output while it plays, with no dropouts; delete a tile; save and reload.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Fifth task of the box chain (goals and decisions in 0014). Editing lands in the standalone app first (user, 2026-10-31: "easy to debug"); the plugin (0025) reuses the same canvas.
- No inspector: each module's tile is its only UI.
- Re-planned 2026-11-01: adding modules moved to the right-click browser (0030, user: "right click can show layer with modules like in vcv rack"); this task is patching, selection and delete.

## Review

- Matches the plan inside `crates/box-app/`. A new `edit.rs` holds the pure patch edits: `connect` (either drag direction, duplicate no-op, loop refused via the compiler), `delete_node`, and line hit-testing. `canvas.rs` gathers each frame's interactions into `Actions` and applies them once drawing is done.
- Wiring:
  - Dragging from a nub draws a live wire that stays attached to its nub, and opposite-direction nubs light yellow as targets.
  - Dropping on a nub connects; dropping anywhere else cancels.
- Wires: a wire on empty canvas lights yellow on hover, and clicking it removes the wire. Clicks over a tile still go to the knobs.
- Selection: clicking or dragging a header selects the tile, which glows cyan. Delete or Backspace removes the tile and its wires, unless a text field has focus. Clicking empty canvas clears the selection.
- Every edit sets `repatch`, so the app runs it through compile and swap.
- 21 box-app tests pass, including the two opt-in renders (`target/snapshots/box-canvas-wiring.png` shows the live wire, target nubs and the selection). The full gate is green.

## Summary

box canvas editing: wire and unwire ports, select and delete tiles, live while audio plays.

## Log

- 2026-10-30 created: box canvas editor
- 2026-10-30 set source user request 2026-10-30: grid foundation as "box" — expandable module interface, performance first, compact views on the grid and the full plugin in an inspector
- 2026-10-30 set depends 0025-box-plugin, 0021-neon-ui-displays-chrome
- 2026-10-30 scope add plugins/box/src/ — canvas editor and inspector
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 set depends 0027-box-app
- 2026-10-31 scope rm plugins/box/src/ — editing lands in the app first
- 2026-10-31 scope add crates/box-app/ — canvas editing in the app
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-app/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8, crates/box-modules/Cargo.toml@5a2bc9f7141a, crates/box-modules/src/env/dsp.rs@37ca3a6402b8, crates/box-modules/src/env/mod.rs@ec2c7b0cc8b2, crates/box-modules/src/env/view.rs@1ae063571467, crates/box-modules/src/format.rs@70f4856a6f0e, crates/box-modules/src/gates/dsp.rs@cda17ff8489e, crates/box-modules/src/gates/mod.rs@ff847d58e2b8, crates/box-modules/src/gates/view.rs@aefb6a948ffb, crates/box-modules/src/lib.rs@a0a13387313e, crates/box-modules/src/output/dsp.rs@8cc3825ee817, crates/box-modules/src/output/mod.rs@11be8c8aa138, crates/box-modules/src/output/view.rs@e14b8bf6b6b0, crates/box-modules/src/sine/dsp.rs@5cd092f72bc8, crates/box-modules/src/sine/mod.rs@8dd799221f08, crates/box-modules/src/sine/view.rs@fa64a1efe809, crates/box-modules/src/tile.rs@644ff8e7e087
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
