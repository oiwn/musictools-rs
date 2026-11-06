---
id: 0011-grid-gain
status: draft
scope: []
created: 2026-10-29
source: user request 2026-10-29: grid module (parked until grid foundation)
depends: [0024-box-module-library]
---
# Task: grid gain

## Plan

- [ ] Grid module: stereo gain on a NeonKnob M with output meters and power-toggle bypass.
- [ ] Module header per design.md.
- [ ] Parked: scope and crate home are set when the grid foundation (0014) lands.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Context

- Parked at draft by the 2026-10-29 decision ("park as drafts until grid"). Module-header visuals per [`design.md`](../design.md).
- Empty scope is intentional until the grid foundation defines where modules live.

## Summary

Grid gain module: stereo gain knob, meters, power bypass.

## Log

- 2026-10-29 created: grid gain
- 2026-10-29 set source user request 2026-10-29: grid module (parked until grid foundation)
- 2026-10-29 set depends 0014-grid-foundation
- 2026-10-30 set depends 0024-box-module-library
