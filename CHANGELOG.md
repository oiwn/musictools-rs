# Changelog

Completed tasks, moved here from `specs/ctx.md` once every checkbox in their
plan is done. Newest entry at the top, dated. Freeform — one block per finished
task. Coexists with any conventional release notes already here.

<!-- Entry template:
## YYYY-MM-DD — <task title>

- what shipped
- files touched / decisions locked
-->

## 2026-11-01 — box app

- box app: a standalone eframe + cpal app that shows a patch on the cell grid with live module tiles and plays it.
- Task `0027-box-app`; source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)

## 2026-11-01 — box module library

- box basic modules: Sine, Gates, Env and Output, each one file with DSP and a fixed-size tile view, registered in `MODULES`.
- Task `0024-box-module-library`; source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)

## 2026-11-01 — box engine

- box engine: patch model, acyclic graph compiler with buffer reuse, and an allocation-free executor with lock-free plan swaps.
- Task `0023-box-engine`; source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)

## 2026-11-01 — grid foundation

- box foundation: `specs/box.md` and `box-core`, the standard module interface (descriptor with cell size and ports, param specs, `Module` trait, lock-free param store) every box module implements.
- Task `0014-grid-foundation`; source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)

## 2026-10-30 — igorek nice port docs

- Truce removed: the Igorek wrapper deleted (DSP kept), a nice-plug cheat-sheet replaces the truce one, and README, specs and drafts re-pointed at nice-plug.
- Task `0018-igorek-nice-port-docs`; source: user request 2026-10-29: igorek port plus tooling/docs sweep

## 2026-10-30 — kirya neon editor

- Kirya's editor rebuilt on the neon kit: one combined IR graph with a dim spectrogram behind glowing envelopes, and PLATE/TONE/MOD knob tabs flanked by Dry/Wet and Freeze.
- Task `0007-kirya-neon-editor`; source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)

## 2026-10-30 — bogdan neon editor

- Bogdan's editor rebuilt on the neon kit to match Kirya: header with help, one scope graph with label gutters, and a MODE / knobs / SHAPE control band.
- Task `0006-bogdan-neon-editor`; source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)

## 2026-10-30 — neon ui polish

- neon-ui foundation and controls polished to the Dark Neon sketch: lit surfaces, Inter font, knob v2, button/tab/slider state ladders, unit readouts.
- Task `0020-neon-ui-polish`; source: user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI

## 2026-10-30 — nice migration foundation

- Workspace on nice-plug 0.4 / egui 0.37 with the alloc-audit module, neon-ui param bindings, and Bogdan ported end-to-end including bundles.
- Task `0016-nice-migration-foundation`; source: user request 2026-10-29: migrate to nice-plug (egui 0.37) per roadmap decision; three-task plan approved in session

## 2026-10-30 — kirya nice port

- Kirya ported to nice-plug as `ppc_kirya`: exact log-taper knobs, the IR render on the background executor behind a stale-request guard, and the legacy-look editor on NiceEguiApp.
- Task `0017-kirya-nice-port`; source: user request 2026-10-29: kirya port to nice-plug after migration foundation

## 2026-10-30 — neon ui snapshot cli

- Headless PNG snapshots of neon-ui scenes via `cargo run -p neon-ui --example snapshot`.
- Task `0019-neon-ui-snapshot-cli`; source: user request 2026-10-30: in-host Bogdan shot looks unpolished vs design sketch; plan UI component polish with a screenshot CLI

## 2026-10-29 — neon ui displays

- Display and chrome widgets for the neon-ui kit — select, tabs, meters, graph grids, header bar — framework-free, demoed in the showcase.
- Task `0005-neon-ui-displays`; source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)

## 2026-10-29 — neon ui controls

- Interactive framework-free controls for the neon-ui kit — knobs, sliders, buttons/toggles, value entry — demoed in the showcase with a Bogdan preview.
- Task `0004-neon-ui-controls`; source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)

## 2026-10-29 — ui showcase

