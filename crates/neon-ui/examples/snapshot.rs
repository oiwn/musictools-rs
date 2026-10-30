//! Headless PNG snapshots of the neon-ui kit, for reviewing widgets against
//! `specs/design-sketch.png` without a window or a plugin host.
//!
//! ```text
//! cargo run -p neon-ui --example snapshot -- [scene...] [--out DIR] [--scale N] [--list]
//! ```
//!
//! With no scene, or `all`, every scene renders. Each scene writes
//! `<out>/<scene>.png` (default `target/snapshots/`); `knobs`, `buttons`,
//! `tabs`, and `sliders` also write `<scene>-states.png`, a strip of the widget in its
//! default, hover, pressed, active, and disabled states. Scenes draw the same
//! sections as the `showcase` window (`common/sections.rs`).
//!
//! Rendering goes through egui_kittest's offscreen wgpu path (Metal on
//! macOS). Output is a review aid, not a pixel-diff baseline: GPU output
//! differs across machines.

#[path = "common/sections.rs"]
mod sections;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eframe::egui::{self, Align2, FontId, Pos2, Rect, Vec2};
use egui_kittest::Harness;
use image::RgbaImage;
use neon_ui::{button, header as chrome, knob, palette, slider, tabs, value_entry};
use sections::{State, section_title};

/// Default device-pixel scale: retina-like, so thin strokes stay readable.
const DEFAULT_SCALE: f32 = 2.0;
/// Scene content width in logical px.
const SCENE_WIDTH: f32 = 720.0;
/// Margin the harness frame adds around the scene, plus our bottom padding.
const SCENE_PADDING: f32 = 24.0;
/// Frames run before reading back layout or rendering; covers egui's
/// first-frame sizing passes and pointer-driven state changes.
const SETTLE_STEPS: usize = 3;

type SceneFn = fn(&mut egui::Ui, &mut State);

/// Scene registry: name, section title, drawing function.
const SCENES: &[(&str, &str, SceneFn)] = &[
    ("palette", "PALETTE", |ui, _| sections::swatches(ui)),
    ("surfaces", "SURFACES", |ui, _| sections::surfaces(ui)),
    ("glow", "GLOW", |ui, _| sections::glow_samples(ui)),
    ("knobs", "KNOBS", sections::knobs),
    ("sliders", "SLIDERS", sections::sliders),
    ("buttons", "BUTTONS", sections::buttons),
    ("entry", "VALUE ENTRY", sections::entry),
    ("tabs", "SELECT + TABS", sections::select_tabs),
    ("meters", "METERS", |ui, _| sections::meters(ui)),
    ("graph", "GRAPH WELLS", |ui, _| sections::graphs(ui)),
    ("header", "HEADER BAR", header_scene),
    ("bogdan", "BOGDAN PREVIEW", sections::bogdan_preview),
];

