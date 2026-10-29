---
id: 0020-neon-ui-polish
status: draft
scope: [specs/design.md, crates/neon-ui/]
created: 2026-10-30
source: user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI
depends: [0019-neon-ui-snapshot-cli]
---
# Task: neon ui polish

## Plan

- [ ] Gradient and depth primitives: `surface::gradient_rect` (vertical mesh fill with radius), top-lit panel/surface/raised variants, soft inner shadow on inset.
- [ ] Font: bundle Inter (OFL) via `include_bytes!` with its license under `assets/`; `neon_ui::fonts::install(ctx)`; tracked-caps helper for titles.
- [ ] Knob v2: ~4 px value arc with `glow_line`, visible track, pointer needle, bezel ring plus domed body; grey disabled state; readout as plain value text with units, inset pill only for editable entry.
- [ ] Buttons and tabs v2: active = blue→cyan gradient fill, glow, bright stroke; pressed and disabled states per sketch; vertical tabs share the active look; disabled options stay readable.
- [ ] Header and graph well: tracked title, Muted subtitle, select slot in `header_bar`; graph-well helper (inset, Ghost grid, legend chips with line swatches).
- [ ] Panel card container (`card(ui, ..)`) for knob rows.
- [ ] Showcase scenes updated; before/after snapshots attached here; `specs/design.md` updated where implementation changed (font, gradients).

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

Gaps from the in-host ppc_bogdan shot (Serum 2, 2026-10-30) vs the sketch:

- **Knobs** — thin 2.5 px arc, no glow, no pointer, flat body, track almost invisible. Sketch: thick glowing arc, bright pointer, domed body with rim.
- **Readouts** — boxed mono pills ("1000.00") look clunky. Sketch: plain value text with units ("-0.1 dB", "1.03 kHz"), label below.
- **Buttons/tabs** — active is a thin bar on a flat fill; disabled tabs nearly invisible. Sketch: gradient fill, glow, bright stroke.
- **Typography** — egui's default font is thin. Sketch: heavier humanist sans (Inter-like), tracked caps titles.
- **Header** — plain text plus "?". Sketch: traffic lights, tracked title, subtitle, select, kebab. `header::header_bar` exists; the work is styling.
- **Graph well** — no grid, grey center line. Sketch: framed inset, Ghost grid, legend chips. `grid::paint_linear` exists but is unused.
- **Depth** — controls float on bare BG. Sketch: Panel cards with hairlines and top-lit gradients.
- No gradient helper exists; only `knob::body_mesh` builds a mesh by hand.
- From the first 0019 snapshots: "pressed" draws the same as "hover" on buttons, knobs, and tabs (no pressed look); hovering a vertical tab changes nothing. The default font has no "●" (U+25CF), so that showcase icon button draws a tofu box. Pick the bundled font with the kit's glyphs in mind.

Bogdan's editor is out of scope; 0006 consumes the polished kit. Each step is verified by re-rendering its snapshot scene (0019).

## Summary

neon-ui widgets polished to match the Dark Neon sketch: gradients, Inter font, knob v2, active states, header and graph well, cards.

## Log

- 2026-10-30 created: neon ui polish
- 2026-10-30 set source user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI
- 2026-10-30 scope add crates/neon-ui/src/ — widget polish, bundled font, showcase scenes, style guide implementation notes
- 2026-10-30 scope add crates/neon-ui/assets/ — widget polish, bundled font, showcase scenes, style guide implementation notes
- 2026-10-30 scope add crates/neon-ui/Cargo.toml — widget polish, bundled font, showcase scenes, style guide implementation notes
- 2026-10-30 scope add crates/neon-ui/examples/ — widget polish, bundled font, showcase scenes, style guide implementation notes
- 2026-10-30 scope add specs/design.md — widget polish, bundled font, showcase scenes, style guide implementation notes
- 2026-10-30 set depends 0019-neon-ui-snapshot-cli
- 2026-10-30 scope add crates/neon-ui/ — kit-wide polish
- 2026-10-30 scope rm crates/neon-ui/src/ — covered by crates/neon-ui/
- 2026-10-30 scope rm crates/neon-ui/assets/ — covered by crates/neon-ui/
- 2026-10-30 scope rm crates/neon-ui/Cargo.toml — covered by crates/neon-ui/
- 2026-10-30 scope rm crates/neon-ui/examples/ — covered by crates/neon-ui/
