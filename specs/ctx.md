# Current Task Context: neon design system and suite tasks

State: in progress

## Plan

- [x] Design system complete 2026-10-29: 0003 theme, 0015 showcase, 0004 controls, 0005 displays.
- [ ] Migration 0016 (foundation + Bogdan) in approval — awaiting the DAW manual check. Then 0017 Kirya port, 0018 Igorek port + docs sweep.
- [ ] Kit polish before the editor rebuilds: 0019 snapshot CLI done; next 0020 neon-ui polish against the sketch (draft, awaiting approval).
- [ ] Then: 0006 + 0007 editor rebuilds on nice-plug + the neon kit; 0008 → 0009 Roman; 0013 any time; 0014 then 0010-0012 grid modules. Order in [`roadmap.md`](roadmap.md).

## Context

- Task in approval: [`0016`](tasks/0016-nice-migration-foundation.md) — workspace on nice-plug 0.4 / egui 0.37, alloc-audit, neon-ui bindings, audio-harness, Bogdan ported and bundled.
- Kirya and Igorek crates are temporarily out of the workspace members until 0017/0018 port them; their sources still reference truce.
- Style guide: [`design.md`](design.md); the living showcase runs on eframe 0.37 (`cargo run -p neon-ui --example showcase`).
- Igorek closed 2026-10-29 (0001, 0002 archived to CHANGELOG); the release set is Bogdan and Kirya, Roman joins with 0009.

## Next

Load `target/bundled/ppc_bogdan.clap` (or the VST3) in a host — it appears as **ppc_bogdan**, a separate plugin from the legacy truce Bogdan, which stays installed and untouched. Confirm params/audio/editor, then accept 0016; 0017 (Kirya port, as `ppc_kirya`) follows immediately.
