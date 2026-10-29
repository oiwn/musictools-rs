---
id: 0017-kirya-nice-port
status: draft
scope: [plugins/kirya/]
created: 2026-10-29
source: user request 2026-10-29: kirya port to nice-plug after migration foundation
depends: [0016-nice-migration-foundation]
---
# Task: kirya nice port

## Plan

- [ ] Params port: skewed ranges → `FloatRange::Skewed` (factor converted), `freeze` BoolParam, Hz formatters via `with_value_to_string`, f32→f64 at the settings boundary.
- [ ] Process port: settings from `smoothed.next()`, tail → `ProcessStatus::Tail` / `KeepAlive` when frozen.
- [ ] RenderIr → `BackgroundTask` (no heap data), rendered in `Plugin::task_executor`, spawned from the editor via `async_executor.execute_background` behind the existing debounce + generation guard (nice-plug has no coalescing), results still through `ir_slot`/`ir_generation`, editor woken via `RepaintNotifier` stored on the plugin.
- [ ] Editor port to `NiceEguiApp` (spectrogram texture code is 0.37-stable).
- [ ] Tests ported, including the zero-alloc sweep on musictools-core's audit.
- [ ] Re-add `plugins/kirya` to workspace members; `cargo nice-plug bundle kirya --release`.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`
- [ ] `cargo nice-plug bundle kirya --release`

## Manual checks

- [ ] Kirya loads in a host: reverb tail correct, IR re-renders on knob moves (spectrogram updates), freeze works.

## Context

- Follows 0016's foundation; the port patterns (params, editor shell, test harness) come from the Bogdan port.
- The `sample_rate_bits` workaround stays: nice-plug's editor context also does not carry the sample rate.

## Summary

Kirya ported to nice-plug with the background IR render on the task executor and the editor on NiceEguiApp.

## Log

- 2026-10-29 created: kirya nice port
- 2026-10-29 set source user request 2026-10-29: kirya port to nice-plug after migration foundation
- 2026-10-29 scope add plugins/kirya/ — kirya wrapper and editor port
- 2026-10-29 set depends 0016-nice-migration-foundation
