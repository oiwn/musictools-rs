---
id: 0024-box-module-library
status: done
scope: [crates/box-modules/, crates/neon-ui/src/]
created: 2026-10-30
source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
depends: [0023-box-engine]
---
# Task: box module library

## Plan

- [x] `crates/box-modules` skeleton: one file per module (descriptor, egui-free DSP, the single tile view), the `MODULES` registry with stable `kind` ids, and `param_*` helpers that bind neon-ui widgets to a `ParamStore` (kit additions go in `neon-ui/src/` if they're reusable).
- [x] **Sine** (1x1): Freq (log 20 Hz–20 kHz) and Level; phase-accumulator sine, out port.
- [x] **Gates** (1x1): Rate (log 0.1–20 Hz) and Width (1–99 %); 0/1 gate out, sample-accurate edges.
- [x] **Env** (2x1): Attack, Decay, Sustain, Release; ports in and gate, out = in × envelope; retriggers on each rising gate edge; a small envelope curve drawn in the tile.
- [x] **Output** (1x1): Level and a peak dot; stereo in; the terminal node the engine sums to the device or host.
- [x] Tests: registered `kind`s are unique; every port sits on its tile; sine frequency and level are exact; gate period and duty match; Env stages hit their targets and retrigger; Output stays finite and below its level; each view fits its N×M tile (layout math); rt-audit across all four.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Third task of the box chain (goals and decisions in 0014). One UI per module (user, 2026-10-31): the tile view is the module's only UI, at 96 px cells.
- Gates replace a manual trigger (user, 2026-10-31: "add gates generator"); a gate is an ordinary 0/1 audio-rate signal, so no new port kind is needed.
- Bogdan and Kirya modules are 0028; Roman's joins after 0009; parked modules 0010-0012 depend on this task.

## Review

- Matches the plan inside `crates/box-modules/`. No neon-ui changes were needed.
- A `ModuleEntry` holds `{descriptor, view, layout}`. The `layout` is the widget rects the view places, so a single test checks that every module's view fits its body.
- Views receive a `Tile { body, header_extra, shared }`. `header_extra` is the header strip to the right of box's label:
  - Env draws its ADSR curve and a level dot there, so the tile stays 2x1 with four knobs.
  - Gates draws a gate lamp there.
- Choices beyond the draft:
  - Env with nothing wired to `in` outputs the envelope itself, so it can also serve as a control source.
  - Output hard-limits its inputs to ±1 before Level, so it can never exceed its setting. It holds the peak with a decay for the dot.
  - Knob readouts use short formats (`12.5k`, `1.50 s`, `-6.0 dB`) so they fit under the small knobs.
- After the 0027 canvas render, Output's knob moved 10 px in from the left edge so the R input nub no longer covers its readout. The gate was re-run.
- 16 tests pass. One of them renders the full Gates → Env ← Sine → Output patch through the engine for 200 blocks with zero allocations.

## Summary

box basic modules: Sine, Gates, Env and Output, each one file with DSP and a fixed-size tile view, registered in `MODULES`.

## Log

- 2026-10-30 created: box module library
- 2026-10-30 set source user request 2026-10-30: grid foundation as "box" — expandable module interface, performance first, compact views on the grid and the full plugin in an inspector
- 2026-10-30 set depends 0023-box-engine
- 2026-10-30 scope add crates/box-modules/ — module adapters, shared panels, ParamAccess
- 2026-10-30 scope add crates/bogdan-ui/ — module adapters, shared panels, ParamAccess
- 2026-10-30 scope add crates/kirya-ui/ — module adapters, shared panels, ParamAccess
- 2026-10-30 scope add crates/neon-ui/src/ — module adapters, shared panels, ParamAccess
- 2026-10-30 scope add plugins/bogdan/src/editor.rs — module adapters, shared panels, ParamAccess
- 2026-10-30 scope add plugins/kirya/src/editor.rs — module adapters, shared panels, ParamAccess
- 2026-10-30 set grade huge
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 scope rm crates/bogdan-ui/ — panel extraction dropped: one UI per module
- 2026-10-31 scope rm crates/kirya-ui/ — panel extraction dropped
- 2026-10-31 scope rm plugins/bogdan/src/editor.rs — plugin editors converge later
- 2026-10-31 scope rm plugins/kirya/src/editor.rs — plugin editors converge later
- 2026-10-31 set grade tiny
- 2026-10-31 set grade ""
- 2026-10-31 draft approved: let's plan to implement basic version of grid to have an idea — Full chain as drafted; Allow moving tiles too; plan approved (user, 2026-10-31)
- 2026-10-31 advance draft → ready
- 2026-10-31 scope approved: crates/box-modules/, crates/neon-ui/src/
- 2026-10-31 advance ready → in-progress/implement
- 2026-10-31 baseline: crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@cf56491f7121, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8, crates/box-dsp/Cargo.toml@9e8da0ead630, crates/box-dsp/examples/box_bench.rs@89d9ac904c87, crates/box-dsp/src/compile.rs@65e1e41d0ac2, crates/box-dsp/src/engine.rs@6571c7427ca0, crates/box-dsp/src/graph.rs@b8ae22d23b78, crates/box-dsp/src/lib.rs@22bd94155ad8, crates/box-dsp/src/patch.rs@c34ae5e50008, crates/box-dsp/src/toys.rs@bd27a16c1de8
- 2026-10-31 advance in-progress/implement → in-progress/verify
- 2026-10-31 advance in-progress/verify → in-progress/review
- 2026-10-31 advance in-progress/review → approval
- 2026-11-01 approved: Wow it looks good and feels good! — It worked, close (user, 2026-11-01)
- 2026-11-01 advance approval → done
