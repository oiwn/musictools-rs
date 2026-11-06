# box

box is the suite's modular grid, in the style of Bitwig's grid. Small modules sit on a cell grid and are patched together with wires. It runs as a standalone app first (`cargo run -p box-app --release`, eframe + cpal), which keeps it easy to debug. A plugin (`ppc_box`) comes later and hosts the same canvas.

## Crates

- `crates/box-core`: the module interface. It holds descriptors, param specs, the `Module` trait, the lock-free `ParamStore` and meters, the `Smoother`, and the cell-grid geometry. It doesn't depend on egui or nice-plug.
- `crates/box-dsp`: the patch model (serde), the graph compiler, and the realtime engine with its lock-free swap.
- `crates/box-modules`: one folder per module (descriptor, egui-free DSP, the single tile view), shared tile helpers (`tile.rs`) and readout formatters (`format.rs`). The `MODULES` registry lives here too. Modules are compiled in; there is no plugin SDK (user, 2026-11-01).
- `crates/box-app`: the standalone app (window, audio stream, canvas, save and load).
- `plugins/box` (`ppc_box`): the nice-plug wrapper (0025).

The dependencies run one way: core ← dsp, core ← modules, and app ← all three.

## The cell grid

- A cell is 96 px. Each module has a fixed size in cells, for example Sine 1x1 or Env 2x1, and that tile is its only UI. There is no second compact view and no inspector (user, 2026-10-31).
- The tile is its cells inset by 2 px. Box draws the frame: an 18 px header strip with a power switch and the label, plus a nub for each port on the tile edges. The module's view fills the body below the header.
- A port has a side (left, right, top or bottom) and a slot. The ports on one side spread evenly along that edge, across the body height for the left and right sides. Inputs usually sit on the left and outputs on the right.
- Tiles snap to cells and never overlap.

## Signals

- Every port carries one mono audio-rate `f32` stream. A gate is just a 0/1 signal on an ordinary port, so Gates patches straight into Env's gate input.
- An unconnected input reads silence, and the module can ask whether an input is connected. Several wires into one input are summed.
- `PortKind` is `#[non_exhaustive]`, so a modulation port kind can be added later without breaking modules.
- **Pitch** is an ordinary port measured in octaves: 0.0 is C4 (261.63 Hz) and +1.0 is one octave up, like V/oct. `box_core::pitch` converts between pitch, semitones, cents and Hz. Oscillators play `FREQ × 2^pitch`; sequencers output pitch and scalers quantize it. Clocks and gates stay 0/1.
- A **sink** module (Output) is a terminal node. Its two outputs, L and R, are the graph's output. They are summed across sinks and not drawn as nubs.

## Params

- Each param is a `ParamSpec`: a stable id, a range with a taper (linear, skew or log), a unit, an optional format and parse, and a smoothing time.
- Values live in a `ParamStore`, one `AtomicU32` (the f32 bits) per param per module instance. The UI writes it, and the engine snapshots it once per block into `ParamValues`. Modules glide between snapshots with `Smoother`, so there's no per-sample atomic.
- Changing a param value never rebuilds the graph. There's no host automation of module params in v1.

## Engine

- Graphs are acyclic. The compiler refuses a wire that would close a cycle.
- The compiler runs off the audio thread. It does a topological sort, culls nodes with no path to a sink, reuses buffers by liveness, and adds a sum step for fan-in. The result is a flat run order.
- The audio thread calls `process` once per module per block. It doesn't allocate, free or take a lock.
- A new compiled graph arrives over an `rtrb` queue and is installed at a block boundary. Modules that survive the change keep their state, matched by uid. The old graph goes back over a return queue and is dropped on the UI thread.
- Graphs report zero latency in v1.

## Persistence

A patch is JSON. Each node is `{uid, kind, cell, params by id, powered}`, and each cable is `{from: (uid, port), to: (uid, port)}`. On load:

- an unknown kind loads as a placeholder tile;
- an unknown param id is skipped;
- a missing param takes its default.

## Adding a module

1. Create a folder `crates/box-modules/src/<name>/`:
   - `mod.rs`: the stable `KIND`, param and port index constants, the `static` port and param specs, `pub static DESCRIPTOR` (with `create: dsp::create`), and `pub const ENTRY` (with `view::view` and `view::layout`).
   - `dsp.rs`: the `Module` impl and its tests. It never imports egui.
   - `view.rs`: `layout(body) -> Vec<Rect>` (the widget rects) and `view(ui, tile)`, built from the helpers in `tile.rs`.
2. Add `pub mod <name>;` and one line to `MODULES` in `crates/box-modules/src/lib.rs`.

The registry tests check that kinds are unique, that the ports sit on the tile and that the view fits.
