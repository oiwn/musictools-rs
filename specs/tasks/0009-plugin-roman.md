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

- [ ] `truce.toml`: Roman entry — category effect, fourcc `Rman`, `vst3_subcategory = "EQ"`, bundle id `roman`.
- [ ] `plugins/roman` wrapper: params (Type LP/HP/BP/Notch plus Allpass, Cutoff Hz log-scaled, Q, Slope 12/24 dB/oct, Output trim), sanitised process, reset wiring, `rt-paranoid` zero-allocation test.
- [ ] Neon editor from day one: header (ROMAN, subtitle biquad filter), Type NeonSelect, Cutoff and Q NeonKnobs, Slope segmented tabs, frequency-response curve painted on a neon grid from the current coefficients (UI thread; no audio tap needed).
- [ ] Specs: `specs/roman.md` (description, signal chain, parameter reference, gotchas); add Roman to the `overview.md` list.
- [ ] Build CLAP + VST3 bundles locally: `cargo truce build -p roman --clap --vst3`.

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

Roman: biquad filter plugin — Truce wrapper, params, and a neon editor with a live response curve.

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
