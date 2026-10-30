# Repository Guidelines

## Project Structure & Module Organization

This Rust 2024 Cargo workspace separates reusable code from plugin wrappers:

- `crates/musictools-core/` contains framework-independent, suite-wide utilities.
- `crates/bogdan-dsp/` contains Bogdan's DSP, unit tests, and offline examples.
- `crates/kirya-dsp/` contains Kirya's plate reverb, its offline impulse-response
  analysis (feature `analysis`, on by default), unit tests, and examples.
- `crates/neon-ui/` is the Dark Neon egui widget kit shared by every editor;
  `crates/audio-harness/` runs plugins over checked buffers in tests.
- `plugins/<name>/` contain the nice-plug wrappers and egui editors
  (nice-plug-egui), packaged as `ppc_<name>`: Bogdan and Kirya today.
- `crates/igorek-dsp/` keeps Igorek's convolver engine; its plugin wrapper
  was removed with truce (2026-10-30) and a revival writes a fresh one.
- `specs/` records architecture, per-plugin specs, active work, roadmap items,
  and ideas. `specs/overview.md` holds workspace-wide architecture and house
  rules; `specs/bogdan.md` and `specs/kirya.md` own each plugin's description,
  signal chain, parameter reference, gotchas, and reference links. A new plugin
  gets its own `specs/<name>.md`.
- Plugin metadata (names, ids, CLAP/VST3 info) lives in each plugin's
  `ClapPlugin`/`Vst3Plugin` impls; generated bundles land in
  `target/bundled/` and must not be committed.

Keep plugin-specific signal processing in its DSP crate. Move code into
`musictools-core` only when multiple plugins can use it.

## Build, Test, and Development Commands

- `cargo build --workspace` compiles every workspace member.
- `cargo test --workspace --all-features` runs the complete unit-test suite.
- `cargo fmt --all --check` verifies rustfmt formatting.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` treats
  all lint warnings as failures.
- `cargo run -p bogdan-dsp --example triangle_probe` runs a DSP probe locally.
- `cargo run -p kirya-dsp --release --example ir_probe` prints Kirya's impulse
  response, its reverb time, and an ASCII spectrogram.
- `cargo run -p kirya-dsp --release --example tank_stability` sweeps the reverb
  tank's parameter corners at four sample rates and fails on any runaway.
  Both Kirya examples want `--release`; they render tens of seconds of audio.
- `cargo run -p neon-ui --example showcase` opens the neon-ui living style
  guide; `cargo run -p neon-ui --example snapshot -- [scene...]` renders the
  same sections headless to `target/snapshots/*.png` (`--list` names the
  scenes, `--scale N` sets pixel density) for review against
  `specs/design-sketch.png`.
- `cargo nice-plug bundle ppc_bogdan --release` (or `ppc_kirya`) creates a
  signed CLAP bundle in `target/bundled/`. CLAP only for now: add `--features vst3` for a VST3
  bundle too. Needs the `cargo-nice-plug` bundler installed
  (`cargo xtask` is an alias for it in `.cargo/config.toml`).
- `just deploy [plugin]` (default `ppc_bogdan`) runs `cargo nice-plug bundle
  <plugin> --release` and symlinks `target/bundled/<plugin>.clap` into
  `~/Library/Audio/Plug-Ins/CLAP`; later bundles update the installed plugin
  in place. `just link` / `just unlink` manage only the symlink.

The hooks in `prek.toml` also run formatting, Clippy, tests, typo checks, and
secret scanning.

## Coding Style & Naming Conventions

Use standard rustfmt output (four-space indentation). Follow Rust conventions:
`snake_case` for modules, functions, and tests; `CamelCase` for types; and
`SCREAMING_SNAKE_CASE` for constants. Document public APIs and keep audio-thread
code allocation-free and robust against non-finite host input. Unsafe Rust is
forbidden workspace-wide.

Agents edit files with the harness's file tools (Read/Edit/Write), not Python,
sed, or heredoc scripts. Edits stay exact and reviewable that way, and
"file changed on disk" surprises go away.

## Testing Guidelines

Place focused `#[cfg(test)]` modules beside the code they exercise. Name tests
after observable behavior, such as `non_finite_values_use_fallback`. DSP changes
should cover ceiling bounds, finite output, channel independence, and state
reset behavior where relevant. Run the full test and Clippy commands before
opening a PR; there is no numeric coverage threshold.

## Commit & Pull Request Guidelines

Existing history uses short, informal subjects such as `initial design` and
`first version`. For new work, prefer concise imperative subjects, optionally
scoped, for example `bogdan-dsp: clamp fold output`. PRs should explain the
audible or architectural effect, list verification commands, and link relevant
issues or specs. Include screenshots for editor changes and listening or host
validation notes for audio behavior changes.

<!-- BEGIN specdev -->
## specdev

This project uses **specdev** (specification-driven development). Load the
specdev skill when starting a session, continuing from specs, or picking up a
task. Always read `specs/overview.md` and `specs/ctx.md` before coding, plus the
`specs/<plugin>.md` for whichever plugin you are touching. Task files live in
`specs/tasks/` and their state moves only through the `specdev` CLI; the task
contract, acceptance defaults, and size limits are in `specdev.toml`.
<!-- END specdev -->
