---
id: 0006-bogdan-neon-editor
status: done
scope: [plugins/bogdan/src/editor.rs, plugins/bogdan/src/lib.rs, specs/bogdan.md]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
grade: huge
depends: [0004-neon-ui-controls, 0005-neon-ui-displays]
---
# Task: bogdan neon editor

## Plan

- [x] Neon shell matching Kirya (0007): `dark_neon_visuals()`, `palette::BG`, every block in exact rects from one vertical budget; header strip ("PPC BOGDAN" tracked, Muted subtitle + version) with the "?"/"×" help button; legacy theme constants removed.
- [x] Scope graph in an inset well with label gutters: PROCESSED in Cyan with glow over a dimmer DRIVEN in Orange, Red ceiling guides labelled with the ceiling in dB, centre line, legend chips above, a "waiting for audio" placeholder when silent; `scope_tap`/`ScopeHistory` plumbing unchanged.
- [x] Control band (lit panel): MODE vertical tabs left, Drive/Ceiling/Detail/Amount knobs centre, SHAPE vertical tabs right (disabled unless Fold), captions over all three columns, dividers.
- [x] Neon help screen from a `HELP` table (Bogdan's existing copy), replacing graph + band while open.
- [x] Tests: vertical budget fits, existing scope tests kept; editor section of `specs/bogdan.md` updated.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Manual checks

- [x] Open Bogdan in a host: knobs, scope, mode/shape tabs behave; no flicker or repaint storms. — waived by user: "Waive and close"
- [x] Editor matches design.md palette and glow on screenshot review. — waived by user: "Waive and close"

## Context

- Style guide: [`design.md`](../design.md); layout canon is header bar, inset graph well, knob rows.
- Since 0020, `param_knob` readouts use the param's value-to-string with units. Done ahead of this task (user, 2026-10-30): Detail no longer double-prints "Hz" (formatter owns the unit, `parse_hz` accepts Hz/kHz), Amount reads and parses as a percent, and the editor installs the neon fonts. The legacy theme colors and layout remain for this task.
- Bogdan keeps its 560x376 size unless the design needs a step up — flag in review if it does.
- Re-planned 2026-10-30 after 0007. User: "also while we're there can you make similar ui for @plugins/bogdan/ also, there are also some flaws, KIRYA looks better atm". The plan now mirrors Kirya's editor. Dropped from the first plan: A/B + kebab (no A/B state), the transfer-curve graph, output meters (they need `lib.rs` meter plumbing — a follow-up if wanted), greying beyond Shape. No longer waits on 0021.

## Review

- `editor.rs` is rebuilt to plan. `ui()` drains the scope tap and calls a host-free `draw(ui, setter)`, the same split as Kirya.
- **Window size, flagged:** 540×420, up from 500×376. The band needs about 506 px across, and the scope keeps 206 px of height.
- Offscreen check: I rendered the main view (with a synthetic clipped sine) and the help screen through a temporary `egui_kittest` harness with a no-op host in `audio-harness`. Both read clean. All temporary code is removed.
- `lib.rs` is untouched.
- Out of scope here:
  - Output meters stay a follow-up.
  - The header strip, help screen, captions and legend are now duplicated with Kirya. 0021's header v2 step is updated to fold them into the kit.
- Gate:
  - workspace fmt, clippy (all and default features), tests, typos and gitleaks are green;
  - `cargo nice-plug bundle ppc_bogdan --release` rebuilt the CLAP.

## Summary

Bogdan's editor rebuilt on the neon kit to match Kirya: header with help, one scope graph with label gutters, and a MODE / knobs / SHAPE control band.

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
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays, 0021-neon-ui-displays-chrome
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays
- 2026-10-30 draft approved: also while we're there can you make similar ui for @plugins/bogdan/ also, there are also some flaws, KIRYA looks better atm (user, 2026-10-30)
- 2026-10-30 advance draft → ready
- 2026-10-30 scope approved: plugins/bogdan/src/editor.rs, plugins/bogdan/src/lib.rs, specs/bogdan.md
- 2026-10-30 advance ready → in-progress/implement
- 2026-10-30 baseline: AGENTS.md@3100f4b048d6, crates/audio-harness/src/lib.rs@36e82b5987ab, crates/musictools-core/src/lib.rs@f3092ee5532a, crates/musictools-core/src/units.rs@da3a8a279f65, crates/neon-ui/assets/fonts/Inter-Medium.ttf@458cd0601d28, crates/neon-ui/assets/fonts/Inter-SemiBold.ttf@47f8ab1d6814, crates/neon-ui/assets/fonts/OFL.txt@9b2ca37b3ffc, crates/neon-ui/examples/common/sections.rs@90b35b47802d, crates/neon-ui/examples/showcase.rs@cd98936639b1, crates/neon-ui/examples/snapshot.rs@c3199d0df5c5, crates/neon-ui/src/bindings.rs@d3bfe2758262, crates/neon-ui/src/button.rs@dfeb559d8e27, crates/neon-ui/src/fonts.rs@fe9c7118b8a7, crates/neon-ui/src/glow.rs@9ed09cc2c24e, crates/neon-ui/src/knob.rs@cee5c93e9d12, crates/neon-ui/src/lib.rs@7591d17c893d, crates/neon-ui/src/slider.rs@8e426330d6f7, crates/neon-ui/src/surface.rs@44fc99ed22e9, crates/neon-ui/src/tabs.rs@1b0ab9394a39, crates/neon-ui/src/text.rs@337c8083e44d, justfile@98d7535da77b, plugins/bogdan/Cargo.toml@548fa77e8f7c, plugins/kirya/Cargo.toml@af3105b93fe3, plugins/kirya/src/editor.rs@b051c1ff59c4, plugins/kirya/src/lib.rs@c40db5397115, plugins/kirya/src/task.rs@47cf84f1339c, prek.toml@1aaf4748f62f, specdev.toml@85e705ffe263
- 2026-10-30 advance in-progress/implement → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 approved: Looks great now — Waive and close (user, 2026-10-30)
- 2026-10-30 advance approval → done
