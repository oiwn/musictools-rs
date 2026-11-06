---
id: 0028-box-plugin-modules
status: draft
scope: [crates/box-modules/]
created: 2026-10-31
source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
depends: [0024-box-module-library]
---
# Task: box plugin modules

## Plan

- [ ] Bogdan module: a fixed tile (3x2 or 3x3, settled against the 96 px cell at draft review) with its knobs, mode and shape, and in/out ports on the tile edges; the DSP wraps `bogdan_dsp::DetailClipper` in a block loop.
- [ ] Kirya module: a larger fixed tile with its knobs and freeze, stereo in/out; the DSP wraps `kirya_dsp::KiryaReverb`.
- [ ] Tests: each module matches its plugin's DSP for the same settings; tiles fit their size; rt-audit.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Context

- Box chain, after the basic modules (goals and decisions in 0014). User, 2026-10-31: plugin modules get their grid extension (wires, later modulation) "on after".
- Converge later (user, 2026-10-31): the shipped ppc_bogdan and ppc_kirya editors stay as they are for now, and are eventually replaced by these tiles shown alone (idea in `ideas.md`).
- Roman's module follows the same pattern once 0009 lands.

## Summary

Bogdan and Kirya as box modules: fixed-size tiles with ports, wrapping their existing DSP.

## Log

- 2026-10-31 created: box plugin modules
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 set depends 0024-box-module-library
- 2026-10-31 scope add crates/box-modules/ — Bogdan and Kirya grid modules
