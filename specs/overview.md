# Project Overview

Workspace architecture and the rules that apply across every plugin. Each plugin's own description, signal chain, parameter reference, gotchas and links live in its own spec:

- [`bogdan.md`](bogdan.md) — detail-preserving clipper and wavefolder.
- [`kirya.md`](kirya.md) — Dattorro plate reverb with a live IR probe.
- [`igorek.md`](igorek.md) — two-stage convolver for percussion (DSP crate only; the wrapper was removed).
- [`design.md`](design.md) — Dark Neon UI style guide behind the shared neon-ui kit.
- [`nice_overview.md`](nice_overview.md) — nice-plug 0.4 cheat-sheet: params, process, background tasks, state, editors, tests, bundling.

Plugin naming (decided 2026-10-29): the nice-plug generation ships as separate, coexisting plugins — `ppc_bogdan`, `ppc_kirya`, and `ppc_roman` when it lands — in display name, package name, bundle file name, and identity (`com.oiwn.ppc_*` CLAP ids, fresh VST3 class ids). The legacy truce-built plugins keep their names and ids untouched so existing projects and installs are unaffected.

## Architecture

The repository is a nice-plug Cargo workspace (migrated from Truce in 0016–0018: Bogdan, then Kirya; the Igorek wrapper was dropped instead of ported) designed to host a suite of audio plugins. Shared dependency versions live in the virtual workspace root.

- **`crates/musictools-core`** — framework-independent, suite-wide utilities: identity constants, `finite_or`, `format_hz`, and `units` (Hz/percent text, the exact `LogTaper`).
- **`crates/bogdan-dsp`** — Bogdan's DSP, tests, offline examples.
- **`crates/kirya-dsp`** — Kirya's reverb and its offline IR analysis (feature `analysis`, default on), tests, examples.
- **`crates/igorek-dsp`** — Igorek's partitioned convolution engine, room IR generator, IR baking; tests, examples. Kept without a plugin wrapper (removed 2026-10-30); a revival writes a fresh nice-plug one.
- **`crates/neon-ui`** — the Dark Neon widget kit; param bindings behind its `nice-plug` feature.
- **`crates/audio-harness`** — checked-buffer harness for running plugins in tests, plus `automate` for moving params like a host; the workspace's only `unsafe`.
- **`plugins/bogdan`, `plugins/kirya`** — nice-plug wrappers and egui editors only.

New plugins belong under `plugins/` with a sibling `crates/<name>-dsp`, and opt into the centrally pinned nice-plug dependencies. Code moves into `musictools-core` only when it is genuinely suite-wide — when a *second* concrete consumer needs it, not in anticipation of one. Plugin-specific DSP stays in its own crate.

The suite targets CLAP and VST3 only. Standalone applications, Audio Units, VST2, LV2, AAX, and other plugin formats are out of scope.

## House rules

These hold for every plugin's DSP crate:

- **Sanitise every public entry point.** Host samples and parameters go through `musictools_core::finite_or` / `finite_or_f64`, and sample rates through a private `safe_sample_rate` guard, so a malformed block cannot leak NaNs or infinities into a feedback path that would then never recover.
- **Allocate only in `reset`.** `process` must be allocation-free. Buffers are sized for the worst case the parameters allow, so a runtime control change only moves indices. Plugins with a large state assert this with a `rt-paranoid` zero-allocation test that also moves the risky parameters.
- **Cache filter coefficients.** Recompute only when the cutoff actually moves, so a static knob costs no `exp()` per sample.
- **One `Copy` settings struct in, one `Copy` frame struct out**, per sample. The frame exposes intermediate signals so editors and tests can see inside the processor without a second code path.
- **Anything off the audio thread reads targets, not smoothed values.** `FloatParam::read()` advances the smoother, which belongs to `process`; background tasks and editor change-detection use `value()`.
- Unsafe Rust is forbidden workspace-wide.

## Licensing

The workspace is MIT. Where a plugin takes its ideas from GPL software, it is written clean-room from published papers and observable behaviour — algorithms and published constants are not copyrightable, but source is. Per-plugin specs record what that means in each case.
