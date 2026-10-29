---
id: 0005-neon-ui-displays
status: draft
scope: [crates/neon-ui/]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
depends: [0003-neon-ui-theme]
---
# Task: neon ui displays

## Plan

- [ ] NeonSelect: Surface pill with chevron; opens a dark popup menu (Panel fill, hairline stroke, accent hover).
- [ ] NeonTabs: segmented control; active segment on Surface with accent underline glow.
- [ ] NeonMeter: L/R bars reading truce meter ids (`#[meter]` fields + `set_meter`), gradient fill, peak-hold tick, red clip latch, click to reset.
- [ ] Grid painters: linear, log-frequency, and spectrum grids at Ghost-alpha lines, plus the inset well frame.
- [ ] Header bar: decorative traffic lights, plugin title with Muted subtitle, A/B toggle, kebab menu.
- [ ] Unit tests: log-grid decade positions; meter level-to-color mapping including the clip threshold.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Context

- Widget visuals per [`design.md`](../design.md). Meters consume the truce meter store the same way `truce_egui::widgets::level_meter` does.

## Summary

Display and chrome widgets for the neon-ui kit: select, tabs, meters, graph grids, header bar.

## Log

- 2026-10-29 created: neon ui displays
- 2026-10-29 set source user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
- 2026-10-29 scope add crates/neon-ui/ — displays and chrome module in the neon-ui kit
- 2026-10-29 set depends 0003-neon-ui-theme
