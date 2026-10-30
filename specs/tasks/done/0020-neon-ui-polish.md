---
id: 0020-neon-ui-polish
status: done
scope: [specs/design.md, crates/neon-ui/]
created: 2026-10-30
source: user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI
grade: huge
depends: [0019-neon-ui-snapshot-cli]
---
# Task: neon ui polish

## Plan

- [x] Primitives: `surface::gradient_rect` (rounded-rect fan mesh, colors by y, hairline over the edge), top-lit `panel_lit`/`surface_lit`/`raised_lit`, soft inner shadow on `inset`; `text::tracked` for letter-spaced caps.
- [x] Font: Inter Medium + SemiBold (OFL, `OFL.txt` alongside) under `assets/fonts/`; `fonts::install(ctx)` with egui fonts as fallback, `fonts::heading(size)`; installed in showcase and snapshot (Bogdan adopts it in 0006).
- [x] Knob v2, footprint unchanged: 4 px track and glowing value arc, top-lit body with bezel, pointer line, plain value readout (pill opt-in), tracked caps label; hover, pressed/drag, and single-dimmed disabled looks.
- [x] Bindings: knob readouts format through the param's value-to-string with units ("-0.10 dB", "1.00 kHz").
- [x] Buttons v2: default/hover/pressed/disabled ladder; `selectable_button` (blue gradient, cyan glow) for the sketch's Active; pill `switch`; icon buttons on the same ladder.
- [x] Tabs v2: Surface strip with dividers, deep-blue gradient active segment with glowing bar, per-segment hover and pressed, readable disabled state; size helpers unchanged.
- [x] Slider handle and track per the sketch; showcase, snapshot states strips (sliders added), before/after renders linked in Review; `specs/design.md` updated (font + license, lit surfaces, state ladder).

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `cargo run -p neon-ui --example snapshot -- all`

## Manual checks

- [x] Snapshot scenes reviewed against `specs/design-sketch.png` by the user. — waived by user: "Waive and close"

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

Split 2026-10-30 (user): this task is foundation + controls; meters, header, graph well, and cards moved to 0021. Bogdan's editor is out of scope; 0006 consumes the polished kit. Inter is OFL 1.1: free for commercial and subscription products; ship `OFL.txt` and credit it, never sell the font alone. Each step is verified by re-rendering its snapshot scene (0019).

## Review

- Scope held: `crates/neon-ui/` (surface, new text and fonts modules, glow, knob, bindings, button, tabs, slider, examples, `assets/fonts/`) and `specs/design.md`. Spec-only edit outside: a note in 0006 about two Bogdan param quirks the unit readouts expose.
- Primitives: `surface::gradient_rect`/`gradient_mesh` (fan mesh over the rounded outline, colors by height), `panel_lit`/`surface_lit`/`raised_lit`, `lift`, inset inner shadow; `text::tracked`; `glow::glow_line_with` for thick strokes. No new hex: shades derive from palette colors.
- Font: Inter 4.1 Medium + SemiBold from the official release, `OFL.txt` alongside (no Reserved Font Name). Covers the kit's glyphs including "●", so the showcase tofu is gone. ~820 KB of font data per binary that installs it.
- Knob v2 keeps `KnobSize::footprint()`; `KnobParams` gained `pill` (default false). Disabled widgets use Muted, not Ghost, because `add_enabled_ui` already fades them.
- Bindings: `param_knob_with` readouts now come from `normalized_value_to_string(_, true)`. In-host this changes Bogdan's knobs immediately (units shown); Detail will read "kHz Hz" and Amount "1.00 %" until 0006 fixes those params (noted there).
- Buttons: shared `ButtonState` ladder, new `selectable_button`, `switch`, public `paint_body`/`active_fill` for later chrome. Label galleys use the placeholder color so state colors apply.
- Tabs: Surface strip, dividers, deep-blue active segment, per-segment hover/press; horizontal height 22→26 and `tabs_width` widened for Inter (Bogdan uses only the vertical stacks, whose sizing is unchanged).
- Slider: round rimmed handle, glowing gradient fill; also fixed the range variant, which filled from the track start instead of between the thumbs.
- Snapshot: fonts installed, `sliders` states strip, Active = `selectable_button`, tab cursor targets an unselected segment.
- Revision (user: not enough padding around texts in the Bogdan preview): the preview reserved 82 px for an 86 px knob row inside a fixed 216 px panel, so labels touched the panel edge and arcs crowded the well. Its height now derives from `KnobSize::footprint()` plus 12 px gaps.
- Before/after renders: `target/snapshots-before/` vs `target/snapshots/` (not committed). Select, meters, header, graph well, and cards are unchanged here — they are 0021.

## Summary

neon-ui foundation and controls polished to the Dark Neon sketch: lit surfaces, Inter font, knob v2, button/tab/slider state ladders, unit readouts.

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
- 2026-10-30 set grade huge
- 2026-10-30 draft approved: plan for 0020 approved in session (split in two, Inter font)
- 2026-10-30 advance draft → ready
- 2026-10-30 scope approved: specs/design.md, crates/neon-ui/
- 2026-10-30 advance ready → in-progress/implement
- 2026-10-30 baseline: prek.toml@1aaf4748f62f
- 2026-10-30 advance in-progress/implement → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 approved: Looks great now — Waive and close (user, 2026-10-30)
- 2026-10-30 advance approval → done
