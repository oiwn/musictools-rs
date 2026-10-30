---
id: 0021-neon-ui-displays-chrome
status: draft
scope: [crates/neon-ui/, specs/design.md]
created: 2026-10-30
source: user decision 2026-10-30: split 0020 into controls (0020) and displays + chrome (0021)
depends: [0020-neon-ui-polish]
---
# Task: neon ui displays chrome

## Plan

- [ ] Vertical L/R meter variant with a dB scale (0, -6, -12, -36, -48), peak tick, and clip latch, per the sketch's Meters block.
- [ ] Header v2: centered tracked title, Muted subtitle, a right-side slot for a select, optional A/B, kebab; built on 0020's lit surfaces and fonts. Fold in what Kirya (0007) and Bogdan (0006) now duplicate: the header strip with "?"/"×" help button, the `HELP`-table help screen, column captions, and the gutter legend chips.
- [ ] Graph-well helper: inset with inner shadow, Ghost grid, legend chips (line swatch + caps label) left- or right-aligned; returns the plot rect for curves.
- [ ] `card(ui, ..)` container: top-lit Panel, hairline, inner margin, for knob rows and module sections.
- [ ] Select popup alignment fix (see `specs/cleanup.md`), restyled to the 0020 state ladder.
- [ ] Showcase and snapshot scenes updated (meters, header, graph, bogdan); `specs/design.md` updated.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`
- [ ] `cargo run -p neon-ui --example snapshot -- all`

## Manual checks

- [ ] Snapshot scenes reviewed against `specs/design-sketch.png` by the user.

## Context

- Split from 0020 on 2026-10-30: 0020 is foundation + controls, this task is displays + chrome. 0006/0007 depend on this one.
- 0006 needs the vertical meters, header, and graph well; keep APIs plain-value and framework-free like the rest of the kit.

## Summary

neon-ui displays and chrome polished to the sketch: vertical meters, header v2, graph well, cards.

## Log

- 2026-10-30 created: neon ui displays chrome
- 2026-10-30 set source user decision 2026-10-30: split 0020 into controls (0020) and displays + chrome (0021)
- 2026-10-30 set depends 0020-neon-ui-polish
- 2026-10-30 scope add crates/neon-ui/ — meters, header, graph well, cards in the kit; style guide notes
- 2026-10-30 scope add specs/design.md — meters, header, graph well, cards in the kit; style guide notes
