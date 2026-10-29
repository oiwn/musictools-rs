//! Host-free showcase for the neon-ui kit: palette, surfaces, glow, controls,
//! displays, and a Bogdan preview.
//!
//! Run with `cargo run -p neon-ui --example showcase` and compare against
//! `specs/design-sketch.png`. The shell is eframe on purpose — no plugin
//! framework. The sections live in `common/sections.rs`, shared with the
//! `snapshot` example that renders them to PNG.

#[path = "common/sections.rs"]
mod sections;

use eframe::egui::{self, RichText};
use neon_ui::palette;
use sections::{State, section_title};

/// Vertical breathing room between sections.
const SECTION_GAP: f32 = 24.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("neon-ui — Dark Neon showcase")
            .with_inner_size([980.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "neon-ui showcase",
        options,
        Box::new(|cc| {
            cc.egui_ctx
                .set_visuals(neon_ui::visuals::dark_neon_visuals());
            Ok(Box::new(Showcase {
                state: State::new(),
            }))
        }),
    )
}

struct Showcase {
    state: State,
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let s = &mut self.state;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                page_header(ui);

                section_title(ui, "PALETTE");
                sections::swatches(ui);
                ui.add_space(SECTION_GAP);

                section_title(ui, "SURFACES");
                sections::surfaces(ui);
                ui.add_space(SECTION_GAP);

                section_title(ui, "GLOW");
                sections::glow_samples(ui);
                ui.add_space(SECTION_GAP);

                section_title(ui, "WIDGETS");
                hint(
                    ui,
                    "knobs: drag vertically, Shift = fine, double-click = default; DRIVE snaps",
                );
                for part in [
                    sections::knobs,
                    sections::sliders,
                    sections::buttons,
                    sections::entry,
                ] {
                    part(ui, s);
                    ui.add_space(palette::SPACING);
                }
                ui.add_space(SECTION_GAP - palette::SPACING);

                section_title(ui, "DISPLAYS");
                sections::select_tabs(ui, s);
                ui.add_space(palette::SPACING);
                sections::meters(ui);
                ui.add_space(palette::SPACING);
                sections::graphs(ui);
                ui.add_space(SECTION_GAP);

                section_title(ui, "BOGDAN PREVIEW");
                sections::bogdan_preview(ui, s);
                ui.add_space(SECTION_GAP);
            });
        });
    }
}

fn page_header(ui: &mut egui::Ui) {
    ui.add_space(palette::SPACING);
    ui.heading(
        RichText::new("NEON-UI")
            .size(16.0)
            .strong()
            .color(palette::TEXT_PRIMARY),
    );
    ui.label(
        RichText::new("Dark Neon design system — living style guide")
            .size(10.0)
            .color(palette::TEXT_MUTED),
    );
    ui.add_space(SECTION_GAP);
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(10.0).color(palette::TEXT_MUTED));
}
