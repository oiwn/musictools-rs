# Current Task Context: neon design system and suite tasks

State: in progress

## Plan

- [x] 0003 neon-ui theme and 0015 showcase: done 2026-10-29, accepted off the showcase window.
- [x] 0004 controls: implemented, verified, in approval — awaiting interaction + Bogdan-preview manual checks.
- [ ] 0005 displays: needs the same framework-free amendment at its review (meter-store coupling defers to the bindings task).
- [ ] Then: nice-plug migration task → small neon-ui bindings task → 0006 + 0007 editor rebuilds on nice-plug; 0008 → 0009 Roman; 0013 any time; 0014 then 0010-0012 grid modules. Decision and order in [`roadmap.md`](roadmap.md).

## Context

- Task in approval: [`0004`](tasks/0004-neon-ui-controls.md) — framework-free knobs, sliders, buttons/toggles, value entry, plus the showcase WIDGETS and BOGDAN PREVIEW sections.
- Style guide: [`design.md`](design.md), transcribed from the sketch (`design-sketch.png`); the living version is `cargo run -p neon-ui --example showcase`.
- Framework decision 2026-10-29, ordered design-first: 0004/0005 framework-free → migrate to nice-plug → neon-ui bindings → editor rebuilds.
- Igorek closed 2026-10-29 (0001, 0002 archived to CHANGELOG); the release set is Bogdan and Kirya, Roman joins with 0009.
- Drafts: 0005 displays, 0006-0007 editor rebuilds, 0008-0009 Roman, 0010-0012 parked grid modules, 0013 CI packaging, 0014 grid foundation (unlocks 0010-0012).

## Next

Run the showcase and check 0004's two manual items (widget interactions, Bogdan preview), then accept 0004. Next draft up: 0005 with its framework-free amendment.
