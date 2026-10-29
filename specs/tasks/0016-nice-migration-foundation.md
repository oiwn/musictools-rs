---
id: 0016-nice-migration-foundation
status: approval
scope: [Cargo.toml, Cargo.lock, .cargo/config.toml, crates/musictools-core/, crates/neon-ui/, plugins/bogdan/, crates/audio-harness/]
created: 2026-10-29
source: user request 2026-10-29: migrate to nice-plug (egui 0.37) per roadmap decision; three-task plan approved in session
---
# Task: nice migration foundation

## Plan

- [x] Workspace swap: root Cargo.toml drops the truce family and clap-sys, adds `nice-plug` 0.4 + `nice-plug-egui` 0.6, bumps `egui` to 0.37; neon-ui eframe dev-dep to 0.37; delete `truce.toml`; `.cargo/config.toml` alias `xtask` → `nice-plug`.
- [x] neon-ui on egui 0.37: kit source is verified API-compatible; showcase `CentralPanel::show_inside` → `.show`.
- [x] `musictools-core::rt_audit` behind feature `alloc-audit`: the `assert_no_alloc` counting allocator installed with the feature, `audit(closure) -> (R, u64)` over its scope — replaces `truce::rt::audit` / `enable_rt_paranoid!`; unit tests.
- [x] `neon-ui::bindings` behind feature `nice-plug`: `param_knob`, `param_select`, `param_toggle` over nice-plug's `ParamSetter`, replacing the truce_egui widgets.
- [x] Bogdan port: `Plugin` struct (Arc params + DSP state), params to `#[id]` + constructors (smoothers, units, `with_value_to_string(format_hz)`), `EnumParam` for Mode/Shape, `smoothed.next()` reads, `ClapPlugin` + `Vst3Plugin` impls and export macros.
- [x] Bogdan editor to `NiceEguiApp`: `build()` applies visuals (truce theme colors inlined as local consts — the neon rebuild is 0006), bindings widgets, request_repaint_after.
- [x] Checked-buffer harness ported to nice-plug's Buffer as `crates/audio-harness` (the one `set_slices` unsafe, contained); zero-alloc test runs on musictools-core's audit.
- [x] Install cargo-nice-plug; `cargo nice-plug bundle ppc_bogdan --release` produces `.clap` + `.vst3` in target/bundled (both signed).

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `cargo nice-plug bundle ppc_bogdan --release`

## Manual checks

- [ ] Bogdan loads in a host (DAW or pluginval/clap-validator): params automate, audio processes, editor opens and works.

## Context

- Naming convention 2026-10-29 (user): the nice-plug generation ships as separate coexisting plugins — package, display name, bundle file, and ids all `ppc_bogdan` / `com.oiwn.ppc_bogdan` (fresh VST3 class id), so the legacy truce Bogdan stays installed and existing projects are untouched. Kirya/Igorek follow as `ppc_kirya`/`ppc_igorek` in their ports.
- Framework decision 2026-10-29 in [`roadmap.md`](../roadmap.md); egui 0.37 API impact was verified item-by-item during planning — only the showcase rename breaks.
- Kirya (0017) and Igorek (0018) still build on truce until their ports; the workspace cannot hold both frameworks for them, so this task swaps the workspace and ports bogdan in the same commit — kirya/igorek crates are temporarily excluded from workspace members until 0017/0018 port them.
- Risks recorded in the approved plan: egui 0.37 text-tessellation regression (fallback nice-plug-egui 0.5.x + egui 0.36.2), bundler-hardcoded `com.nice-plug.*` bundle IDs, f32-only params.

## Review

