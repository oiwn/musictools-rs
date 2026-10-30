# nice-plug Reference

Project-agnostic notes on building plugins with **nice-plug 0.4** (RustAudio's nih-plug continuation), **nice-plug-egui 0.6**, and **egui 0.37**. They are distilled from the Bogdan and Kirya ports (tasks 0016–0017), so we stop re-deriving the API. Re-verify after a version bump: `cargo agents crate-info nice-plug` gets the source. UI specifics (repaints, fonts, layout, offscreen renders) live in the neon-ui skill, not here.

## Crates and features

- The workspace pins `nice-plug`, `nice-plug-egui` and `egui` (with `default_fonts`, which the plugin path needs) once, in the root `Cargo.toml`.
- A plugin crate is `crate-type = ["cdylib", "staticlib", "rlib"]` with package name `ppc_<name>`.
- Each plugin has two features:
  - `rt-audit = ["musictools-core/alloc-audit"]`, enabled by `--all-features` test runs;
  - `vst3 = []`, which gates the `Vst3Plugin` impl and `nice_export_vst3!`. The bundler only emits a format the library exports, so bundles are CLAP-only by default.
- Dev-dependency `audio-harness` for tests.
- Plugin DSP lives in a framework-free `crates/<name>-dsp`; the wrapper stays thin.

## Params

- Write `#[derive(Params)]` on a struct and give each field `#[id = "snake_name"]`.
  - String ids are the permanent automation contract, so never rename or reuse one.
  - Short ids only matter if two ids' hashes collide.
- Non-param state (`Arc<AtomicU64>`, `Arc<Mutex<…>>`) can sit on the same struct as plain fields.
- Param types: `FloatParam`, `IntParam`, `BoolParam`, `EnumParam<E>` with `#[derive(Enum)]` (variant names show in hosts).
- `FloatRange::Linear`, `Skewed { min, max, factor }` and `SymmetricalSkewed`.
  - `Skewed` normalizes as `t^factor`, the same curve as truce's `skewed`.
  - There is **no log range**. For a log knob, store the position `0..1` and map it through `musictools_core::LogTaper` in `with_value_to_string` / `with_string_to_value` and in the DSP read (see `plugins/kirya/src/lib.rs` `log_hz`).
- `.with_smoother(SmoothingStyle::Exponential(ms) | Linear(ms) | Logarithmic(ms) | None)`.
  - `param.value()` is the target value; `param.smoothed.next()` advances one sample.
  - Read each smoother once per frame, not once per channel.
- Units are printed once: either `.with_unit(" ms")` or a formatter that includes the unit, never both. Shared formatters are in `musictools_core`: `format_hz`, `parse_hz`, `format_percent`, `parse_percent`.
- `.with_step_size(0.01)` gives snapping. Editors push plain values via `ParamSetter`; see the neon-ui bindings.
- nice-plug has no per-param "inactive" flag, so greying out a control is editor-side (`ui.disable()`).

## The plugin impls

- `impl Plugin` provides:
  - `NAME`, `VENDOR`, `URL`, `EMAIL`, `VERSION`;
  - `AUDIO_IO_LAYOUTS` (stereo plus mono);
  - `type Editor`, `type SysExMessage = ()`, `type BackgroundTask`;
  - `params()`, `activate()` (allocate and size for the sample rate here), `reset()`, `process()`, `editor(&mut self, AsyncExecutor<Self>)`.
- **`reset()` can run on the audio thread.** Clear state there, never reallocate (Kirya's `reverb.clear()` vs `reset`).
- `process` returns a `ProcessStatus`:
  - `Normal`;
  - `Tail(samples)` for a decaying effect;
  - `KeepAlive` when output never ends, such as Kirya's Freeze.
- Iterate the audio with `buffer.iter_samples()` and `samples.get_mut(ch)`. Handle mono by falling back to channel 0.
- `impl ClapPlugin` provides `CLAP_ID = "com.oiwn.ppc_<name>"`, a description, and `CLAP_FEATURES` (`AudioEffect`, `Stereo`, plus `Reverb`, `Distortion`, `Filter` and so on).
- The feature-gated `impl Vst3Plugin` provides a fresh 16-byte `VST3_CLASS_ID` and `VST3_SUBCATEGORIES`.
- Export with `nice_export_clap!(P);` and, behind `#[cfg(feature = "vst3")]`, `nice_export_vst3!(P);`.

## Background work

- `type BackgroundTask = T` plus `fn task_executor(&mut self) -> TaskExecutor<Self>` returning `Box::new(move |task| …)`.
  - Capture an `Arc` of the params, not `self`.
  - The task type should carry no heap data.
- The editor gets an `AsyncExecutor<Self>` (Clone) and calls `execute_background(task)`. The audio thread can do the same through `context.execute_background`.
- **No coalescing.** Every posted task runs. Guard with a request id:
  1. bump an `AtomicU64` when posting;
  2. the task skips if it is stale before working;
  3. it checks again under the result slot's lock before publishing. `plugins/kirya/src/task.rs` shows the pattern, and `a_superseded_request_renders_nothing` tests it.
- Results are handed back as `Arc<Mutex<Option<…>>>` plus a generation counter. The editor or audio thread only uses `try_lock`.

## State

- Host params persist automatically.
- Extra state is a `#[persist = "key"]` field holding a serde type behind `Arc<Mutex<…>>` / `Arc<RwLock<…>>` on the params struct. It is not used in this repo yet, so check the API on first use (Igorek's IR and the grid graph will need it).
- There is no "state loaded" callback. Detect recall by polling a generation or comparing values in `process` or the editor.

## Editors (nice-plug-egui)

- Build the window with `EguiEditorState::from_size(WindowSize::from_logical(LogicalSize::new(w, h), 1.0), 1.0)`, then `create_egui_editor(state, repaint.clone(), EguiNiceSettings::new().with_tile("ppc_<name>"), app)`.
- The app implements `NiceEguiApp`:
  - `build` sets the visuals and fonts and stores the `GuiContext`;
  - `ui` gets the root `Ui`, the whole window with no panel;
  - `editor_closed` drops the context.
- Split the work: `ui()` does host plumbing (repaint, `gui.param_setter()`) and calls a host-free `draw(ui, setter)`, which can then be rendered offscreen.
- Repaint pacing, fonts, layout budgets and screenshots: see `.claude/skills/neon-ui/SKILL.md`.

## Tests

- `audio_harness::Harness::new(channels, block)` gives a reused, checked `Buffer`:
  - `set_input`, then `process(&mut plugin)`, then `output(ch)`;
  - it allocates nothing in steady state.
- `audio_harness::automate(&param, plain, rate)` moves a param the way host automation does (value plus smoother retarget). nice-plug keeps those setters `unsafe`, and `audio-harness` is the only crate allowed to call them.
- Zero-allocation audit: `#[cfg(feature = "rt-audit")]` tests wrap the process loop in `musictools_core::rt_audit::audit(|| …)` and assert the count is `0`.
- Sweep the params that resize or swap things inside the audited loop.

## Bundling and install

- `cargo install cargo-nice-plug`, then `cargo nice-plug bundle ppc_<name> --release`. Add `--features vst3` for VST3.
  - Bundles land in `target/bundled/` and are signed for local use on macOS.
  - `cargo xtask` is an alias.
- `just deploy ppc_<name>` bundles and symlinks the `.clap` into `~/Library/Audio/Plug-Ins/CLAP`, so rebuilds update the installed plugin in place. Rescan or restart the host.
- The `ppc_*` plugins coexist with the legacy truce builds, which keep their own names and ids.
