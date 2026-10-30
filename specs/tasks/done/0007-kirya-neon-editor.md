---
id: 0007-kirya-neon-editor
status: done
scope: [plugins/kirya/src/editor.rs, specs/kirya.md]
created: 2026-10-29
source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)
grade: huge
depends: [0004-neon-ui-controls, 0005-neon-ui-displays]
---
# Task: kirya neon editor

## Plan

- [x] Neon shell: `dark_neon_visuals()`, `palette::BG`, fonts; deterministic header strip ("PPC KIRYA" tracked, "plate reverb · v…" Muted) on a lit panel; window 720×540; legacy theme constants removed.
- [x] One combined IR graph in an inset well: spectrogram as a dim backdrop on a new bleak palette-only ramp (peak luma under 40%, no amber/white), L/R envelopes on top with glow (CYAN/ORANGE), labels in gutters (legend + RT60 above, dB left, 100/1k/10k right, time below), envelope trimmed at the floor.
- [x] Control band in exact rects: Dry/Wet knobs left, PLATE/TONE/MOD tabs over a 4-slot knob row centre (MOD centres 3), Freeze toggle right; tab index is editor state.
- [x] Tests: bleak ramp monotonic and capped, vertical budget fits the window, every param in exactly one place, envelope trim; existing probe tests kept.
- [x] Editor section of `specs/kirya.md` updated.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Manual checks

- [x] Open ppc_kirya in a host: tabs switch the knob row, the single graph re-renders after knob changes, nothing clips at the window edges. — waived by user: "Waive and close"
- [x] Editor matches design.md palette and glow on screenshot review. — waived by user: "Waive and close"

## Context

- Style guide: [`design.md`](../design.md).
- Re-planned 2026-10-30 after the 0017 port shipped the legacy layout. User: "looks like a shit. We probably need to group knobs into the tabs and have on graph not 2 separated, if required make freq response more bleak." Decisions: redesign here rather than in 0017; tabs PLATE (Pre-Delay, Size, Diffusion, Decay) / TONE (4 cutoffs) / MOD (Rate, Depth, Shape), with Dry, Wet, Freeze always visible.
- Dropped from the first plan: REVERB/SPECTRUM view tabs (replaced by the above), A/B + kebab (Kirya has no A/B state), greying Mod at depth 0 (stays in `ideas.md`). No longer waits on 0021; its polish lands on the kit later.

## Review

- Revision 1. User: "Everything is AWESOME. One small fix padding from knobs and tabs (PLATE TONE MOD) is visible too low, also need to add help button with some description."
  - The tab-to-knob gap is now 16 px, up from 8. The graph gave up the 8 px, going from 320 to 312.
  - Added a header "?" icon button. It swaps the graph and controls for a scrolling neon help screen (`HELP` table: intro, MIX, PLATE, TONE, MOD, HOLD, TIPS) and shows "×" while open.
  - Both views were re-rendered offscreen and checked; the temporary scaffolding is removed again.
  - Gate green; bundle rebuilt.

- `editor.rs` is rebuilt to plan. `ui()` is now host plumbing around a host-free `draw(ui, setter)`.
- Offscreen check before handing over:
  - What I did: rendered all three tabs through a temporary `egui_kittest` harness with a no-op host in `audio-harness`, then removed both. The finding is recorded in 0022.
  - First render problems: dB labels hidden under the envelope, time labels on the data, the envelope running along the floor edge, and a still-saturated backdrop.
  - Fixes: label gutters, trimming the envelope at the floor, and dimming the ramp to under 40% luma.
  - Second render: clean.
- Not done here:
  - The legend sits top-right in the gutter rather than top-left, to stay clear of the dB labels.
  - L and R mostly overlap, since the tails are near-identical, so orange draws over cyan.
- Gate:
  - 27 Kirya tests;
  - workspace fmt, clippy (all and default features), tests, typos and gitleaks are green;
  - `cargo nice-plug bundle ppc_kirya --release` rebuilt the CLAP.

## Summary

