---
id: 0007-kirya-neon-editor
status: draft
scope: [plugins/kirya/src/editor.rs, specs/kirya.md]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
grade: huge
depends: [0004-neon-ui-controls, 0005-neon-ui-displays, 0020-neon-ui-polish]
---
# Task: kirya neon editor

## Plan

- [ ] Switch to `dark_neon_visuals()`; header bar (KIRYA, subtitle plate reverb) with A/B and kebab menu.
- [ ] NeonTabs splitting REVERB and SPECTRUM views.
- [ ] IR decay-envelope pane and spectrogram texture restyled on neon inset grids; `RenderIr` plumbing and debounce unchanged.
- [ ] Knobs, sliders, and selects become neon widgets; grey out Mod controls at depth 0 per the pattern in `ideas.md`.
- [ ] Keep the 33 ms timed repaint and the coalesced background IR task.
- [ ] Update the editor section of `specs/kirya.md`; attach a screenshot to this task.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Manual checks

- [ ] Open Kirya in a host: tabs switch, IR panes re-render after knob changes, no flicker.
- [ ] Editor matches design.md palette and glow on screenshot review.

## Context

- Style guide: [`design.md`](../design.md). Kirya's 720x700 canvas is the largest surface in the suite — the tab split is what keeps it navigable.

## Summary

Kirya's editor rebuilt on the neon-ui kit with REVERB/SPECTRUM tabs.

## Log

- 2026-10-29 created: kirya neon editor
- 2026-10-29 set source user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
- 2026-10-29 scope add plugins/kirya/src/editor.rs — editor rebuild on neon-ui; spec editor section
- 2026-10-29 scope add specs/kirya.md — editor rebuild on neon-ui; spec editor section
- 2026-10-29 set depends 0004-neon-ui-controls
- 2026-10-29 set depends 0005-neon-ui-displays
- 2026-10-29 set grade huge
- 2026-10-29 set depends 0004-neon-ui-controls, 0005-neon-ui-displays
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays, 0020-neon-ui-polish
