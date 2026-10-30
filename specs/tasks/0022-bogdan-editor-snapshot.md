---
id: 0022-bogdan-editor-snapshot
status: draft
scope: [plugins/bogdan/src/editor.rs, plugins/bogdan/examples/snapshot.rs, plugins/bogdan/Cargo.toml, crates/neon-ui/examples/snapshot.rs, justfile, AGENTS.md]
created: 2026-10-30
source: user request 2026-10-30: render plugin screenshots programmatically for feedback
---
# Task: bogdan editor snapshot

## Context

The 0019 snapshot CLI renders neon-ui sections only; the Bogdan window itself can be seen only in a host. `BogdanEditor::ui` needs a host `GuiContext` (for `param_setter`) and a baseview `Frame`, and a stand-in `GuiContext` means implementing nice-plug's `unsafe` `GuiContextInner` trait, which the workspace's unsafe ban rules out. So the drawing is split from the host plumbing instead.

Finding from 0007 (2026-10-30): the stand-in host can live in `audio-harness`, the one crate allowed `unsafe` — a no-op `GuiContextInner` (eight methods, `PluginState` built from empty maps) plus `GuiContext::new(Arc::new(..))` was enough to render the whole Kirya editor through `egui_kittest` (`build_ui_state`, ppp 2). Kirya already splits `draw(ui, setter)` from `ui()`; Bogdan needs the same split. That route is much smaller than an edits-callback refactor.

Goal: one command renders the real editor to PNG for feedback, the same way kit scenes are reviewed.

## Plan

- [ ] Split `editor.rs`: a host-free `draw(ui, &BogdanParams, &mut ViewState, edits: &mut dyn FnMut(Edit))` paints the whole window (header, help, scope, controls) from param values; `BogdanEditor::ui` wraps it and applies edits through `ParamSetter`. The in-host behaviour is unchanged.
- [ ] `plugins/bogdan/examples/snapshot.rs` (egui_kittest + wgpu, as in neon-ui): default params at the editor's window size, a synthetic clipped sine fed to the scope history, fonts and visuals as in `build`. Scenes: `default`, `fold` (Shape enabled), `help`, plus a hover/drag pass over one knob. Renders go to `target/snapshots/bogdan-*.png`, with `--out`, `--scale`, and `--list` matching the kit CLI.
- [ ] Share the flatten/save/arg helpers with the neon-ui snapshot instead of copying them, if that stays small (an `examples/common` module or a dev-only feature); otherwise duplicate the ~40 lines and note it in `specs/cleanup.md`.
- [ ] A test asserting `draw` lays out within the window size (no control clipped at the bottom edge).
- [ ] Document the command in `AGENTS.md` and `specs/bogdan.md`; add `just snapshot [plugin]`.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`
- [ ] `cargo run -p ppc_bogdan --example snapshot -- all`

## Manual checks

- [ ] The rendered `bogdan-default.png` matches the in-host window (user compares against a host screenshot).

## Log

- 2026-10-30 created: bogdan editor snapshot
- 2026-10-30 set source user request 2026-10-30: render plugin screenshots programmatically for feedback
- 2026-10-30 scope add plugins/bogdan/src/editor.rs — split editor draw from host, add headless editor snapshot example and docs
- 2026-10-30 scope add plugins/bogdan/examples/snapshot.rs — split editor draw from host, add headless editor snapshot example and docs
- 2026-10-30 scope add plugins/bogdan/Cargo.toml — split editor draw from host, add headless editor snapshot example and docs
- 2026-10-30 scope add crates/neon-ui/examples/snapshot.rs — split editor draw from host, add headless editor snapshot example and docs
- 2026-10-30 scope add justfile — split editor draw from host, add headless editor snapshot example and docs
- 2026-10-30 scope add AGENTS.md — split editor draw from host, add headless editor snapshot example and docs
