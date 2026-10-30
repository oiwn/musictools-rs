---
id: 0017-kirya-nice-port
status: done
scope: [plugins/kirya/, Cargo.toml, Cargo.lock, crates/musictools-core/, plugins/bogdan/src/lib.rs, AGENTS.md, crates/audio-harness/src/lib.rs]
created: 2026-10-29
source: user request 2026-10-29: kirya port to nice-plug after migration foundation
depends: [0016-nice-migration-foundation]
---
# Task: kirya nice port

## Plan

- [x] Shared formatters in `musictools-core`: `parse_hz` (moved from Bogdan), `format_percent`/`parse_percent`, `LogTaper { min, max }` (`to_value`/`to_position`, clamped, non-finite safe); Bogdan uses them unchanged in behaviour.
- [x] Params: `#[id]` snake names; linear and skewed ranges as truce (truce `skewed` is the same `t^f` formula, factors carry over); percent params via `" %"` + shared formatters; the five log params store knob position through `LogTaper` (exact truce taper, user decision); `freeze` BoolParam; `ir_slot`/`ir_generation`/`sample_rate_bits` stay plain fields.
- [x] Plugin + process: `ppc_kirya`, stereo + mono layouts, `configure()` (reset reverb, publish rate, snap smoothers), one settings read per frame, `KeepAlive` when frozen else `Tail(n)`; CLAP export, VST3 behind `vst3`.
- [x] IR task: `RenderIr { sample_rate, request }` as `BackgroundTask`, run by `task_executor`; a request counter skips stale tasks (nice-plug has no coalescing); task tests ported plus a stale-skip test.
- [x] Editor on `NiceEguiApp` with the legacy look: Bogdan's shell (`create_egui_editor`, repaint ping, fonts), `neon_ui::bindings` knobs/toggle, rows from `KnobSize::Medium.footprint()`, `AsyncExecutor` stored for render requests; probe code and its tests unchanged.
- [x] Plugin tests on `audio_harness::Harness` (impulse/tail, mono, non-finite, Tail/KeepAlive, reset, readouts + typed entry + exact log midpoint, zero-alloc sweep under `rt-audit`).
- [x] Workspace member + Cargo.toml (`ppc_kirya`, truce deps removed); `specs/kirya.md` and `AGENTS.md` updated.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `cargo nice-plug bundle ppc_kirya --release`

## Manual checks

- [x] ppc_kirya loads in a host: reverb tail correct, IR re-renders on knob moves (spectrogram updates), freeze works, log knobs' low end feels like truce Kirya. — user: "I touch knobs and all feels good" (layout feedback moved to 0007)

## Context

- Follows 0016's foundation; the port patterns (params, editor shell, test harness) come from the Bogdan port.
- The `sample_rate_bits` workaround stays: nice-plug's editor context also does not carry the sample rate.
- Naming per 0016: package, display name, bundle and ids are `ppc_kirya` / `com.oiwn.ppc_kirya`; the truce Kirya stays installed beside it. CLAP only, VST3 behind `vst3` (as Bogdan). The neon look stays in 0007.
- Log knobs (user, 2026-10-30, "Exact log curve"): nice-plug has no log range and a power curve squashes the low end on 1000:1 spans, so the five log params store position 0–1 and map `hz = min·(max/min)^pos`; exponential smoothing on position equals truce's constant-ratio glide.

## Review

- Diff matches the plan:
  - `musictools-core::units` holds `parse_hz`, `format_percent`/`parse_percent` and `LogTaper`, with tests. Bogdan now uses them and its 9 tests still pass.
  - `ppc_kirya` has the Kirya wrapper, task and editor.
  - `plugins/kirya` is a workspace member.
- One scope addition, `crates/audio-harness/src/lib.rs`: an `automate(param, plain, rate)` helper. nice-plug's param setters are `unsafe`, and the harness is the only crate allowed `unsafe`. It sets the value and retargets the smoother, which is the wrappers' host-automation path. Kirya's tests use it wherever truce called `set_value`.
- Behaviour changes, all intended:
  - Percent readouts print the unit with a space ("55 %"), matching Bogdan.
  - The header reads "PPC KIRYA".
  - The IR request guard drops superseded renders, where truce coalesced them.
  - `Plugin::reset` uses `KiryaReverb::clear()`, so it never reallocates on the audio thread.
  - Reopening the editor drops the old window's spectrogram texture and re-requests a render.