- Host-free eframe showcase app rendering the neon-ui palette, surfaces, and glow as the suite's living style guide.
- Task `0015-ui-showcase`; source: user request 2026-10-29: host-free app showcasing the neon-ui kit before the nice-plug migration

## 2026-10-29 — neon ui theme

- neon-ui crate with the Dark Neon palette, egui visuals, surface painters, and glow helper.
- Task `0003-neon-ui-theme`; source: user request 2026-10-29: implement Dark Neon design from sketch (specs/design.md)

## 2026-10-29 — clippy toolchain lint cleanup

- Modernized 18 sites for the rustc 1.99 Clippy lints (`_f32` literals and `as_chunks`), restoring a clean `-D warnings` build.
- Task `0002-clippy-toolchain-lint-cleanup`; source: clippy 0.1.99 failures on rustc 1.99 blocking task 0001's acceptance gate

## 2026-10-29 — igorek two stage convolver

- Two-stage convolver: Color and Room stages with baked selection/envelope, a partitioned FFT engine, and a Truce wrapper with an egui editor.
- Task `0001-igorek-two-stage-convolver`; source: specs/ctx.md Igorek design task; landed in commit 0b73ddd

## 2026-08-07 — Kirya follow-ups: IR window, Hz formatting, per-plugin specs

- **Adaptive IR probe window.** The probe rendered a fixed 3 s, so at the long
  decays this plugin exists for the tail never reached the floor and the
  readout degraded to "RT60 > 3 s". The window is now sized from the reverb's
  own tail estimate and snapped to one of `IR_WINDOW_STOPS` (0.5 s to 32 s,
  stepping ~1.6x) with 1.3x headroom. Extracted
  `kirya_dsp::estimated_tail_seconds` so the host's `tail()`, the render task
  and the `ir_probe` example all share one estimate. The editor's time axis is
  now data-driven, with tick spacing chosen per window. Visual only — no DSP
  change. Default settings now measure RT60 5.10 s where the truncated 3 s
  window reported 4.64 s; the dark preset reports 10.21 s instead of nothing.
- **Bounded analysis resolution**, required by the above: analysis hops widen on
  a long render instead of the point count growing without bound. At a fixed
  256-sample hop a 32 s render at 192 kHz would have needed ~24 000 spectrogram
  columns — past the 8192 texture limit common on GPUs, and 24 000 transforms
  per render. `IrRender` now carries the hops it chose. A 3 s / 48 kHz render is
  unchanged.
- **`musictools_core::format_hz`** — Truce formats `ParamUnit::Hz` as whole
  hertz below 1 kHz, which collapsed Mod Rate's whole 0.01–20 Hz range into
  "0 Hz" and "1 Hz". One shared formatter (2 decimals below 10 Hz, 1 below 100,
  integer below 1 kHz, then kHz with 2 decimals) is now wired via
  `#[param(format = ...)]` into Kirya's five Hz params and Bogdan's Detail, so
  both plugins read consistently. Display only; no ids, ranges or defaults
  changed, and automation is untouched.
- **Per-plugin specs.** New `specs/bogdan.md` and `specs/kirya.md` own each
  plugin's description, signal chain, parameter table, gotchas and reference
  links. `specs/overview.md` trimmed to workspace architecture, the crate map,
  and the house rules that apply everywhere. `AGENTS.md` and `README.md` point
  at the new layout.
- Verified: `cargo test --workspace --all-features` green (112 tests), clippy
  clean at `-D warnings`, `rt-paranoid` still 0 allocations. Both plugins
  rebuilt and reinstalled, since the formatter change touches Bogdan too.
  Listening and in-host checks are still outstanding.

## 2026-08-07 — Kirya: Dattorro plate reverb (CLAP + VST3)

- Second plugin in the suite. Clean-room implementation of the reverberator in
  Dattorro 1997 (JAES 45(9)) with ValleyAudio Plateau's control set. Plateau is
  GPL-3.0-or-later; nothing was copied, and `kirya-dsp` stays MIT.
