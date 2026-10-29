---
id: 0018-igorek-nice-port-docs
status: draft
scope: [plugins/igorek/, README.md, AGENTS.md, specdev.toml]
created: 2026-10-29
source: user request 2026-10-29: igorek port plus tooling/docs sweep
depends: [0017-kirya-nice-port]
---
# Task: igorek nice port docs

## Plan

- [ ] Params port: `IntParam` discrete → `IntRange::Discrete`, unsmoothed floats, f32→f64 at the boundary.
- [ ] Process port: `read_into` → `smoothed.next_block`, audio-thread task spawns → `context.execute_background` (generation guards replace coalescing), latency → `context.set_latency_samples`.
- [ ] Custom state: snapshot/load hooks → `#[persist]` on a serde type; rebake-on-recall via generation polling (nice-plug has no state-changed callback).
- [ ] Editor port to `NiceEguiApp`; rfd AsyncFileDialog flow unchanged.
- [ ] Docs/tooling sweep: `specs/truce_overview.md` → `specs/nice_overview.md` (written from the ports), truce references in overview.md / design.md / ideas.md / per-plugin specs, README.md + AGENTS.md commands, specdev.toml `always_allowed` (truce.toml → bundler.toml), re-point drafts 0006/0007/0009/0013/0014 at nice-plug.
- [ ] Re-add `plugins/igorek` to workspace members; `cargo nice-plug bundle igorek --release`.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`
- [ ] `cargo nice-plug bundle igorek --release`

## Manual checks

- [ ] Igorek loads in a host: bakes on load, WAV import works, state recall rebakes.

## Context

- Final migration task; completes the workspace on nice-plug/egui 0.37 and cleans every remaining truce reference.
- Igorek is out of the release set (decision 2026-10-29) but stays building.

## Summary

Igorek ported to nice-plug and the repository swept clean of truce: docs, commands, and drafts re-pointed at nice-plug.

## Log

- 2026-10-29 created: igorek nice port docs
- 2026-10-29 set source user request 2026-10-29: igorek port plus tooling/docs sweep
- 2026-10-29 scope add plugins/igorek/ — igorek port and framework docs/tooling sweep
- 2026-10-29 scope add README.md — igorek port and framework docs/tooling sweep
- 2026-10-29 scope add AGENTS.md — igorek port and framework docs/tooling sweep
- 2026-10-29 scope add specdev.toml — igorek port and framework docs/tooling sweep
- 2026-10-29 set depends 0017-kirya-nice-port