- Known small quirk: the Freeze row is centred from the toggle width measured the frame before, so frame 1 uses an 84 px guess.
- Results:
  - 24 Kirya tests pass. They cover the truce-era readout strings, the exact log midpoint (50% → 632 Hz), typed entry, the stale-request skip, and the zero-alloc sweep under `rt-audit`.
  - The workspace gate is green.
  - The bundle is `.clap` only; `nm` shows `clap_entry` and no VST3 factory.
  - Clippy is clean with default features too.

## Summary

Kirya ported to nice-plug as `ppc_kirya`: exact log-taper knobs, the IR render on the background executor behind a stale-request guard, and the legacy-look editor on NiceEguiApp.

## Log

- 2026-10-29 created: kirya nice port
- 2026-10-29 set source user request 2026-10-29: kirya port to nice-plug after migration foundation
- 2026-10-29 scope add plugins/kirya/ — kirya wrapper and editor port
- 2026-10-29 set depends 0016-nice-migration-foundation
- 2026-10-30 scope add Cargo.toml — workspace member, shared formatters (second consumer), Bogdan switch to them, docs
- 2026-10-30 scope add Cargo.lock — workspace member, shared formatters (second consumer), Bogdan switch to them, docs
- 2026-10-30 scope add crates/musictools-core/ — workspace member, shared formatters (second consumer), Bogdan switch to them, docs
- 2026-10-30 scope add plugins/bogdan/src/lib.rs — workspace member, shared formatters (second consumer), Bogdan switch to them, docs
- 2026-10-30 scope add AGENTS.md — workspace member, shared formatters (second consumer), Bogdan switch to them, docs
- 2026-10-30 draft approved: User approved the 0017 plan via ExitPlanMode (2026-10-30), choosing 'Exact log curve (Recommended)' for the log knobs
- 2026-10-30 advance draft → ready
- 2026-10-30 scope approved: plugins/kirya/, Cargo.toml, Cargo.lock, crates/musictools-core/, plugins/bogdan/src/lib.rs, AGENTS.md
- 2026-10-30 advance ready → in-progress/implement
- 2026-10-30 baseline: crates/neon-ui/assets/fonts/Inter-Medium.ttf@458cd0601d28, crates/neon-ui/assets/fonts/Inter-SemiBold.ttf@47f8ab1d6814, crates/neon-ui/assets/fonts/OFL.txt@9b2ca37b3ffc, crates/neon-ui/examples/common/sections.rs@90b35b47802d, crates/neon-ui/examples/showcase.rs@cd98936639b1, crates/neon-ui/examples/snapshot.rs@c3199d0df5c5, crates/neon-ui/src/bindings.rs@d3bfe2758262, crates/neon-ui/src/button.rs@dfeb559d8e27, crates/neon-ui/src/fonts.rs@fe9c7118b8a7, crates/neon-ui/src/glow.rs@9ed09cc2c24e, crates/neon-ui/src/knob.rs@cee5c93e9d12, crates/neon-ui/src/lib.rs@7591d17c893d, crates/neon-ui/src/slider.rs@8e426330d6f7, crates/neon-ui/src/surface.rs@44fc99ed22e9, crates/neon-ui/src/tabs.rs@1b0ab9394a39, crates/neon-ui/src/text.rs@337c8083e44d, justfile@98d7535da77b, plugins/bogdan/Cargo.toml@548fa77e8f7c, plugins/bogdan/src/editor.rs@3df4e6343672, prek.toml@1aaf4748f62f, specdev.toml@85e705ffe263
- 2026-10-30 scope add crates/audio-harness/src/lib.rs — automate() test helper: nice-plug's param setters are unsafe, and audio-harness is the workspace's only unsafe crate
- 2026-10-30 advance in-progress/implement → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 approved: I touch knobs and all feels good — close 0017, redo the layout in 0007 (user choice 2026-10-30)
- 2026-10-30 advance approval → done
