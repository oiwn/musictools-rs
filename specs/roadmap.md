# Roadmap

Committed future direction, in rough priority order. Promote items here from `ideas.md` when decided; move items into `ctx.md` when they become the active task.

## Next

- Dark Neon design system: 0004 (controls) next, then 0005 (displays — needs the same framework-free amendment at its review), then a kit polish pass (0019 snapshot CLI → 0020 controls → 0021 displays + chrome, added 2026-10-30 after the first in-host shot looked flat), then the Bogdan (0006) and Kirya (0007) editor rebuilds on nice-plug. Style guide: [`design.md`](design.md); the living version is `cargo run -p neon-ui --example showcase`.
- Framework migration to nice-plug (egui 0.37), decided 2026-10-29 because truce had stalled, is single-maintainer, and pinned egui 0.34: done in 0016–0018 (Bogdan, Kirya; the Igorek wrapper was removed). CI packaging (0013) still follows.
- Roman, the biquad filter plugin: `roman-dsp` (0008) then the nice-plug wrapper with a neon editor (0009).
- CI packaging for macOS, Linux, Windows (0013).

The queue lives in `specs/tasks/_index.md`; `ctx.md` tracks the active task.

## Later

- Grid system, Bitwig-grid style: a canvas of modules with ports and patching — task 0014 (grid foundation), queued after the design system and the three plugins. Module headers (compact and full, in [`design.md`](design.md)) are the seed. Parked module drafts: 0010 meters, 0011 gain, 0012 mix4 — they depend on 0014 and activate when it lands.
- Add further `oiwn` plugins as sibling workspace crates, extracting shared infrastructure only when a second concrete consumer requires it.
- Igorek stays out of the release set (decision 2026-10-29); its plugin wrapper was removed with truce (2026-10-30) and only `igorek-dsp` remains. A revival writes a fresh nice-plug wrapper with a neon, Kirya-style editor. Its frozen futures — Gardner partitioning, true-stereo IR matrix, drag-and-drop WAV loading — revive only if it returns.
