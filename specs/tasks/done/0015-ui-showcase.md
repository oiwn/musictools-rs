---
id: 0015-ui-showcase
status: done
scope: [crates/neon-ui/]
created: 2026-10-29
source: user request 2026-10-29: host-free app showcasing the neon-ui kit before the nice-plug migration
---
# Task: ui showcase

## Plan

- [x] `crates/neon-ui`: add `eframe` 0.34 as a dev-dependency (example shell only; the kit's runtime deps stay `egui` alone).
- [x] `examples/showcase.rs`: eframe app that installs `dark_neon_visuals()` at startup and lays out a scrollable page on the 8 px grid.
- [x] Palette section: a swatch per color with name and hex, grouped surfaces / accents / text / strokes.
- [x] Surface and glow section: the nested panel > surface/raised > inset ladder, plus glow rect, glow line, and LED circle in each accent.
- [x] Reserved placeholder sections for the 0004/0005 widgets (typography, knobs, sliders, buttons, meters) — the showcase is the living style guide; widget tasks add their sections here.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Manual checks

- [x] `cargo run -p neon-ui --example showcase` opens a window whose palette, surfaces, and glow match [`design-sketch.png`](../design-sketch.png) — user: "ok looks good"

## Context

- Shell is eframe, deliberately neither truce nor nice-plug: the showcase is host-free, so it survives the framework migration untouched except for the eventual egui version bump.
- Framework decision 2026-10-29: migrate to nice-plug after this task lands; 0004+ target nice-plug ([`roadmap.md`](../roadmap.md)).
- Runs on the workspace's egui 0.34 pins today; the migration task bumps egui (and eframe) to 0.37 with everything else.
- Showcases only the public neon-ui API; if a section needs something the kit does not export, add the minimal kit API in this task and note it in Review.

## Review

- Scope held: only `crates/neon-ui/` (example plus eframe dev-dependency); no kit API additions were needed — every section consumes the public palette/visuals/surface/glow API from 0003.
- eframe 0.34 already uses the new app shape: `App::ui(&mut self, ui, frame)` instead of `update`, and top-level `CentralPanel::show` is deprecated for `show_inside` — a useful preview of the egui-side changes the nice-plug migration will carry.
- One clippy fix: float literals in `Stroke::new` need the `_f32` suffix on this toolchain (house style already does this).
- Smoke-run: the window launches and stays up with no startup panic; the visual comparison against the sketch is the open manual check.

## Summary

Host-free eframe showcase app rendering the neon-ui palette, surfaces, and glow as the suite's living style guide.

## Log

- 2026-10-29 created: ui showcase
- 2026-10-29 set source user request 2026-10-29: host-free app showcasing the neon-ui kit before the nice-plug migration
- 2026-10-29 scope add crates/neon-ui/ — showcase example plus eframe dev-dependency
- 2026-10-29 draft approved: Ok, let's start with 0015-ui-showcase.md first
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: crates/neon-ui/
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 baseline: .tmuxp.yaml@f3de42ad9eda
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
- 2026-10-29 approved: ok looks good so close the task
- 2026-10-29 advance approval → done