fn main() -> ExitCode {
    let args = match Args::parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("snapshot: {message}");
            eprintln!("usage: snapshot [scene...] [--out DIR] [--scale N] [--list]");
            return ExitCode::FAILURE;
        }
    };
    if args.list {
        for (name, ..) in SCENES {
            println!("{name}");
        }
        return ExitCode::SUCCESS;
    }
    let out = match std::fs::create_dir_all(&args.out).and_then(|()| args.out.canonicalize()) {
        Ok(out) => out,
        Err(error) => {
            eprintln!("snapshot: cannot create {}: {error}", args.out.display());
            return ExitCode::FAILURE;
        }
    };

    let mut failed = false;
    for (name, title, draw) in SCENES {
        if !args.wants(name) {
            continue;
        }
        let mut outputs = vec![(name.to_string(), render_scene(title, *draw, args.scale))];
        if let Some(strip) = state_strip(name, args.scale) {
            outputs.push((format!("{name}-states"), strip));
        }
        for (file, image) in outputs {
            match image.and_then(|image| save(&image, &out, &file)) {
                Ok(path) => println!("{}", path.display()),
                Err(error) => {
                    eprintln!("snapshot: {file}: {error}");
                    failed = true;
                }
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct Args {
    scenes: Vec<String>,
    out: PathBuf,
    scale: f32,
    list: bool,
}

impl Args {
    fn parse(mut raw: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut args = Self {
            scenes: Vec::new(),
            out: default_out_dir(),
            scale: DEFAULT_SCALE,
            list: false,
        };
        while let Some(arg) = raw.next() {
            match arg.as_str() {
                "--out" => {
                    args.out = raw.next().ok_or("--out needs a directory")?.into();
                }
                "--scale" => {
                    let value = raw.next().ok_or("--scale needs a number")?;
                    args.scale = value
                        .parse::<f32>()
                        .ok()
                        .filter(|scale| scale.is_finite() && (0.5..=4.0).contains(scale))
                        .ok_or(format!("--scale wants 0.5..=4, got `{value}`"))?;
                }
                "--list" => args.list = true,
                "all" => {}
                flag if flag.starts_with("--") => return Err(format!("unknown flag `{flag}`")),
                scene if SCENES.iter().any(|(name, ..)| *name == scene) => {
                    args.scenes.push(arg);
                }
                scene => {
                    let known: Vec<_> = SCENES.iter().map(|(name, ..)| *name).collect();
                    return Err(format!(
                        "unknown scene `{scene}`; known: all, {}",
                        known.join(", ")
                    ));
                }
            }
        }
        Ok(args)
    }

    /// No scenes named (or only `all`) renders everything.
    fn wants(&self, name: &str) -> bool {
        self.scenes.is_empty() || self.scenes.iter().any(|scene| scene == name)
    }
}

/// `target/snapshots` at the workspace root, independent of the cwd.
fn default_out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/snapshots")
}

/// Write `image` flattened onto the window background: egui leaves unpainted
/// pixels transparent, which image viewers show as white or checkerboard.
fn save(image: &RgbaImage, dir: &Path, name: &str) -> Result<PathBuf, String> {
    let mut flat = RgbaImage::from_pixel(
        image.width(),
        image.height(),
        image::Rgba([palette::BG.r(), palette::BG.g(), palette::BG.b(), 255]),
    );
    image::imageops::overlay(&mut flat, image, 0, 0);
    let path = dir.join(format!("{name}.png"));
    flat.save(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

fn harness<'a>(
    size: Vec2,
    scale: f32,
    app: impl FnMut(&mut egui::Ui, &mut Probe) + 'a,
) -> Harness<'a, Probe> {
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(scale)
        .with_theme(egui::Theme::Dark)
        .with_accessibility_check(false)
        // Draw missing glyphs as tofu instead of panicking: a tofu box in a
        // snapshot is the finding, the same box a plugin host would show.
        .allow_missing_glyphs()
        // States come from injected pointer events; the drawn cursor would
        // only cover the widget under review.
        .with_render_cursor(false)
        .with_context_setup(|ctx| {
            ctx.set_visuals(neon_ui::visuals::dark_neon_visuals());
            neon_ui::fonts::install(ctx);
        })
        .build_ui_state(app, Probe::default())
}

/// Per-harness scratch: demo state plus layout read back after a frame.
struct Probe {
    state: State,
    /// Bottom edge of the drawn content, for sizing the image.
    bottom: f32,
    /// The widget a state strip points the cursor at.
    target: Rect,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            state: State::new(),
            bottom: 0.0,
            target: Rect::NOTHING,
        }
    }
}

/// One scene: section title plus its drawing function, cropped to height.
fn render_scene(title: &str, draw: SceneFn, scale: f32) -> Result<RgbaImage, String> {
    let mut harness = harness(Vec2::new(SCENE_WIDTH, 2_000.0), scale, |ui, probe| {
        ui.vertical(|ui| {
            section_title(ui, title);
            draw(ui, &mut probe.state);
        });
        // min_rect also covers children placed with explicit rects.
        probe.bottom = ui.min_rect().bottom();
    });
    harness.run_steps(SETTLE_STEPS);
    let height = harness.state().bottom + SCENE_PADDING;
    harness.set_size(Vec2::new(SCENE_WIDTH, height));
    harness.run_steps(SETTLE_STEPS);
    harness.render()
}

fn header_scene(ui: &mut egui::Ui, _: &mut State) {
    let id = ui.make_persistent_id("snapshot-header");
    let mut ab = false;
    let _ = chrome::header_bar(
        ui,
        id,
        "BOGDAN",
        "detail-preserving limiter  v0.3.0",
        &mut ab,
        &["init patch", "reset to default"],
    );
}

/// Interaction a strip cell puts the widget in.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Default,
    Hover,
    Pressed,
    /// The widget's own "on" state: a toggle on, a knob mid-drag.
    Active,
    Disabled,
}

impl Look {
    const ALL: [Self; 5] = [
        Self::Default,
        Self::Hover,
        Self::Pressed,
        Self::Active,
        Self::Disabled,
    ];

