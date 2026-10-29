---
id: 0006-bogdan-neon-editor
status: draft
scope: [plugins/bogdan/src/editor.rs, plugins/bogdan/src/lib.rs, specs/bogdan.md]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
grade: huge
depends: [0004-neon-ui-controls, 0005-neon-ui-displays, 0020-neon-ui-polish]
---
# Task: bogdan neon editor

## Plan

- [ ] Switch to `dark_neon_visuals()`; header bar (BOGDAN, subtitle detail-preserving clipper) with A/B and kebab menu.
- [ ] Transfer-curve graph on the neon inset well: DRIVEN and PROCESSED curves, CEILING line in Red, ALL-CAPS legend chips.
- [ ] Oscilloscope pane restyled on the same grid painters; `scope_tap` plumbing unchanged.
- [ ] Knob rows become NeonKnobs (M primary, S trim); grey out controls the current Mode ignores, per the pattern in `ideas.md`.
- [ ] Output meters: `#[meter]` fields on `BogdanParams`, `set_meter` calls in `process`, NeonMeter L/R in the editor.
- [ ] Keep the 16 ms timed repaint; rt-paranoid zero-allocation test still passes.
- [ ] Update the editor section of `specs/bogdan.md`; attach a screenshot to this task.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Manual checks

- [ ] Open Bogdan in a host: knobs, scope, and meters behave; no flicker or repaint storms.
- [ ] Editor matches design.md palette and glow on screenshot review.

## Context

- Style guide: [`design.md`](../design.md); layout canon is header bar, inset graph well, knob rows.
- Bogdan keeps its 560x376 size unless the design needs a step up — flag in review if it does.

## Summary

Bogdan's editor rebuilt on the neon-ui kit with output meters.

## Log

- 2026-10-29 created: bogdan neon editor
- 2026-10-29 set source user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
- 2026-10-29 scope add plugins/bogdan/src/editor.rs — editor rebuild on neon-ui; meter fields in params; spec editor section
- 2026-10-29 scope add plugins/bogdan/src/lib.rs — editor rebuild on neon-ui; meter fields in params; spec editor section
- 2026-10-29 scope add specs/bogdan.md — editor rebuild on neon-ui; meter fields in params; spec editor section
- 2026-10-29 set depends 0004-neon-ui-controls
- 2026-10-29 set depends 0005-neon-ui-displays
- 2026-10-29 set grade huge
- 2026-10-29 set depends 0004-neon-ui-controls, 0005-neon-ui-displays
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays, 0020-neon-ui-polish
