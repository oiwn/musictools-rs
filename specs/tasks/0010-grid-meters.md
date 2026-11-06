---
id: 0010-grid-meters
status: draft
scope: []
created: 2026-10-29
source: user request 2026-10-29: grid module (parked until grid foundation)
depends: [0024-box-module-library]
---
# Task: grid meters

## Plan

- [ ] Grid module: stereo passthrough with L/R peak and RMS metering exposed to the module UI.
- [ ] Module header, compact and full variants per design.md, with power toggle and kebab menu.
- [ ] Meter display per NeonMeter: peak hold, clip latch, click to reset.
- [ ] Parked: scope and crate home are set when the grid foundation (0014) lands; depends already points there.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Context

- Parked at draft by the 2026-10-29 decision ("park as drafts until grid"). The grid direction lives in `roadmap.md`; module-header visuals live in [`design.md`](../design.md).
- Empty scope is intentional until the grid foundation defines where modules live.

## Summary

Grid meters module: stereo passthrough with peak/RMS metering and a module header.

## Log

- 2026-10-29 created: grid meters
- 2026-10-29 set source user request 2026-10-29: grid module (parked until grid foundation)
- 2026-10-29 set depends 0014-grid-foundation
- 2026-10-30 set depends 0024-box-module-library
