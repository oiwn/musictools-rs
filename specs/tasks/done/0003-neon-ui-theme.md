---
id: 0003-neon-ui-theme
status: done
scope: [crates/neon-ui/, Cargo.toml]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
---
# Task: neon ui theme

## Plan

- [x] `crates/neon-ui` skeleton: workspace member on the workspace `egui` 0.34 dep, workspace lints (unsafe forbidden). `truce-egui` is deferred to 0004 — nothing in this task uses it.
- [x] `palette` module: five background surfaces, five accents, four text greys, stroke colors, with semantic aliases.
- [x] `dark_neon_visuals()`: egui `Visuals`/`Style` (window and panel fills, widget base colors, spacing, fonts) as the drop-in replacement for `truce_egui::theme::dark()`.
- [x] Surface painters: `panel`, `surface`, `raised`, `inset` rects — fill plus hairline stroke, 2–3 px radius.
- [x] Glow helper: layered strokes/fills of a color at falling alpha; no blur passes.
- [x] Unit tests: palette constants decode to the spec hex values; `dark_neon_visuals()` constructs without panic.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- Style guide: [`design.md`](../design.md) — palette, typography, surfaces are normative there.
- Foundation only: no editor adopts the kit in this task (0006, 0007, and 0009 do).
- Framework gate: decide truce 6.3.0 vs nice-plug (egui 0.37) before 0004 — findings in [`roadmap.md`](../roadmap.md). This task is framework-free, so the decision does not change it.
- egui 0.34 keeps text styles on `Style`, not `Visuals`, so `dark_neon_visuals()` carries fills, widget colors, radii, and shadows only; typography is set per-painter in 0004+, matching how the current editors size text explicitly.

## Review

- Scope held: `crates/neon-ui/` plus the workspace `Cargo.toml` members list; specs edits only (roadmap gate, task amendments).
- Palette is test-locked to the design.md hex values; aliases point at their accents.
- `dark_neon_visuals()` mirrors the `truce_egui::theme::dark()` structure (same fields touched) so it is a signature-level drop-in for `with_visuals`; shadows are off per "no drop shadows, depth from the surface ladder".
- Deviation from the draft wording, documented in Context: egui 0.34 keeps text styles and spacing on `Style`, not `Visuals`, so the visuals carry fills/colors/radii/shadows only; typography is per-painter in 0004+, matching the current editors. `truce-egui` is not a dependency — nothing in this task used it.
- One mid-verify fix: ecolor 0.34 stores premultiplied colors, so the glow alpha test's expected RGB bytes changed, not the helper.

## Summary

neon-ui crate with the Dark Neon palette, egui visuals, surface painters, and glow helper.

## Log

- 2026-10-29 created: neon ui theme
- 2026-10-29 set source user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
- 2026-10-29 scope add crates/neon-ui/ — new shared UI kit crate and workspace membership
- 2026-10-29 scope add Cargo.toml — new shared UI kit crate and workspace membership
- 2026-10-29 draft approved: Plan approved in session 2026-10-29 (plan-mode approval): neon-ui theme foundation, egui-only deps, truce-egui deferred to 0004, framework gate recorded before 0004.
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: crates/neon-ui/, Cargo.toml
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 baseline: .tmuxp.yaml@f3de42ad9eda
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
- 2026-10-29 approved: ok looks good so close the task
- 2026-10-29 advance approval → done
