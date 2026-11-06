---
id: 0030-box-module-browser
status: approval
scope: [crates/box-app/]
created: 2026-11-01
source: user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
depends: [0026-box-canvas-editor]
---
# Task: box module browser

## Plan

- [x] Right-click on empty canvas opens a browser layer at the pointer (VCV Rack style): a search field focused at once, results listed by category from `MODULES`, each row with name, category and tile size.
- [x] Search filters as you type over name, category and kind (case-insensitive; prefix matches rank first); ↑/↓ move the highlight, Enter or click places, Esc or a click outside closes.
- [x] Placement: the module's tile goes at the right-clicked cell, or the nearest free cell where its whole size fits; the new node gets `patch.next_uid()` and the patch is re-applied.
- [x] Tests: filtering and ranking, nearest-free-cell search around occupied tiles for 1x1 and 2x1, placement adds exactly one node; offscreen render of the open browser.

## Manual checks

- [ ] In the app: right-click, type "env", Enter: an Env lands at the click; Esc closes without adding.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- User, 2026-11-01: "need search of modules in the box, i think right click can show layer with modules like in vcv rack".
- Right-click on a tile stays free for a later tile menu.

## Review

- Matches the plan inside `crates/box-app/`. The new `browser.rs` holds:
  - `search` (ranked: name prefix, then name substring, then category or kind; an empty query lists every module by category);
  - `nearest_free_cell` (ring search outward from the clicked cell, nearest first);
  - `place`;
  - the `Browser` layer (a foreground `egui::Area` with the kit palette).
- The canvas reports a right-click on empty space as `Outcome::browse`, and the app opens the browser there.
- Arrow keys, Enter and Esc are taken before the search field sees them. A click outside closes the browser.
- Each row shows the name and tile size under category captions, rather than the category on every row.
- 26 box-app tests pass, including the render at `target/snapshots/box-browser.png`. The full gate is green.

## Summary

box module browser: right-click opens a searchable module list that places the chosen module at the nearest free cell.

## Log

- 2026-11-01 created: box module browser
- 2026-11-01 set source user request 2026-11-01: more box modules (clock, mix4 with pan, 8-step sequencer, scaler with exotic scales), filter as the first plugin in box, right-click module search, folder per module
- 2026-11-01 set depends 0026-box-canvas-editor
- 2026-11-01 scope add crates/box-app/ — right-click module browser in the canvas
- 2026-11-01 draft approved: let's go implmenet in this order (user, 2026-11-01)
- 2026-11-01 advance draft → ready
- 2026-11-01 scope approved: crates/box-app/
- 2026-11-01 advance ready → in-progress/implement
- 2026-11-01 baseline: _typos.toml@8cd802995953, crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@9157d7e0e7e0, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/pitch.rs@016875cd0e9c, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@d4a2fbbcbb16, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8, crates/box-modules/Cargo.toml@5a2bc9f7141a, crates/box-modules/src/env/dsp.rs@37ca3a6402b8, crates/box-modules/src/env/mod.rs@ec2c7b0cc8b2, crates/box-modules/src/env/view.rs@1ae063571467, crates/box-modules/src/format.rs@70f4856a6f0e, crates/box-modules/src/gates/dsp.rs@cda17ff8489e, crates/box-modules/src/gates/mod.rs@ff847d58e2b8, crates/box-modules/src/gates/view.rs@aefb6a948ffb, crates/box-modules/src/lib.rs@a0a13387313e, crates/box-modules/src/output/dsp.rs@8cc3825ee817, crates/box-modules/src/output/mod.rs@11be8c8aa138, crates/box-modules/src/output/view.rs@e14b8bf6b6b0, crates/box-modules/src/sine/dsp.rs@5cd092f72bc8, crates/box-modules/src/sine/mod.rs@8dd799221f08, crates/box-modules/src/sine/view.rs@fa64a1efe809, crates/box-modules/src/tile.rs@644ff8e7e087
- 2026-11-01 advance in-progress/implement → in-progress/verify
- 2026-11-01 advance in-progress/verify → in-progress/review
- 2026-11-01 advance in-progress/review → approval
