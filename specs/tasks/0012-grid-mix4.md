---
id: 0012-grid-mix4
status: draft
scope: []
created: 2026-10-29
source: user request 2026-10-29: grid module (parked until grid foundation)
depends: [0014-grid-foundation]
---
# Task: grid mix4

## Plan

- [ ] Grid module: four stereo inputs mixed to stereo out; per-channel S gain knobs plus mute/solo LED buttons; master trim.
- [ ] Module header, full variant per design.md, sized for the mixer row.
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

Grid mix4 module: four stereo channels to stereo with per-channel gain, mute/solo, master trim.

## Log

- 2026-10-29 created: grid mix4
- 2026-10-29 set source user request 2026-10-29: grid module (parked until grid foundation)
- 2026-10-29 set depends 0014-grid-foundation