- Scope held: workspace manifests, musictools-core (two new modules), neon-ui (bindings + feature), bogdan (full port), audio-harness (new, scope-added before touching). Kirya/igorek are out of workspace members until 0017/0018, per the approved plan.
- The planning research was directionally right but the port found real differences: `activate()` takes single refs (not slices), `DetailSettings` is f32-based (the planned f32→f64 boundary conversion was unnecessary), params ship `Smoother::none()` and need an explicit `reset(value)` snap at activation (now in `configure()`), `WindowSize::from_logical` wants a `LogicalSize<f64>` plus scale, and `Frame` in `NiceEguiApp` is `egui_baseview`'s re-export.
- rt-paranoid replacement is `assert_no_alloc` (warn_debug mode: counts, never aborts) with `#[global_allocator]` in `musictools-core::rt_audit` applied whenever the feature is on; `audit()` enters the allocator's scope so counts are real. Bogdan's zero-alloc test passes on it.
- `crates/audio-harness` contains the workspace's only unsafe: one `Buffer::set_slices` over leaked, never-reallocated storage, reused across process calls so steady-state is allocation-free; `StubContext` implements the eight required `ProcessContext` methods.
- Kit widgets now carry proper `mark_changed()` contracts (knob/slider/range/select/toggle), so the bindings' `response.changed()` push path works — found while writing the bindings, fixed in the kit.
- The egui 0.37 bump needed exactly the one predicted rename in the showcase; the kit compiled untouched. The showcase smoke-runs.
- Bundles: `cargo nice-plug bundle ppc_bogdan --release` produced and ad-hoc-signed both bundles in target/bundled.
- Post-review fixes from the first in-host screenshot (Serum 2), both source-verified: (1) all text rendered as tofu boxes — egui 0.37 ships `default_fonts` as a non-default feature and the plugin path (nice-plug-egui/egui-baseview) enables it only in egui-baseview's dev-dependencies; fixed by enabling `default_fonts` on the workspace `egui` dep. (2) the Detail knob sat at ~8% travel — `FloatRange::Skewed` normalizes as `((v-min)/(max-min)).powf(factor)`, so the ported factor 3.46 was inverted; corrected to 0.232, which restores truce's `log(20, 2000)` feel (1000 Hz default ≈ 85% travel). Empty scope in that shot was no-audio-flow, not a defect; re-check with playback.
- Round 2 from in-host feedback: the Mode/Shape dropdowns opened misaligned horizontally (egui popup `BOTTOM_START` + edge auto-flip) and only appeared after a mouse move — replaced with popup-free segmented controls (`neon-ui::bindings::param_tabs` over the kit's tabs, which gained `tabs_width` + `mark_changed`), editor widened 560→600, and the editor now pings a cloned `RepaintNotifier` every frame (polled at 1/64 s) so repaints never wait on OS events. The select popup alignment is recorded in `specs/cleanup.md` for 0006/0007.
- Round 3 from in-host feedback ("buttons are better but we need to use vertical space and align them vertically"): the horizontal tab strips were 22 px tall floating beside 86 px knob columns. Added `tabs::tabs_vertical` to the kit (segments stacked, accent bar on the leading edge, `tabs_vertical_width` sizing from the longest label) plus `bindings::param_tabs_vertical(ui, setter, param, height, salt)`; `KnobSize::footprint()` now exposes the knob's 56×86 allocation so editors size the stacks to the knob column exactly. Bogdan's Mode/Shape stacks now fill the knob height and align top/bottom; the scope grew 184→192 to consume the window's vertical slack. Showcase demos the stacked variant next to the horizontal one.
- Round 4 from in-host feedback: knobs now step in 0.01 via `with_step_size(0.01)` on all four float params plus a fixed push path in `bindings::param_knob_with` (plain value through `set_parameter`; `set_parameter_normalized` skips snapping per nice-plug docs). Reset-to-default existed on double-click but was unreliable in-host, so ctrl/⌘-click was added in the kit knob. Shape is read only by the Fold voice (verified in bogdan-dsp), so the editor grays the Shape stack via `add_enabled_ui(mode == Fold)` and the kit's tabs paint a ghost state when `!ui.is_enabled()`. Window narrowed 600→500 with `tabs_vertical_width` padding 24→16 / floor 72→64. The 24 px title block became a compact header — "BOGDAN v0.3.0" left, "?" `icon_button` right — toggling to a scrolling help screen (mode/control copy from specs/bogdan.md) in place of the interface; the freed header height went to the scope (192→226). Workspace version bumped 0.2.0→0.3.0 (same mechanism as the v0.2.0 entry; `Plugin::VERSION` = `env!("CARGO_PKG_VERSION")` feeds the CLAP/VST3 descriptors). specs/bogdan.md parameter lines now record the step.
- Round 5 (header polish): the `ui.horizontal` + nested `right_to_left` header let layout padding sit above the row and drift the "?" off the text line. The header is now a deterministic 26px strip allocated flush at the top — text painted left-center, "?" `icon_button` placed via `new_child(UiBuilder)` at an exact rect top-right — and reads "PPC BOGDAN v0.3.0" (help-screen heading matched). Renamed-deprecation notes: egui 0.37 wants `Ui::scope_id`/`UiBuilder::scope_id`, `Id::new` is deprecated.

## Summary

Workspace on nice-plug 0.4 / egui 0.37 with the alloc-audit module, neon-ui param bindings, and Bogdan ported end-to-end including bundles.

## Log

- 2026-10-29 created: nice migration foundation
- 2026-10-29 set source user request 2026-10-29: migrate to nice-plug (egui 0.37) per roadmap decision; three-task plan approved in session
- 2026-10-29 scope add Cargo.toml — workspace dependency swap, alloc-audit module, neon-ui bindings, bogdan port
- 2026-10-29 scope add Cargo.lock — workspace dependency swap, alloc-audit module, neon-ui bindings, bogdan port
- 2026-10-29 scope add .cargo/config.toml — workspace dependency swap, alloc-audit module, neon-ui bindings, bogdan port
- 2026-10-29 scope add crates/musictools-core/ — workspace dependency swap, alloc-audit module, neon-ui bindings, bogdan port
- 2026-10-29 scope add crates/neon-ui/ — workspace dependency swap, alloc-audit module, neon-ui bindings, bogdan port
- 2026-10-29 scope add plugins/bogdan/ — workspace dependency swap, alloc-audit module, neon-ui bindings, bogdan port
- 2026-10-29 draft approved: Approved with the three-task migration plan (ExitPlanMode approval, 2026-10-29)
- 2026-10-29 advance draft → ready
- 2026-10-29 scope approved: Cargo.toml, Cargo.lock, .cargo/config.toml, crates/musictools-core/, crates/neon-ui/, plugins/bogdan/
- 2026-10-29 advance ready → in-progress/implement
- 2026-10-29 scope add crates/audio-harness/ — checked-buffer test harness: the one unsafe Buffer::set_slices call, encapsulated and test-only
- 2026-10-29 advance in-progress/implement → in-progress/verify
- 2026-10-29 advance in-progress/verify → in-progress/review
- 2026-10-29 advance in-progress/review → approval
