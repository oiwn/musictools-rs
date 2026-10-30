---
id: 0009-plugin-roman
status: draft
scope: [plugins/roman/, Cargo.toml, specs/roman.md, specs/overview.md]
created: 2026-10-29
source: user request 2026-10-29: Roman biquad filter plugin
depends: [0008-roman-dsp-biquad, 0004-neon-ui-controls, 0005-neon-ui-displays]
---
# Task: plugin roman

## Plan

- [ ] `plugins/roman` nice-plug wrapper, package `ppc_roman`: `ClapPlugin` (id `com.oiwn.ppc_roman`, features AudioEffect + Filter) and a `vst3`-feature-gated `Vst3Plugin` (subcategory Filter/EQ, fresh class id), like Bogdan and Kirya.
- [ ] Params (Type LP/HP/BP/Notch plus Allpass, Cutoff Hz via `musictools_core::LogTaper`, Q, Slope 12/24 dB/oct, Output trim), sanitised process, reset wiring, `rt-audit` zero-allocation test on `audio_harness::Harness`.
- [ ] Neon editor from day one, on the Kirya/Bogdan pattern (header with help, one graph, control band; vertical budget test): frequency-response curve in the graph well from the current coefficients (UI thread; no audio tap needed), Type and Slope tabs, Cutoff/Q/Trim knobs.
- [ ] Specs: `specs/roman.md` (description, signal chain, parameter reference, gotchas); add Roman to the `overview.md` list.
- [ ] `cargo nice-plug bundle ppc_roman --release`; `just deploy ppc_roman` for the host check.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Manual checks

- [ ] Load Roman in a host: cutoff sweeps audibly for LP and HP; slope toggle changes character; the response curve follows the knobs.

## Context

- Third release plugin per the 2026-10-29 decision: the release set is Bogdan, Kirya, Roman (Igorek stays out of releases).
- Editor layout follows the filter section of [`design.md`](../design.md); Allpass stays available in DSP even though the design shows four types.

## Summary

Roman: biquad filter plugin — nice-plug wrapper, params, and a neon editor with a live response curve.

## Log

- 2026-10-29 created: plugin roman
- 2026-10-29 set source user request 2026-10-29: Roman biquad filter plugin
- 2026-10-29 scope add plugins/roman/ — plugin wrapper + neon editor; workspace membership; new spec; overview list
- 2026-10-29 scope add Cargo.toml — plugin wrapper + neon editor; workspace membership; new spec; overview list
- 2026-10-29 scope add specs/roman.md — plugin wrapper + neon editor; workspace membership; new spec; overview list
- 2026-10-29 scope add specs/overview.md — plugin wrapper + neon editor; workspace membership; new spec; overview list
- 2026-10-29 set depends 0008-roman-dsp-biquad
- 2026-10-29 set depends 0004-neon-ui-controls
- 2026-10-29 set depends 0005-neon-ui-displays
- 2026-10-29 set depends 0008-roman-dsp-biquad, 0004-neon-ui-controls, 0005-neon-ui-displays
