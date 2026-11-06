---
id: 0014-grid-foundation
status: done
scope: [crates/box-core/]
created: 2026-10-29
source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
depends: [0004-neon-ui-controls, 0005-neon-ui-displays]
---
# Task: grid foundation

## Plan

- [x] `specs/box.md`: what box is, crate map, the cell grid (96 px cells, each module a fixed N×M tile that is its only UI; box draws the tile frame and port nubs, the module fills the body), signals (everything audio-rate `f32`, gates are 0/1), engine rules (acyclic, block-based, swap/return queues), persistence, and the "add a module" recipe. Link from `overview.md`.
- [x] `crates/box-core` (no egui, no nice-plug): `ModuleDescriptor` (stable `kind` id, name, category, `size: (cols, rows)` in cells, named ports with tile side and row, `params: &'static [ParamSpec]`, `create(rate, max_block)`), `ParamSpec` (stable id, name, `Float { min, max, default, taper: Linear | Skew | Log }` / `Bool` / `Choice`, unit, `format`/`parse` fn pointers, smoothing ms), `#[non_exhaustive] PortKind::Audio`.
- [x] `trait Module: Send` — `reset(rate, max_block)` (may allocate), `clear()` (realtime-safe), `tail()`, `process(&mut self, io: &mut BlockIo, params: &ParamValues)` over whole blocks of slices; plus a block `Smoother` helper so modules glide params without per-sample atomics.
- [x] `ParamStore`: one `AtomicU32` (f32 bits) per param per instance, written by the UI and snapshotted once per block into `ParamValues`; defaults and normalize/denormalize from `ParamSpec`.
- [x] Tests on a toy Gain module: descriptor/params round trip, port positions sit on the tile edge, tapers invert, store snapshot matches writes, non-finite writes fall back to defaults, process allocates nothing (rt-audit).

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Chain (re-planned 2026-10-31): **0014 spec + core API** → 0023 engine → 0024 basic modules → 0027 standalone app → 0026 canvas editing → 0025 plugin; 0028 (Bogdan/Kirya as modules) after 0024. 0010-0012 wait on 0024.
- User goals (2026-10-30): "expandability i.e. will be easy to add new modules, so there must be kinda standard interface for it", "performance focused".
- User redirect (2026-10-31): "too much UI variants per module… would be better to have one at start"; a simple grid model that runs "as app without host plugins (easy to debug)"; "grid and fixed sizes for modules, for instance Bogdan can be 2x2 or 3x3"; wires/modulation for plugin modules "on after"; first modules sine, env, output, and a gates generator. Decisions: shipped plugin editors converge onto their tiles later; the app uses cpal for audio; cells are 96 px.
- Still in force (2026-10-30): no host automation of module params in v1; audio-only ports, kept extensible via `#[non_exhaustive] PortKind`; no cycles in v1; the plugin is named **box** (`ppc_box`). The task keeps its id and title.
- Adding a module must stay: one file in `box-modules` (descriptor, DSP, tile view) plus one registry line.
- Module param values never rebuild the graph; only topology changes go through the engine's swap (0023).
- Roman is no longer a dependency; its module joins after 0009.

## Review

- Matches the plan inside scope (`crates/box-core/`, plus the workspace manifest, `specs/box.md` and the overview link).
- Choices beyond the draft:
  - Ports carry an explicit `slot` on their side.
  - `Role::Sink` marks Output, whose L/R outputs are the graph output and aren't drawn.
  - Per-instance `Shared` holds the `ParamStore` and the `Meters` atomics the views read.
  - `BlockIo.connected` tells a module which inputs are wired.
  - `geometry` holds the 96 px cell, 2 px inset and 18 px header math, so the canvas and the tests share one source.
- `ModuleDescriptor::validate` lets each registry test its entries.
- 19 tests pass, including the rt-audit run on the toy Gain.

## Summary

box foundation: `specs/box.md` and `box-core`, the standard module interface (descriptor with cell size and ports, param specs, `Module` trait, lock-free param store) every box module implements.

## Log

- 2026-10-29 created: grid foundation
- 2026-10-29 set source user request 2026-10-29: grid system must exist before grid plugins; queued after the design system and the three plugins (kirya, bogdan, roman)
- 2026-10-29 scope add crates/grid-dsp/ — graph runtime crate and the grid plugin wrapper; specs and Cargo/truce files are always-allowed scope
- 2026-10-29 scope add plugins/grid/ — graph runtime crate and the grid plugin wrapper; specs and Cargo/truce files are always-allowed scope
- 2026-10-29 set grade huge
- 2026-10-29 set depends 0004-neon-ui-controls, 0005-neon-ui-displays, 0008-roman-dsp-biquad
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays
- 2026-10-30 scope rm crates/grid-dsp/ — renamed to box; engine moves to 0023
- 2026-10-30 scope rm plugins/grid/ — wrapper moves to 0025
- 2026-10-30 scope add crates/box-core/ — module API crate
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 set grade ""
- 2026-10-31 draft approved: let's plan to implement basic version of grid to have an idea — Full chain as drafted; Allow moving tiles too; plan approved (user, 2026-10-31)
- 2026-10-31 advance draft → ready
- 2026-10-31 scope approved: crates/box-core/
- 2026-10-31 advance ready → in-progress/implement
- 2026-10-31 advance in-progress/implement → in-progress/verify
- 2026-10-31 advance in-progress/verify → in-progress/review
- 2026-10-31 advance in-progress/review → approval
- 2026-11-01 approved: Wow it looks good and feels good! — It worked, close (user, 2026-11-01)
- 2026-11-01 advance approval → done