- `crates/kirya-dsp` (new): `constants.rs`, `delay.rs` (`InterpDelay`),
  `allpass.rs`, `filters.rs`, `lfo.rs`, `tank.rs`, `analysis.rs` (feature
  `analysis`, default on, via `realfft`), and `KiryaReverb` / `KiryaSettings` /
  `KiryaFrame` in `lib.rs`. Examples `ir_probe` and `tank_stability`.
- `plugins/kirya` (new): 14 params (ids 0–13, a permanent contract), the
  `RenderIr` background task, and an egui editor with a stacked envelope +
  spectrogram IR probe over three control rows.
- Decisions locked during implementation:
  - **Mod Shape** morphs the tank LFOs from triangle (0%) to sine (100%).
  - The four cutoff params use `log(20)` smoothing rather than `exp(20)`;
    a frequency reads as smooth when it ramps by a constant ratio.
  - LFO phase is carried in `f64`. At the 0.01 Hz end of the rate knob one
    turn is ~4.8 M samples, where an `f32` step is a fraction of an ulp and the
    modulation would quantise into a staircase.
  - `OUTPUT_TRIM = 1.12`, measured: the untrimmed seven-tap sum sat at 0.893x
    the dry RMS at default settings. Wet at 100% now lands within 0.01 dB of
    dry.
  - Host tail time is estimated from the loop length and clamped to 30 s
    (`u32::MAX` while frozen); the raw figure runs to hours at decay 0.9999.
  - RT60 reports nothing rather than a guess when the tail has not fallen
    35 dB inside the render window — otherwise the end of the window, not the
    reverb, sets the slope.
  - **Three** `#[skip]` handoff fields, not the two originally planned:
    `sample_rate_bits: Arc<AtomicU64>` was added because `PluginContext` does
    not carry the host rate, and the IR probe has to render at whatever the
    host is playing or the displayed reverb time is wrong anywhere but 48 kHz.
  - `RenderIr` builds its `Analyzer` per run rather than sharing a scratch
    buffer. `SERIALIZED = true` is kept anyway for a different reason than the
    plan gave: it stops two concurrent renders finishing out of order and
    leaving a stale tail on screen.
  - Editor is 720x700, not the 720x620 planned — the two analysis panels plus
    three control rows do not fit in 620.
- Fixed while building: the tank's damping filters were not cleared by
  `reset`, so a reused instance injected a decaying tail into an empty tank.
- Verified: `cargo test --workspace --all-features` green (77 tests), clippy
  clean at `-D warnings`, `rt-paranoid` 0 allocations across 400 blocks while
  sweeping Size and toggling Freeze, `tank_stability` stable at 44.1/48/96/192
  kHz across every Size / Decay / Diffusion corner, CLAP + VST3 bundles built
  and signed.

## 2026-08-04 — v0.2.0: modes + wavefolder

- Version bumped `0.1.0` → `0.2.0` (workspace root `Cargo.toml`); the editor
  header now shows a `v<version>` build stamp so the loaded bundle is
  identifiable (reinstall with `cargo truce install -p bogdan --clap --vst3
  --user` to refresh the host's copy).
- Added a **Mode** switch — Clean / Detail / Fold:
  - Clean: plain hard clip.
  - Detail: original 1 kHz high-passed-delta rectified duck (DnB-bass voice),
    kept bit-identical via the `process_sample` wrapper.
  - Fold: real antialiased **wavefolder** (first-order ADAA), transparent below
    the ceiling and reflecting above it; **Sine** or **Triangle** shape.
- New params: `Mode` (2), `Detail` Hz cutoff (3), `Amount` depth (4), `Shape`
  (5). Drive sets fold density, Ceiling the amplitude. Fixed the `Amount` `%`
  display (`linear(0,1)`).
- `bogdan-dsp`: `ClipMode`/`FoldShape`/`DetailSettings`, runtime cutoff with a
  cached coefficient, ADAA fold helpers; `finite_or_f64` + `safe_sample_rate`.
- Editor: larger/bolder title, centered control row with margins, Mode/Shape
  dropdowns.
- Verified: `cargo test --all` green, clippy clean, rt-paranoid 0 allocations,
  CLAP + VST3 build.
