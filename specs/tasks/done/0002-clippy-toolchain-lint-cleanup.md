---
id: 0002-clippy-toolchain-lint-cleanup
status: done
scope: [plugins/bogdan/src/editor.rs, plugins/bogdan/src/lib.rs, plugins/kirya/src/editor.rs, plugins/igorek/src/editor.rs]
created: 2026-10-29
source: clippy 0.1.99 failures on rustc 1.99 blocking task 0001's acceptance gate
---
# Task: clippy toolchain lint cleanup

## Plan

- [x] `plugins/bogdan/src/editor.rs`: suffix the seven float literals with `_f32` and switch the `chunks_exact` call to `as_chunks`.
- [x] `plugins/bogdan/src/lib.rs`: switch the `chunks_exact` call to `as_chunks`.
- [x] `plugins/kirya/src/editor.rs`: suffix the five float literals with `_f32`.
- [x] `plugins/igorek/src/editor.rs`: suffix the four float literals with `_f32`.
- [x] Confirm the workspace passes every acceptance command.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`

## Context

- rustc 1.99 / clippy 0.1.99 promote two lints the code predates: `float-literal-f32-fallback` (rust#154024) and `chunks_exact_to_as_chunks`. The repo runs `-D warnings`, so both are hard errors.
- 16 float-fallback sites across the three editor files; 2 `chunks_exact` sites in the bogdan plugin. The five igorek-dsp sites were already fixed inside task 0001's scope.
- No behavior change: an explicit `_f32` suffix picks the type the compiler already inferred, and `as_chunks` keeps the same full-chunk iteration as `chunks_exact`.

## Summary

Modernized 18 sites for the rustc 1.99 Clippy lints (`_f32` literals and `as_chunks`), restoring a clean `-D warnings` build.

## Review

- 18 mechanical sites: 16 `_f32` suffixes and 2 `as_chunks` swaps. No logic changed; the same values and the same full-chunk iteration.
- The workspace is Clippy-clean on rustc 1.99 again. rustfmt reflowed one `Shape::line` call that grew past the line width.
- Nothing is `allow`ed or suppressed: the next toolchain bump may promote another lint, which is expected to be handled the same way.

## Log

- 2026-10-29 created: clippy toolchain lint cleanup
- 2026-10-29 set source clippy 0.1.99 failures on rustc 1.99 blocking task 0001's acceptance gate
- 2026-10-29 scope add plugins/bogdan/src/editor.rs — the 18 remaining lint sites live in these files
- 2026-10-29 scope add plugins/bogdan/src/lib.rs — the 18 remaining lint sites live in these files
- 2026-10-29 scope add plugins/kirya/src/editor.rs — the 18 remaining lint sites live in these files
- 2026-10-29 scope add plugins/igorek/src/editor.rs — the 18 remaining lint sites live in these files
- 2026-10-29 draft approved: New task 0002 for the toolchain lint cleanup
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: plugins/bogdan/src/editor.rs, plugins/bogdan/src/lib.rs, plugins/kirya/src/editor.rs, plugins/igorek/src/editor.rs
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 baseline: AGENTS.md@84894d0ea22c, crates/igorek-dsp/examples/conv_probe.rs@b2177f1a4e90, crates/igorek-dsp/src/engine.rs@a04133e6e99d, specdev.toml@7c63880b5cec
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
- 2026-10-29 approved: Close both, drop from releases — convolver failed miserably; Bogdan and Kirya are the plugins I use.
- 2026-10-29 advance approval → done
