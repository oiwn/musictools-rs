# Design System — Dark Neon

Source of truth for every editor in the suite. Transcribed from the user's style-guide sketch ([design-sketch.png](design-sketch.png)); implemented as the shared `crates/neon-ui` kit on egui 0.37, used by nice-plug-egui editors. The sketch is canonical where a rendered editor and this file disagree.

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

Strokes `#26313D`/`#313E4B` for hairline panel and grid lines. Corner radius 2–3 px. Glow is layered strokes/fills of an accent at falling alpha — no blur passes. Surfaces may be top-lit: a vertical gradient from the base lifted 6% toward white down to the base (`surface::*_lit`); insets carry a soft shadow under their top edge. The active blue is a gradient from lifted Blue to Blue sunk 35% toward BG; tab selections use Blue sunk 70–84% toward Inset.

## Typography

- Font: Inter Medium (body, labels) and Inter SemiBold (headings, `fonts::heading`), bundled in `crates/neon-ui/assets/fonts/` and installed with `neon_ui::fonts::install(ctx)`; egui's fonts stay as glyph fallback. Inter is SIL OFL 1.1: free in commercial and subscription products; `OFL.txt` ships with it and the plugin credits mention it; never sell the font alone.
- Labels: 10–11 px, ALL CAPS, 0.6–0.8 px tracking (`text::tracked`), Secondary/Muted.
- Value readouts: plain Primary text under knobs, with the param's unit; editable readouts use a monospace inset pill.
- Headings: SemiBold 14–16 px Primary; sub-labels 10 px Muted.

## Widgets

Every widget is fully custom-painted; stock egui widget visuals never show. Interactive widgets share one state ladder: idle, hover (lit Raised or a stronger glow), pressed (Inset with an accent hairline, or a held glow), active/selected (the blue gradient with a cyan glow), disabled (Muted, never Ghost — `add_enabled_ui` already fades the widget once).

- **NeonKnob** — sizes S/M/L. 4 px track ring and value arc in the plugin's accent over a glow that grows on hover and while held; top-lit dome body with a bezel; pointer line (accent while held); plain value readout below (inset pill opt-in), tracked ALL-CAPS label. Tick ring marks snap points and lights Yellow while snapping. Interaction: vertical drag, Shift = fine, double-click = default.
- **NeonSlider** — thin inset track, Blue→Cyan glowing fill, round Surface handle with a bright rim and an accent glow; double-thumb range variant filling between the thumbs.
- **NeonButton / Toggle / Switch** — lit Surface with a hairline on the state ladder; `selectable_button` shows the active gradient; toggles add an LED state dot (accent when on, Ghost when off); the pill switch slides its knob onto the active gradient.
- **IconButton** — square, on the state ladder; Secondary glyph that lights accent on hover and press.
- **NeonSelect** — closed: Surface pill with the current item and a chevron; opens a dark menu (Panel, hairline stroke, accent on hover/selected).
- **NeonTabs** — Surface strip with hairline dividers, horizontal or stacked; the active segment takes the deep-blue gradient and a glowing accent bar (below or leading); other segments light on hover and sink on press; disabled strips keep the selection in grey.
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

- `crates/neon-ui` depends only on egui 0.37; nice-plug enters only through the optional `nice-plug` feature (`bindings`). No plugin keeps private UI helpers; shared widgets live here. DSP crates never depend on it.
- Two layers: pure painters (rect/arc/grid math, no interaction, unit-testable) and interactive widgets returning `egui::Response`; `bindings::param_*` wrap them over nice-plug's `ParamSetter`.
- Editors call `egui_ctx.set_visuals(dark_neon_visuals())` and `neon_ui::fonts::install(&egui_ctx)` in `NiceEguiApp::build`.
- Editors keep timed repaints (16–33 ms) while meters or graphs animate; fully static views repaint on interaction only.
- Adoption: Bogdan and Kirya rebuild on the kit (tasks 0006/0007); Roman uses it from day one (0009); later grid modules inherit it for free.

## Tooling

- `cargo run -p neon-ui --example showcase` — the living style guide in a window.
- `cargo run -p neon-ui --example snapshot -- [scene...] [--out DIR] [--scale N] [--list]` — renders the same sections headless (egui_kittest + wgpu) to `target/snapshots/<scene>.png`; `knobs`, `buttons`, `tabs`, and `sliders` also write `<scene>-states.png` (default, hover, pressed, active, disabled). A review aid for comparing against the sketch, not a pixel-diff baseline. Missing glyphs render as tofu boxes there, as they would in a host.
