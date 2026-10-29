---
id: 0014-grid-foundation
status: draft
scope: [crates/grid-dsp/, plugins/grid/]
created: 2026-10-29
source: user request 2026-10-29: grid system must exist before grid plugins; queued after the design system and the three plugins (kirya, bogdan, roman)
grade: huge
depends: [0004-neon-ui-controls, 0005-neon-ui-displays, 0008-roman-dsp-biquad]
---
# Task: grid foundation

## Plan

- [ ] `specs/grid.md`: module contract (Copy settings in / frame out, reset, latency), audio and control ports, graph topology rules, canvas interaction model, persistence format.
- [ ] `crates/grid-dsp`: module graph runtime — topological execution, allocation-free block processing, graph edits applied through a build-and-swap at block boundaries (igorek's IR swap pattern).
- [ ] Module adapters for the existing DSP crates: Bogdan clipper, Kirya reverb, Roman biquad — the first module library.
- [ ] `plugins/grid` Truce wrapper: custom state persistence for the graph, master gain, bypass, `rt-paranoid` zero-allocation test.
- [ ] Canvas editor on neon-ui: pan and zoom, module placement, compact/full module headers per design.md, port drag-to-patch, selection and delete.
- [ ] House-rule hygiene: `finite_or` at every module boundary, graph output sanitised, allocation only in reset/build paths.
- [ ] Tests: graph processing matches directly-chained processing; non-finite input stays finite; state save/load round-trip; block-swap never tears down mid-block.
- [ ] Update `specs/overview.md` architecture and the roadmap.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Manual checks

- [ ] Open the grid plugin in a host: place modules, patch input → filter → reverb → clipper → output, audio flows, no dropouts on re-patch.
- [ ] Graph survives save/reload in the host.

## Context

- Order decided 2026-10-29: design system and the three plugins (Kirya, Bogdan, Roman) first; this task unlocks the parked module drafts 0010-0012, which depend on it.
- Umbrella task by user request; when picked up it may split into ordered smaller tasks (runtime, adapters, canvas, persistence) — decide at draft review.
- Open decision for review: the plugin's display name — suite convention is Slavic names (Bogdan, Kirya, Roman, Igorek); working name is "grid".
- Module headers (compact and full) are specified in [`design.md`](../design.md); igorek's install/swap protocol is the reference for audio-safe graph edits.

## Summary

Grid foundation: a modular canvas plugin — graph runtime, the three existing DSPs as modules, patching UI on neon-ui — unlocking the parked grid modules.

## Log

- 2026-10-29 created: grid foundation
- 2026-10-29 set source user request 2026-10-29: grid system must exist before grid plugins; queued after the design system and the three plugins (kirya, bogdan, roman)
- 2026-10-29 scope add crates/grid-dsp/ — graph runtime crate and the grid plugin wrapper; specs and Cargo/truce files are always-allowed scope
- 2026-10-29 scope add plugins/grid/ — graph runtime crate and the grid plugin wrapper; specs and Cargo/truce files are always-allowed scope
- 2026-10-29 set grade huge
- 2026-10-29 set depends 0004-neon-ui-controls, 0005-neon-ui-displays, 0008-roman-dsp-biquad
