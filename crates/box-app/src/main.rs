//! box — the standalone modular grid app.
//!
//! `cargo run -p box-app --release [-- patch.json]`
//!
//! Opens the patch (or the default gated-sine patch) on the cell grid and
//! plays it through the default output device. ⌘S saves to the patch path,
//! ⌘O reloads it. Drag a tile's header to move it; drag empty space to pan.
//! Drag from a port to another port to wire them; click a wire to remove
//! it; click a header to select a tile and press Delete to remove it.
//! Right-click empty canvas to search for a module and place it.

mod audio;
mod browser;
mod canvas;
mod edit;
mod patch_io;

use std::path::PathBuf;
use std::time::Duration;

use box_dsp::{Controller, Patch, new_engine};
use browser::Browser;
use canvas::Canvas;
use egui::{Align2, FontId, Key, pos2};
use neon_ui::palette;

/// Largest block the engine renders in one go.
const MAX_BLOCK: usize = 1024;
/// Sample rate used when no device opens (the UI still runs).
const FALLBACK_RATE: u32 = 48_000;
/// How long a status message stays up.
const MESSAGE_SECONDS: f64 = 4.0;

fn main() -> eframe::Result {
    let path = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from(patch_io::DEFAULT_PATH), PathBuf::from);
    let device = audio::open();
    let sample_rate = device
        .as_ref()
        .map_or(FALLBACK_RATE, |device| device.info.sample_rate);
    let (controller, engine) = new_engine(box_modules::registry, sample_rate as f32, MAX_BLOCK);
    let audio = device.and_then(|device| audio::start(device, engine));

    let (patch, message) = if path.exists() {
        match patch_io::load(&path) {
            Ok(patch) => (patch, format!("loaded {}", path.display())),
            Err(error) => (patch_io::demo_patch(), error),
        }
    } else {
        (patch_io::demo_patch(), "demo patch".to_owned())
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("box")
            .with_inner_size([1280.0, 520.0 + 2.0 * 96.0]),
        ..Default::default()
    };
    eframe::run_native(
        "box",
        options,
        Box::new(move |cc| {
            cc.egui_ctx
                .set_visuals(neon_ui::visuals::dark_neon_visuals());
            neon_ui::fonts::install(&cc.egui_ctx);
            let mut app = BoxApp {
                patch,
                path,
                controller,
                audio,
                canvas: Canvas::default(),
                browser: Browser::default(),
                message: String::new(),
                message_at: 0.0,
                dirty: false,
            };
            app.apply();
            app.say(&cc.egui_ctx, message);
            Ok(Box::new(app))
        }),
    )
}

struct BoxApp {
    patch: Patch,
    path: PathBuf,
    controller: Controller,
    audio: Result<audio::Audio, String>,
    canvas: Canvas,
    browser: Browser,
    message: String,
    message_at: f64,
    /// Unsaved changes.
    dirty: bool,
}

impl BoxApp {
    fn say(&mut self, ctx: &egui::Context, message: impl Into<String>) {
        self.message = message.into();
        self.message_at = ctx.input(|input| input.time);
    }

    fn apply(&mut self) -> Option<String> {
        self.controller
            .apply(&self.patch)
            .err()
            .map(|error| error.to_string())
    }

    fn save(&mut self) -> String {
        self.controller.capture(&mut self.patch);
        match patch_io::save(&self.path, &self.patch) {
            Ok(()) => {
                self.dirty = false;
                format!("saved {}", self.path.display())
            }
            Err(error) => error,
        }
    }

    fn reload(&mut self) -> String {
        match patch_io::load(&self.path) {
            Ok(patch) => {
                self.patch = patch;
                self.controller.forget_instances();
                self.dirty = false;
                self.apply()
                    .unwrap_or_else(|| format!("loaded {}", self.path.display()))
            }
            Err(error) => error,
        }
    }

    fn status_line(&self, ui: &mut egui::Ui, now: f64) {
        let rect = ui.max_rect();
        let painter = ui.painter();
        let left = match &self.audio {
            Ok(audio) => format!(
                "{}  ·  {} Hz  ·  {} ch  ·  block {}  ·  load {:.1}%",
                audio.info.device,
                audio.info.sample_rate,
                audio.info.channels,
                audio.stats.frames(),
                audio.stats.load() * 100.0
            ),
            Err(error) => format!("no audio: {error}"),
        };
        let font = FontId::proportional(11.0);
        painter.text(
            pos2(rect.left() + 8.0, rect.center().y),
            Align2::LEFT_CENTER,
            left,
            font.clone(),
            palette::TEXT_SECONDARY,
        );
        let mut right = format!(
            "{}{}  ·  ⌘S save  ⌘O reload",
            self.path.display(),
            if self.dirty { " *" } else { "" }
        );
        if now - self.message_at < MESSAGE_SECONDS && !self.message.is_empty() {
            right = format!("{}   {right}", self.message);
        }
        painter.text(
            pos2(rect.right() - 8.0, rect.center().y),
            Align2::RIGHT_CENTER,
            right,
            font,
            palette::TEXT_MUTED,
        );
    }
}

impl eframe::App for BoxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Meters and lamps move with the audio.
        ctx.request_repaint_after(Duration::from_millis(16));
        self.controller.collect();

        let (save, reload) = ctx.input(|input| {
            (
                input.modifiers.command && input.key_pressed(Key::S),
                input.modifiers.command && input.key_pressed(Key::O),
            )
        });
        if save {
            let message = self.save();
            self.say(&ctx, message);
        }
        if reload {
            let message = self.reload();
            self.say(&ctx, message);
        }

        let now = ctx.input(|input| input.time);
        egui::Panel::bottom("status")
            .exact_size(24.0)
            .frame(egui::Frame::new().fill(palette::PANEL))
            .show(ui, |ui| self.status_line(ui, now));
        let outcome = egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette::BG))
            .show(ui, |ui| {
                self.canvas.show(ui, &mut self.patch, &mut self.controller)
            })
            .inner;
        if outcome.moved {
            self.dirty = true;
        }
        if let Some((at, cell)) = outcome.browse {
            self.browser.open(at, cell);
        }
        let placed = self.browser.show(&ctx, &mut self.patch);
        if outcome.repatch || placed {
            self.dirty = true;
            if let Some(error) = self.apply() {
                self.say(&ctx, error);
            }
        }
        if let Some(hint) = outcome.hint {
            self.say(&ctx, hint);
        }
    }
}
