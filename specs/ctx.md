# Current Task Context: neon design system and suite tasks

State: in progress

## Plan

- [x] Design system complete 2026-10-29: 0003 theme, 0015 showcase, 0004 controls, 0005 displays.
- [x] Migration to nice-plug: 0016 foundation + Bogdan, 0017 Kirya done; 0018 truce removal (Igorek wrapper deleted, docs swept) done 2026-10-30.
- [x] Kit polish and editor rebuilds: 0019, 0020, 0006 Bogdan, 0007 Kirya done 2026-10-30.
- [x] box first slice done 2026-11-01: 0014 core API, 0023 engine, 0024 basic modules, 0027 standalone app.
- [ ] box second round, all in approval (2026-11-01): 0029 folders + pitch, 0026 patching, 0030 module browser, 0031 Clock, 0012 Mix4, 0032 Seq, 0033 Scaler, 0008 roman-dsp, 0034 Filter + demo patch. Manual checks wait on the user in 0026, 0030, 0031, 0012, 0032, 0033, 0034.
- [ ] Also open: 0025 box plugin, 0009 ppc_roman, 0028 Bogdan/Kirya modules, 0021 kit chrome, 0013 CI. Order in [`roadmap.md`](roadmap.md).

## Context

- No task in progress. box model: [`box.md`](box.md); run it with `cargo run -p box-app --release`. nice-plug cheat-sheet: [`nice_overview.md`](nice_overview.md).
- Style guide: [`design.md`](design.md); the living showcase runs on eframe 0.37 (`cargo run -p neon-ui --example showcase`).
- The release set is Bogdan and Kirya; Roman joins with 0009.

## Next

The user plays the demo (`cargo run -p box-app --release`; delete or move `box-patch.json` first if one was saved), checks or waives the manual checks, and accepts or revises the nine tasks.
