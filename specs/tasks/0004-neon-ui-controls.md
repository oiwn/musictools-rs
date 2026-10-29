---
id: 0004-neon-ui-controls
status: approval
scope: [crates/neon-ui/]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
depends: [0003-neon-ui-theme]
---
# Task: neon ui controls

## Plan

- [x] Framework-free widget API: controls take plain normalized values (`&mut f64`) and return `egui::Response`; no truce or nice-plug imports in the kit.
- [x] NeonKnob painter + interaction: S/M/L sizes, Raised→Panel radial body, accent arc swept from start to value, value readout, ALL-CAPS label, tick ring with Yellow snap highlight; vertical drag, Shift = fine, double-click = default; drag-to-value math in a pure unit-tested function.
- [x] NeonSlider: inset track, accent fill up to the handle, raised handle; double-thumb range variant.
- [x] NeonButton, NeonToggle, IconButton: Surface fill, hairline stroke, LED state dot (accent on, Ghost off), hover accent glow.
- [x] Value entry: click a readout to edit in place with a dark mono input; Enter commits, click-away cancels.
- [x] Showcase sections: WIDGETS with every control driven by local state, plus a BOGDAN PREVIEW composing them per the layout canon — header text, inset scope-well placeholder, M knob row, toggles — previewing the 0006 rebuild.
- [x] Unit tests: drag-to-value math (normal, fine, clamped), value-entry parse/format round-trip, knob arc sweep bounds.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Manual checks

- [ ] Showcase WIDGETS: knobs drag / Shift-fine / double-click / snap-to-ticks, single and range sliders, toggles, and value entry (Enter commits, click-away cancels) all behave.
- [ ] BOGDAN PREVIEW reads as the 0006 editor shape per [`design-sketch.png`](../design-sketch.png).

## Context

- Framework decision 2026-10-29: the kit stays framework-free. The original draft's truce `PluginContext` wrapper moves to a small post-migration bindings task (nice-plug params); 0005's meter-store coupling gets the same treatment at its review ([`roadmap.md`](../roadmap.md)).
- Widget visuals per [`design.md`](../design.md); painters and interaction layers stay separate so the math stays unit-testable without a live UI.
- Typography is painted per the spec (10–11 px ALL-CAPS labels, mono readouts) — egui 0.34 keeps text styles off `Visuals`, so widgets size text explicitly like the current editors.
- Value entry prefers an inline edit over a popup: the egui 0.37 popup API rework would churn a popup-based one at migration.
- Runs before the nice-plug migration by user decision (design first, then migrate).

## Review

- Scope held: four new modules in `crates/neon-ui` (`knob`, `slider`, `button`, `value_entry`) plus the showcase; no framework imports anywhere in the kit.
- All interaction math is pure and tested: `drag_value`, `snap_index`, `value_angle`, `arc_points`, `body_mesh`, `pos_to_value`, `nearest_low`, `commit_text`, percent round-trip — 12 new tests, 189 total workspace.
- The knob body is a per-vertex colored fan mesh (`egui::epaint::Vertex` re-exported through egui) — the one place the kit touches epaint types directly.
- 0.34 API notes hit during the build, all future-migration-relevant: `allocate_ui_at_rect` and `allocate_new_ui` are deprecated in favor of `scope_builder`; `Key::NumPadEnter` does not exist; `TextEdit::clip_text` takes a bool.
- One test fix: the snap epsilon boundary (`0.52` vs `0.5`) is float-flaky, so the test uses unambiguous inside/outside values.
- Smoke-run: the showcase launches with the new sections and stays up; interaction and the Bogdan preview composition are the open manual checks.

## Summary

Interactive framework-free controls for the neon-ui kit — knobs, sliders, buttons/toggles, value entry — demoed in the showcase with a Bogdan preview.

## Log

- 2026-10-29 created: neon ui controls
- 2026-10-29 set source user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
- 2026-10-29 scope add crates/neon-ui/ — controls module in the neon-ui kit
- 2026-10-29 set depends 0003-neon-ui-theme
- 2026-10-29 draft approved: Ok, let's go
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: crates/neon-ui/
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 baseline: .tmuxp.yaml@f3de42ad9eda
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
