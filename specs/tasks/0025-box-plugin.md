---
id: 0025-box-plugin
status: draft
scope: [plugins/box/, justfile]
created: 2026-10-30
source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
depends: [0026-box-canvas-editor]
---
# Task: box plugin

## Plan

- [ ] `plugins/box`, package `ppc_box`: `ClapPlugin` (`com.oiwn.ppc_box`), feature-gated `Vst3Plugin`, stereo and mono layouts; host params Master Gain and Bypass only.
- [ ] Patch persisted through a `#[persist = "patch"]` serde field; recall rebuilds the engine off the audio thread; the UI ↔ audio handoff uses the 0023 swap and return queues.
- [ ] Process: engine run inside `process`, `Tail` from the longest module tail, Bypass passes input through.
- [ ] Editor: the app's canvas (0027/0026) inside nice-plug-egui; the canvas code moves from `box-app` into a shared spot both use (decide at draft review: `box-modules` feature or a `box-ui` crate).
- [ ] An Input module (host audio in) joins the registry so box works as an effect as well as a generator.
- [ ] Tests on `audio_harness::Harness`: Input → Output passes audio through; Gates → Env ← Sine → Output renders; save/load round trip; rt-audit across a re-patch.
- [ ] `cargo nice-plug bundle ppc_box --release`; `just deploy ppc_box`.

## Manual checks

- [ ] Load ppc_box in a host: add and remove modules while audio plays, no dropouts; the patch survives save/reload.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`
- [ ] `cargo nice-plug bundle ppc_box --release`

## Context

- Sixth task of the box chain (goals and decisions in 0014): it comes after the standalone app and canvas editing (user, 2026-10-31). No host automation of module params in v1 (user, 2026-10-30).
- `#[persist]` is unused in the repo so far; verify its API first (`specs/nice_overview.md` flags it).

## Summary

ppc_box: the box plugin wrapper hosting the app's canvas, with persisted patches, master gain, bypass and a host Input module.

## Log

- 2026-10-30 created: box plugin
- 2026-10-30 set source user request 2026-10-30: grid foundation as "box" — expandable module interface, performance first, compact views on the grid and the full plugin in an inspector
- 2026-10-30 set depends 0024-box-module-library
- 2026-10-30 scope add plugins/box/ — ppc_box wrapper and deploy
- 2026-10-30 scope add justfile — ppc_box wrapper and deploy
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 set depends 0026-box-canvas-editor