    fn caption(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

/// Widget drawn in one strip cell; returns its rect for the cursor.
type CellFn = fn(&mut egui::Ui, Look) -> Rect;

/// A strip of state cells for scenes that have one.
fn state_strip(scene: &str, scale: f32) -> Option<Result<RgbaImage, String>> {
    let (cell, draw): (Vec2, CellFn) = match scene {
        "knobs" => (Vec2::new(110.0, 130.0), knob_cell),
        "buttons" => (Vec2::new(110.0, 70.0), button_cell),
        "tabs" => (Vec2::new(200.0, 140.0), tabs_cell),
        "sliders" => (Vec2::new(170.0, 60.0), slider_cell),
        _ => return None,
    };
    Some(render_strip(cell, draw, scale))
}

fn render_strip(cell: Vec2, draw: CellFn, scale: f32) -> Result<RgbaImage, String> {
    let cells = Look::ALL
        .iter()
        .map(|&look| render_cell(cell, draw, look, scale))
        .collect::<Result<Vec<_>, _>>()?;
    let width = cells.iter().map(RgbaImage::width).sum();
    let height = cells.iter().map(RgbaImage::height).max().unwrap_or(0);
    let mut strip = RgbaImage::new(width, height);
    let mut x = 0;
    for image in &cells {
        image::imageops::replace(&mut strip, image, i64::from(x), 0);
        x += image.width();
    }
    Ok(strip)
}

fn render_cell(cell: Vec2, draw: CellFn, look: Look, scale: f32) -> Result<RgbaImage, String> {
    let mut harness = harness(cell, scale, |ui, probe| {
        let rect = ui.max_rect();
        let target = ui
            .add_enabled_ui(look != Look::Disabled, |ui| draw(ui, look))
            .inner;
        probe.target = target;
        ui.painter().text(
            Pos2::new(rect.center().x, rect.bottom() - 4.0),
            Align2::CENTER_BOTTOM,
            look.caption(),
            FontId::proportional(11.0),
            palette::TEXT_SECONDARY,
        );
    });
    harness.run_steps(SETTLE_STEPS);
    let center = harness.state().target.center();
    if matches!(look, Look::Hover | Look::Pressed | Look::Active) {
        harness.hover_at(center);
    }
    if matches!(look, Look::Pressed | Look::Active) {
        // Press without release: the widget stays held for the frame.
        harness.drag_at(center);
    }
    if look == Look::Active {
        // A small upward drag so held-and-moving widgets show their drag look.
        harness.hover_at(center - Vec2::new(0.0, 12.0));
    }
    harness.run_steps(SETTLE_STEPS);
    harness.render()
}

/// Medium knob centered in the cell.
fn knob_cell(ui: &mut egui::Ui, _look: Look) -> Rect {
    let mut value = 0.62;
    centered(ui, knob::KnobSize::Medium.footprint(), |ui| {
        knob::knob_with(
            ui,
            &mut value,
            "drive",
            &value_entry::format_percent,
            &knob::KnobParams {
                snap_points: &[0.25, 0.5, 0.75],
                ..Default::default()
            },
        )
        .rect
    })
}

/// Push button; the active cell shows it selected.
fn button_cell(ui: &mut egui::Ui, look: Look) -> Rect {
    centered(ui, Vec2::new(78.0, 28.0), |ui| {
        button::selectable_button(ui, look == Look::Active, "bake").rect
    })
}

/// Vertical tab stack, the shape Bogdan uses for Mode and Shape.
fn tabs_cell(ui: &mut egui::Ui, _look: Look) -> Rect {
    let labels = ["clean", "detail", "fold"];
    let height = knob::KnobSize::Medium.footprint().y;
    let size = Vec2::new(tabs::tabs_vertical_width(&labels), height);
    let mut current = 1;
    let rect = centered(ui, size, |ui| {
        tabs::tabs_vertical(ui, &mut current, &labels, height).rect
    });
    // Point at the last segment: hovering the selected one shows nothing new.
    Rect::from_min_max(
        Pos2::new(rect.left(), rect.bottom() - height / labels.len() as f32),
        rect.max,
    )
}

/// Single slider; the cursor targets its handle.
fn slider_cell(ui: &mut egui::Ui, _look: Look) -> Rect {
    let mut value = 0.6;
    let rect = centered(ui, Vec2::new(140.0, 22.0), |ui| {
        slider::slider(ui, &mut value, 0.5, 140.0).rect
    });
    let x = rect.left() + 7.0 + 0.6 * (rect.width() - 14.0);
    Rect::from_center_size(Pos2::new(x, rect.center().y), Vec2::splat(14.0))
}

/// Run `add` in a child ui of `size` centered in the cell above the caption.
fn centered(ui: &mut egui::Ui, size: Vec2, add: impl FnOnce(&mut egui::Ui) -> Rect) -> Rect {
    let area = ui.max_rect();
    let rect = Rect::from_center_size(
        Pos2::new(area.center().x, area.top() + 4.0 + size.y / 2.0),
        size,
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .scope_id(ui.scope_id().with("cell"))
            .max_rect(rect),
    );
    add(&mut child)
}
