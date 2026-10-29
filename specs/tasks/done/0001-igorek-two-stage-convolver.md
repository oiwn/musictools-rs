---
id: 0001-igorek-two-stage-convolver
status: done
scope: [crates/igorek-dsp/, plugins/igorek/]
created: 2026-10-29
source: specs/ctx.md Igorek design task; landed in commit 0b73ddd
grade: huge
---
# Task: igorek two stage convolver

## Plan

- [x] `crates/igorek-dsp` skeleton: workspace member, `IgorekSettings`/frame structs, `safe_sample_rate`, identity default Color IR.
- [x] Partitioned overlap-save engine (P=128, N=256, block carry), tested against naive direct convolution.
- [x] Room generator ported from stoRIR-rs with stereo decorrelation; tests for RT60, ITDG, seed determinism, L/R independence.
- [x] Selection + 3-point envelope baking; RMS-normalized WAV load path (`hound`, linear resample, caps, non-finite rejection).
- [x] Processor assembly: stage mixes, Dry/Wet, install/swap protocol.
- [x] `plugins/igorek` wrapper: 22 params (ids per spec), `latency() = 128`, custom-state Color IR persistence, `rt-paranoid` test.
- [x] Background build task (debounced, `spawn_coalescing`) + editor: IR panes, envelope/selection dragging, `rfd` load button on a helper thread.
- [x] `truce.toml` entry (fourcc `Igrk`), CLAP + VST3 bundles built.
- [x] Specs, roadmap, and `ctx.md` updated.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Manual checks

- [x] Load Igorek in a host: pane drags, room regeneration, and the Load WAV dialog behave; listen to both stages. — waived by user: "Close both, drop from releases — we tried the convolver and it failed miserably; I use Bogdan and Kirya, moving on."

## Summary

Two-stage convolver: Color and Room stages with baked selection/envelope, a partitioned FFT engine, and a Truce wrapper with an egui editor.

## Context

- Design, signal chain, parameter contract, gotchas: [`igorek.md`](../igorek.md).
- Landed as one commit (`0b73ddd`); this file is the task record, converted from the `specs/ctx.md` design task on 2026-10-29.
- User decisions: Color IRs come from the user's own WAV files (no synthesized library), ~2.7 ms latency is acceptable, envelope/selection are host params, and stoRIR code may be ported directly.

## Findings

- **Open — the Load WAV dialog still fails in one host** even with `rfd::AsyncFileDialog`. Deferred until testing in Waveform; the suspects are recorded in `igorek.md` under Gotchas.
- Supply chain verified clean: `arrayref` pinned at 0.3.9, no compromised crates, `cargo audit` reports zero advisories.

## Review

- Acceptance re-run on rustc 1.99: `cargo fmt --all --check`, Clippy with `-D warnings`, 14 test suites, `typos`, and `gitleaks` all pass.
- The workspace was Clippy-red when verification started, from two newly promoted lints. Five sites inside this task's scope (`crates/igorek-dsp` tests and the `conv_probe` example) were fixed here; the other 18 went to task 0002 rather than widening this task.
- This round's code diff is `chunks_exact` → `as_chunks` in test and example code only. The DSP path is untouched.
- Still open: the Load WAV dialog in one host. The Manual check stays unticked, so `done` waits for the host test.

## Log

- 2026-10-29 created: igorek two stage convolver
- 2026-10-29 set source specs/ctx.md Igorek design task; landed in commit 0b73ddd
- 2026-10-29 set grade huge
- 2026-10-29 scope add crates/igorek-dsp/ — DSP crate and Truce wrapper landed in 0b73ddd
- 2026-10-29 scope add plugins/igorek/ — DSP crate and Truce wrapper landed in 0b73ddd
- 2026-10-29 draft approved: Convert to a task file 0001-igorek-two-stage-convolver
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: crates/igorek-dsp/, plugins/igorek/
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 baseline: AGENTS.md@84894d0ea22c, specdev.toml@7c63880b5cec
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 blocked: acceptance gate depends on task 0002: workspace clippy is red from two newly promoted lints on rustc 1.99
- 2026-10-29 advance blocked → in-progress/verify
- 2026-10-29 baseline: AGENTS.md@84894d0ea22c, plugins/bogdan/src/editor.rs@6c8d69483497, plugins/bogdan/src/lib.rs@5a963e4b0f46, plugins/kirya/src/editor.rs@6bda23791c42, specdev.toml@7c63880b5cec
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
- 2026-10-29 approved: Close both, drop from releases — convolver failed miserably; Bogdan and Kirya are the plugins I use.
- 2026-10-29 advance approval → done