Kirya's editor rebuilt on the neon kit: one combined IR graph with a dim spectrogram behind glowing envelopes, and PLATE/TONE/MOD knob tabs flanked by Dry/Wet and Freeze.

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
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays, 0021-neon-ui-displays-chrome
- 2026-10-30 set depends 0004-neon-ui-controls, 0005-neon-ui-displays
- 2026-10-30 draft approved: User approved the re-planned 0007 via ExitPlanMode (2026-10-30): close 0017, redo in 0007; tabs Plate / Tone / Mod
- 2026-10-30 advance draft → ready
- 2026-10-30 scope approved: plugins/kirya/src/editor.rs, specs/kirya.md
- 2026-10-30 advance ready → in-progress/implement
- 2026-10-30 baseline: AGENTS.md@3100f4b048d6, crates/audio-harness/src/lib.rs@36e82b5987ab, crates/musictools-core/src/lib.rs@f3092ee5532a, crates/musictools-core/src/units.rs@da3a8a279f65, crates/neon-ui/assets/fonts/Inter-Medium.ttf@458cd0601d28, crates/neon-ui/assets/fonts/Inter-SemiBold.ttf@47f8ab1d6814, crates/neon-ui/assets/fonts/OFL.txt@9b2ca37b3ffc, crates/neon-ui/examples/common/sections.rs@90b35b47802d, crates/neon-ui/examples/showcase.rs@cd98936639b1, crates/neon-ui/examples/snapshot.rs@c3199d0df5c5, crates/neon-ui/src/bindings.rs@d3bfe2758262, crates/neon-ui/src/button.rs@dfeb559d8e27, crates/neon-ui/src/fonts.rs@fe9c7118b8a7, crates/neon-ui/src/glow.rs@9ed09cc2c24e, crates/neon-ui/src/knob.rs@cee5c93e9d12, crates/neon-ui/src/lib.rs@7591d17c893d, crates/neon-ui/src/slider.rs@8e426330d6f7, crates/neon-ui/src/surface.rs@44fc99ed22e9, crates/neon-ui/src/tabs.rs@1b0ab9394a39, crates/neon-ui/src/text.rs@337c8083e44d, justfile@98d7535da77b, plugins/bogdan/Cargo.toml@548fa77e8f7c, plugins/bogdan/src/editor.rs@3df4e6343672, plugins/bogdan/src/lib.rs@556a1a63932d, plugins/kirya/Cargo.toml@af3105b93fe3, plugins/kirya/src/lib.rs@c40db5397115, plugins/kirya/src/task.rs@47cf84f1339c, prek.toml@1aaf4748f62f, specdev.toml@85e705ffe263
- 2026-10-30 advance in-progress/implement → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 advance approval → in-progress/fix (revision)
- 2026-10-30 baseline: AGENTS.md@3100f4b048d6, crates/audio-harness/src/lib.rs@36e82b5987ab, crates/musictools-core/src/lib.rs@f3092ee5532a, crates/musictools-core/src/units.rs@da3a8a279f65, crates/neon-ui/assets/fonts/Inter-Medium.ttf@458cd0601d28, crates/neon-ui/assets/fonts/Inter-SemiBold.ttf@47f8ab1d6814, crates/neon-ui/assets/fonts/OFL.txt@9b2ca37b3ffc, crates/neon-ui/examples/common/sections.rs@90b35b47802d, crates/neon-ui/examples/showcase.rs@cd98936639b1, crates/neon-ui/examples/snapshot.rs@c3199d0df5c5, crates/neon-ui/src/bindings.rs@d3bfe2758262, crates/neon-ui/src/button.rs@dfeb559d8e27, crates/neon-ui/src/fonts.rs@fe9c7118b8a7, crates/neon-ui/src/glow.rs@9ed09cc2c24e, crates/neon-ui/src/knob.rs@cee5c93e9d12, crates/neon-ui/src/lib.rs@7591d17c893d, crates/neon-ui/src/slider.rs@8e426330d6f7, crates/neon-ui/src/surface.rs@44fc99ed22e9, crates/neon-ui/src/tabs.rs@1b0ab9394a39, crates/neon-ui/src/text.rs@337c8083e44d, justfile@98d7535da77b, plugins/bogdan/Cargo.toml@548fa77e8f7c, plugins/bogdan/src/editor.rs@3df4e6343672, plugins/bogdan/src/lib.rs@556a1a63932d, plugins/kirya/Cargo.toml@af3105b93fe3, plugins/kirya/src/lib.rs@c40db5397115, plugins/kirya/src/task.rs@47cf84f1339c, prek.toml@1aaf4748f62f, specdev.toml@85e705ffe263
- 2026-10-30 advance in-progress/fix → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 approved: Looks great now — Waive and close (user, 2026-10-30)
- 2026-10-30 advance approval → done
