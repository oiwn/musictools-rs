# Bogdan — detail-preserving clipper

A clipper for drums and mix buses that keeps a hard outer ceiling while putting the detail a normal clipper would flatten back *inside* the plateau. Also carries an antialiased wavefolder as a second voice.

Crates: `crates/bogdan-dsp` (DSP, MIT) and `plugins/bogdan` (nice-plug wrapper, package `ppc_bogdan`, and nice-plug-egui editor). Formats: CLAP; VST3 exists behind the `vst3` cargo feature, off for now (user, 2026-10-30).

## How it works

The reference idea is not whole-signal foldback distortion. A normal clipper turns a loud transient into a flat plateau and throws away everything that was happening up there. Bogdan keeps that information and folds it back inward, so the waveform keeps a fixed outer ceiling with smaller detail moving inside the region that would otherwise be flat.

```
in -> x drive -> hard clip to ceiling -> clipped
                        |
                        +-> delta = driven - clipped
                                     |
                                     +-> one-pole high-pass (Detail knob)
                                                |
                                                +-> rectify, duck inward
```

Per sample the processor produces a `DetailFrame`: `driven`, `clipped`, `delta`, `filtered_delta` and the final `output`. The editor's oscilloscope draws `driven` against `output`.

### Modes

- **Clean** — plain hard clip. The clipping delta is discarded. The reference path.
- **Detail** — the original voice. The delta is high-passed, rectified, and subtracted inward from the clipped plateau: `magnitude = (|clipped| - amount * |filtered_delta|).max(0)`, sign restored from `clipped`. Suits drum-and-bass material.
- **Fold** — antialiased wavefolder. Excursions past the ceiling reflect back inside it. Drive sets fold density, Ceiling sets fold amplitude, Amount blends clip toward fold.

### The wavefolder

`Fold` is a memoryless nonlinearity made alias-suppressed with **first-order antiderivative antialiasing (ADAA)**:

```
y = (H(x) - H(x1)) / (x - x1)
```

where `H` is the antiderivative of the transfer function `h` and `x1` is the previous driven sample. Below `ADAA_EPSILON` the quotient is ill-conditioned, so it falls back to evaluating `h` at the midpoint.

Two shapes, both bounded to the ceiling `c` and both with closed-form antiderivatives:

- **Sine**: `c * sin(pi x / 2c)`. Smooth and musical.
- **Triangle**: `c * (2/pi) * asin(sin(pi x / 2c))`. Brighter zig-zag folds; its antiderivative is built by phase reduction over the period `4c`.

The blend is `(1 - amount) * clip(x) + amount * fold(x)`, and its antiderivative is the same blend of the two antiderivatives — so ADAA applies to the mix directly rather than to each branch.

## Parameters

Ids are a permanent contract: a host stores automation against them, so an id must never be reused or renumbered.

- **0 Drive** — `linear(0, 24)` dB, default `0`, step `0.01`, smoothing `exp(5)`.
- **1 Ceiling** — `linear(-24, 0)` dB, default `-0.1`, step `0.01`, smoothing `exp(5)`.
- **2 Mode** — enum Clean / Detail / Fold, default Detail, no smoothing.
- **3 Detail** — `log(20, 2000)` Hz, default `1000`, step `0.01`, smoothing `exp(20)`.
- **4 Amount** — `linear(0, 1)` %, default `1`, step `0.01`, smoothing `exp(5)`.
- **5 Shape** — enum Sine / Triangle, default Sine, no smoothing. Read only by the Fold voice; the editor grays it out in Clean/Detail.

The Detail cutoff is displayed through `musictools_core::format_hz`, shared with Kirya, so both plugins print frequencies the same way.

## Editor

Dark Neon, 540×420 (up from the legacy 500×376 to give the scope room), laid out like Kirya's, every block in an exact rect from one vertical budget (asserted in tests):

- **Header strip** — "PPC BOGDAN" tracked, "detail clipper · v…" in Muted, and a "?" icon button (shows "×" while open) that swaps the scope and controls for a scrolling help screen. The help copy is the `HELP` table in `editor.rs`; keep it in step with this spec.
- **Scope** in an inset well — PROCESSED in Cyan with glow over a dimmer DRIVEN in Orange, Red ceiling guides labelled with the Ceiling in dB, a centre line. Labels sit in gutters: legend chips above, levels on the left. Guides sit at ±75% of the plot's half-height (`CEILING_GUIDE_SPAN`), the waveform scaled to match; a silent tap idles as a flat line. Off-scale stretches of a trace are skipped (gaps), not flattened against the border, so DRIVEN never looks pre-clipped.
- **Control band** — MODE vertical tabs, the CLIPPER knobs (Drive, Ceiling, Detail, Amount), SHAPE vertical tabs, each captioned and divided. SHAPE is disabled unless Mode is Fold.

## Gotchas / knowledge

- **The output can never exceed the hard-clipping reference ceiling**, and unclipped samples always pass through unchanged. Both are asserted in tests across every mode and a range of ceilings.
- **The delta filter runs every sample**, including through unclipped gaps and across mode switches, so its memory stays warm and clipping resuming does not click.
- **`Detail` at Amount 0 is bit-identical to a plain hard clip**, which makes the knob a true depth control rather than a character change.
- The filter coefficient is recomputed only when the cutoff actually moves, so a static Detail knob costs no `exp()` per sample.
- Constant clipping settles back to the hard ceiling: the high-passed delta of a constant is zero, so a sustained plateau stops ducking. That is intended — the effect is about motion, not level.
- The ADAA quotient can nudge a hair past the ceiling numerically, so the result is clamped.

## Readings

- Au5 describes the reference method as high-passed foldback of the clipping delta, using ring-mod sidechain ducking to subtract detail from the clipped signal: <https://www.patreon.com/Au5Music/posts/detail-clipper-156412018>
- Kilohearts Compactor, the reference building block, because it can duck at sample rate: <https://kilohearts.com/products/compactor>
- Jatin Chowdhury on antialiased wavefolding: <https://ccrma.stanford.edu/~jatin/ComplexNonlinearities/Wavefolder.html>

## Status

The first usable implementation uses independent channels, a one-pole 1 kHz high-pass on the clipping delta, unity inward subtraction, zero latency, and no oversampling. These are conservative v1 defaults rather than a claim that the measured and listened-to comparison is complete.
