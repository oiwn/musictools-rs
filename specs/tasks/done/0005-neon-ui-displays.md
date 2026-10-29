---
id: 0005-neon-ui-displays
status: done
scope: [crates/neon-ui/]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
depends: [0003-neon-ui-theme]
---
# Task: neon ui displays

## Plan

- [x] Framework-free displays: no truce or nice-plug imports; meters take plain dB levels, peak and clip state live in egui id storage.
- [x] NeonSelect: Surface pill with a painted chevron; opens a dark popup menu (Panel fill, hairline stroke, accent on hover/selected).
- [x] NeonTabs: segmented control; active segment on Surface with accent underline glow.
- [x] NeonMeter: L/R bars on Inset — gradient fill cyan → yellow → red, peak-hold tick with decay, red clip latch, click resets.
- [x] Grid painters: linear grid, log-frequency grid, spectrum grid at Ghost-alpha lines.
- [x] Header bar: decorative traffic lights, plugin title with Muted subtitle, A/B compare toggle, kebab menu opening the select-style popup.
- [x] Showcase DISPLAYS section (select, tabs, animated meter, grid wells) and the BOGDAN PREVIEW header upgraded to the kit header bar.
- [x] Unit tests: log-decade and dB tick positions, level-to-color mapping with clip threshold, meter fraction mapping, tab hit-test math.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Manual checks

- [x] Showcase DISPLAYS: select opens/closes and picks, tabs switch, meter animates with peak-hold and clip latch (click resets), grids render in their wells — user: "Perfect."
- [x] BOGDAN PREVIEW header reads as the kit chrome per [`design-sketch.png`](../design-sketch.png) — user: "Perfect."

## Context

- Framework decision 2026-10-29: like 0004, this task stays framework-free. `NeonMeter` no longer reads the truce meter store — the post-migration bindings task wires meters to the framework ([`roadmap.md`](../roadmap.md)).
- Popup finding (better than the draft assumed): egui 0.34 already ships the new `egui::Popup` API (`Popup::new` / `toggle_id` / `close_id`, with `popup_below_widget` and the `Memory` popup fns deprecated). The select and kebab use the new API, so 0.37's popup rework costs the kit nothing here.
- Chevron, kebab dots, and traffic lights are painted shapes, not glyphs, so nothing depends on font coverage.
- Widget visuals per [`design.md`](../design.md).

## Review

- Scope held: five new modules (`select`, `tabs`, `meter`, `grid`, `header`) plus the showcase; no framework imports, meters hold plain dB levels with state in egui id storage.
- Pure and tested: `decade_ticks` (1-2-5 pattern), `db_ticks`, `freq_to_x`/`db_to_y` mappings, `curve_point`, `db_to_fraction`, `level_color`, `is_clipped`, `advance_state` (hold/decay/latch, non-finite safety), `segment_at` — 16 new tests, 200 total workspace.
- Tests caught two real bugs before they shipped: `MeterState::default()` started peaks at 0 dB instead of the −60 floor, and NaN levels slipped through the fraction clamp. Both fixed with dedicated tests.
- The meter fill is a smooth vertex-colored gradient mesh (cyan → yellow → red), matching the knob body technique.
- Popup work uses the new-in-0.34 `egui::Popup` API throughout — see the Context finding; no deprecated calls.
- Smoke-run: the showcase launches with DISPLAYS and the upgraded preview header and stays up; the two manual checks remain open.

## Summary

Display and chrome widgets for the neon-ui kit — select, tabs, meters, graph grids, header bar — framework-free, demoed in the showcase.

## Log

- 2026-10-29 created: neon ui displays
- 2026-10-29 set source user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
- 2026-10-29 scope add crates/neon-ui/ — displays and chrome module in the neon-ui kit
- 2026-10-29 set depends 0003-neon-ui-theme
- 2026-10-29 draft approved: Ok let's go 0005-neon-ui-displays.md
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: crates/neon-ui/
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
- 2026-10-29 approved: Perfect. Let's close this task
- 2026-10-29 advance approval → done
