---
id: 0019-neon-ui-snapshot-cli
status: done
scope: [crates/neon-ui/Cargo.toml, crates/neon-ui/examples/, AGENTS.md, specs/design.md]
created: 2026-10-30
source: user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI
---
# Task: neon ui snapshot cli

## Plan

- [x] Add dev-dep `egui_kittest` 0.37 (features `wgpu`, `snapshot`) to neon-ui: offscreen render through egui-wgpu (Metal on macOS) to an image.
- [x] Split `examples/showcase.rs` sections into a shared module (`examples/common/sections.rs`), each a plain `fn(&mut Ui, &mut State)`, so the window and the CLI draw the same code.
- [x] New example `snapshot`: `cargo run -p neon-ui --example snapshot -- [scene...] [--out DIR] [--scale 2]`. Scenes: knobs, buttons, tabs, header, graph, meters, bogdan, all (default). Output defaults to `target/snapshots/<scene>.png`.
- [x] Scenes render fixed states plus hover/active/disabled variants (hover forced through kittest pointer events), matching the sketch rows.
- [x] Document the command in `AGENTS.md` and the tooling line of `specs/design.md`.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `cargo run -p neon-ui --example snapshot -- all`

## Suggested checks

- Write a crop of the matching `design-sketch.png` region next to each scene PNG for side-by-side review.

## Context

- Today the showcase is an interactive eframe window only; it cannot save images. Polish (0020) needs headless renders to iterate without a DAW.
- Fallback if kittest's wgpu path fails on this machine: showcase `--screenshot PATH` via `ViewportCommand::Screenshot` → `Event::Screenshot`, saved with the `image` crate.
- Snapshots are a review tool, not pixel-diff tests: GPU output differs across machines.

## Review

- Scope held: `crates/neon-ui/Cargo.toml` (dev-deps `egui_kittest` 0.37 with `wgpu`, `image` 0.25 png-only), `examples/` (new `common/sections.rs`, `snapshot.rs`, showcase rewritten on the shared sections), `AGENTS.md`, `specs/design.md` (new Tooling section). Kit source untouched.
- Sections are fine-grained (`swatches`, `surfaces`, `glow_samples`, `knobs`, `sliders`, `buttons`, `entry`, `select_tabs`, `meters`, `graphs`, `bogdan_preview`); callers own titles and gaps, so both binaries use every shared item and no dead-code allowances are needed.
- 12 scenes; `knobs`, `buttons`, `tabs` also write a `-states` strip stitched from per-cell renders (default, hover, pressed, active, disabled) with pointer events injected through kittest. Active means the widget's own on state: toggle on for buttons, held drag for knobs and tabs.
- Renders are flattened onto BG (egui leaves unpainted pixels transparent), the synthetic cursor is off, and missing glyphs render as tofu rather than panicking — the showcase's "●" icon button is one, recorded for 0020.
- Kit findings from the first renders went to 0020's Context: no distinct pressed look, vertical tabs ignore hover, the tofu glyph.
- The sketch-crop side-by-side stayed a Suggested check; not done.
- Unrelated: `cargo test` warns about an `ir_probe` example-name collision between kirya-dsp and igorek-dsp (pre-existing).

## Summary

Headless PNG snapshots of neon-ui scenes via `cargo run -p neon-ui --example snapshot`.

## Log

- 2026-10-30 created: neon ui snapshot cli
- 2026-10-30 set source user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI
- 2026-10-30 scope add crates/neon-ui/Cargo.toml — kittest dev-dep, snapshot example + shared sections, document the command
- 2026-10-30 scope add crates/neon-ui/examples/ — kittest dev-dep, snapshot example + shared sections, document the command
- 2026-10-30 scope add AGENTS.md — kittest dev-dep, snapshot example + shared sections, document the command
- 2026-10-30 scope add specs/design.md — kittest dev-dep, snapshot example + shared sections, document the command
- 2026-10-30 draft approved: let's do @specs/tasks/0019-neon-ui-snapshot-cli.md
- 2026-10-30 advance draft → ready
- 2026-10-30 scope approved: crates/neon-ui/Cargo.toml, crates/neon-ui/examples/, AGENTS.md, specs/design.md
- 2026-10-30 advance ready → in-progress/implement
- 2026-10-30 baseline: .cargo/config.toml@cf4a8f5c3cc7, .codex/hooks.json@f274ace3f299, crates/audio-harness/Cargo.toml@8b49baef880c, crates/audio-harness/src/lib.rs@91303006c5c9, crates/musictools-core/Cargo.toml@2d1c4dbee0ab, crates/musictools-core/src/lib.rs@befadf55c88a, crates/musictools-core/src/rt_audit.rs@fc4d355149b1, crates/musictools-core/src/scope_tap.rs@06a30fa682ac, crates/neon-ui/src/bindings.rs@be125bc7a067, crates/neon-ui/src/button.rs@bfc9e836e128, crates/neon-ui/src/grid.rs@e110daa66b30, crates/neon-ui/src/header.rs@aa35fc11e2ab, crates/neon-ui/src/knob.rs@dc9b0fdf4f33, crates/neon-ui/src/lib.rs@9c422e5d8249, crates/neon-ui/src/meter.rs@70b3aa2aa90b, crates/neon-ui/src/select.rs@75a781426303, crates/neon-ui/src/slider.rs@9dfb8b5ff358, crates/neon-ui/src/tabs.rs@a79f648143a2, plugins/bogdan/Cargo.toml@03fc0f500ba1, plugins/bogdan/src/editor.rs@38dc8e7d4eeb, plugins/bogdan/src/lib.rs@ebb5cc68e0a6
- 2026-10-30 advance in-progress/implement → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 approved: Looks great let's close it
- 2026-10-30 advance approval → done
