# Design System — Dark Neon

Source of truth for every editor in the suite. Transcribed from the user's style-guide sketch ([design-sketch.png](design-sketch.png)); implemented as the shared `crates/neon-ui` kit on egui 0.34 + truce-egui 6.3.0. The sketch is canonical where a rendered editor and this file disagree.

## Palette

Backgrounds, in depth order (window down to wells):

- **BG** `#0E141B` — window background, darkest layer.
- **Panel** `#1A222C` — panels and cards holding sections.
- **Surface** `#232C36` — controls: buttons, closed selects, slider tracks.
- **Raised** `#2D3945` — hover fills, icon buttons, knob bodies.
- **Inset** `#121921` — graph wells and value readouts sunk into a panel.

Accents, used sparingly; one accent leads per plugin:

- **Cyan** `#39D9FF` — primary: active states, meter fills, focus rings.
- **Blue** `#3A6BFF` — secondary: alternate curves and labels.
- **Orange** `#FF9800` — dry/mix-side marks, secondary readouts, warnings.
- **Red** `#F34D57` — clip, ceiling, destructive.
- **Yellow** `#E6C35C` — highlight ticks, snap indicators.

Text:

- **Primary** `#F6EDF6`, **Secondary** `#9AA5B1`, **Muted** `#63707E`, **Ghost** `#404A55` (grids, disabled).

Strokes `#26313D`/`#313E4B` for hairline panel and grid lines. Corner radius 2–3 px. Glow is layered strokes/fills of an accent at falling alpha — no blur passes.

## Typography

- Labels: 10–11 px, ALL CAPS, wide tracking, Secondary/Muted.
- Value readouts: monospace, Primary on Inset, editable on click.
- Headings: bold 14–16 px Primary; sub-labels 10 px Muted.

## Widgets

Every widget is fully custom-painted; stock egui widget visuals never show:

- **NeonKnob** — sizes S/M/L. Radial body (Raised→Panel gradient), arc indicator swept from start to value in the plugin's accent, value readout below or centered, ALL-CAPS label. Tick ring marks snap points and lights Yellow while snapping. Interaction: vertical drag, Shift = fine, double-click = default.
- **NeonSlider** — inset track, accent fill up to the handle, raised handle; double-thumb range variant.
- **NeonButton / Toggle** — Surface fill, hairline stroke; toggles add an LED state dot (accent when on, Ghost when off); hover adds a subtle accent glow.
- **IconButton** — square Raised fill, Ghost glyph, accent glow on hover.
- **NeonSelect** — closed: Surface pill with the current item and a chevron; opens a dark menu (Panel, hairline stroke, accent on hover/selected).
- **NeonTabs** — segmented control; the active segment sits on Surface with an accent underline glow.
- **NeonMeter** — L/R bars on Inset: gradient fill cyan → yellow → red near clip, peak-hold tick, red clip latch; clicking resets peaks.
- **Graph wells** — Inset panel plus grid painters: linear grid, log-frequency grid, spectrum grid; grid lines Ghost at low alpha; curves and regions take accents with ALL-CAPS legend chips.
- **Header bar** — Panel strip: decorative traffic lights, plugin name with a 10 px Muted subtitle, A/B compare toggle, kebab menu opening a dark popup.
- **Value entry** — clicking a readout opens a dark mono input popup; Enter commits, click-away cancels.
- **Module headers** — compact (power icon + 12 px label + kebab, one row) and full (adds a mini graph and S-knob row). These are the seed of the future grid modules.

## Layout canon

1. Header bar across the top.
2. Tab strip when the plugin has views.
3. Inset graph well as the visual anchor.
4. Knob rows — M knobs primary, S knobs for trim.
5. Footer strip only when needed (state text, tiny controls).

Panels: 2–3 px radius, hairline strokes, no drop shadows; depth comes from the surface ladder. Spacing on an 8 px grid.

## Implementation approach

- `crates/neon-ui` depends only on egui 0.34 and truce-egui 6.3.0. No plugin keeps private UI helpers; shared widgets live here. DSP crates never depend on it.
- Two layers: pure painters (rect/arc/grid math, no interaction, unit-testable) and interactive widgets returning `egui::Response`; param-bound variants wrap truce params through `PluginContext`, parity with `param_knob`.
- `dark_neon_visuals()` replaces `truce_egui::theme::dark()` via `EguiEditor::with_ui(...).with_visuals(...)`.
- Editors keep timed repaints (16–33 ms) while meters or graphs animate; fully static views repaint on interaction only.
- Adoption: Bogdan and Kirya rebuild on the kit (tasks 0006/0007); Roman uses it from day one (0009); later grid modules inherit it for free.
