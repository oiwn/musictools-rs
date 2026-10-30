# MusicTools

Rust audio plugins by `oiwn`, built as a multi-plugin workspace with
[nice-plug](https://github.com/RustAudio/nice-plug) and egui editors on the
shared Dark Neon kit.

**Bogdan** is an experimental detail-preserving clipper. Its zero-latency v1
high-passes the signal removed by hard clipping and uses that detail to
modulate clipped plateaus inward while retaining the sample ceiling.

**Kirya** is a Dattorro plate reverb: the 1997 paper's topology with Plateau's
control set — size, freeze, four modulated tank allpasses, split input and
reverb damping — plus an impulse-response probe in the editor that renders the
current settings off-thread and draws their decay envelope over a
log-frequency spectrogram. Written clean-room from the paper; it is not a
Plateau clone and does not sound like one.

Each plugin has its own spec covering how it works, its parameters, and its
reference links: [`specs/bogdan.md`](specs/bogdan.md) and
[`specs/kirya.md`](specs/kirya.md). Workspace-wide architecture and house rules
live in [`specs/overview.md`](specs/overview.md); the active task is tracked in
[`specs/ctx.md`](specs/ctx.md).

## Workspace

- `crates/musictools-core` — suite-wide identity and reusable real-time-safe
  utilities.
- `crates/bogdan-dsp` — framework-independent clipper primitives and tests.
- `crates/kirya-dsp` — framework-independent plate reverb, offline
  impulse-response analysis, and tests.
- `crates/igorek-dsp` — a two-stage convolver engine, kept without a plugin
  wrapper (see [`specs/igorek.md`](specs/igorek.md)).
- `crates/neon-ui` — the Dark Neon egui widget kit every editor uses.
- `crates/audio-harness` — runs plugins over checked buffers in tests.
- `plugins/bogdan`, `plugins/kirya` — nice-plug wrappers and editors, shipped
  as `ppc_bogdan` and `ppc_kirya`.

Plugins build as CLAP. VST3 export exists behind each plugin's `vst3` cargo
feature and is off for now. Standalone applications, Audio Units, VST2, LV2,
AAX, and other formats are out of scope.

## Development

Install the bundler and run the normal Rust checks:

```sh
cargo install cargo-nice-plug
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Build a signed release bundle into `target/bundled/`:

```sh
cargo nice-plug bundle ppc_bogdan --release
cargo nice-plug bundle ppc_kirya --release   # add --features vst3 for VST3 too
```

On macOS, [`just`](https://github.com/casey/just) builds and installs in one
step by symlinking the bundle into `~/Library/Audio/Plug-Ins/CLAP`, so later
builds update the installed plugin in place:

```sh
just deploy ppc_bogdan
just deploy ppc_kirya
```

Restart the DAW or trigger its plugin rescan, then load **PPC Bogdan** or
**PPC Kirya** as an audio effect.

The DSP crates can be exercised without a host:

```sh
cargo run -p bogdan-dsp --example triangle_probe
cargo run -p kirya-dsp --release --example ir_probe
cargo run -p kirya-dsp --release --example tank_stability
```
