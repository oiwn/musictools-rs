//! Host-free showcase for the neon-ui kit: palette, surfaces, glow, controls,
//! and a Bogdan preview.
//!
//! Run with `cargo run -p neon-ui --example showcase` and compare against
//! `specs/design-sketch.png`. The shell is eframe on purpose — no plugin
//! framework — so the showcase survives the nice-plug migration untouched
//! except for the egui version bump.

use eframe::egui::{
    self, Align2, Color32, FontId, Label, RichText, Sense, Shape, Stroke, StrokeKind, Vec2,
};
use neon_ui::{button, glow, knob, palette, slider, surface, value_entry};

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
            Ok(Box::new(Showcase::new()))
        }),
    )
}

/// Demo state: every interactive section drives plain values.
#[derive(Default)]
struct Showcase {
    trim: f64,
    drive: f64,
    tone: f64,
    master: f64,
    slide: f64,
    range_low: f64,
    range_high: f64,
    soft: bool,
    link: bool,
    entry: f64,
    preview_on: bool,
    preview_drive: f64,
    preview_ceiling: f64,
    preview_mix: f64,
}

impl Showcase {
    fn new() -> Self {
        Self {
            trim: 0.4,
            drive: 0.62,
            tone: 0.5,
            master: 0.75,
            slide: 0.45,
            range_low: 0.25,
            range_high: 0.7,
            soft: true,
            link: false,
            entry: 0.42,
            preview_on: true,
            preview_drive: 0.55,
            preview_ceiling: 0.5,
            preview_mix: 0.3,
        }
    }
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                header(ui);
                palette_section(ui);
                surface_section(ui);
                glow_section(ui);
                widgets_section(ui, self);
                bogdan_preview(ui, self);
            });
        });
    }
}

fn header(ui: &mut egui::Ui) {
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

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(
        RichText::new(title)
            .size(11.0)
            .strong()
            .color(palette::TEXT_SECONDARY)
            .extra_letter_spacing(1.5),
    );
    ui.add_space(palette::SPACING);
}

/// Small caption painted at `pos`.
fn caption(ui: &egui::Ui, pos: egui::Pos2, align: Align2, text: &str, color: Color32) {
    ui.painter()
        .text(pos, align, text, FontId::proportional(10.0), color);
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(10.0).color(palette::TEXT_MUTED));
}

fn hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

fn palette_section(ui: &mut egui::Ui) {
    section_title(ui, "PALETTE");
    swatch_group(
        ui,
        "backgrounds",
        &[
            ("BG", palette::BG),
            ("panel", palette::PANEL),
            ("surface", palette::SURFACE),
            ("raised", palette::RAISED),
            ("inset", palette::INSET),
        ],
    );
    swatch_group(
        ui,
        "accents",
        &[
            ("cyan", palette::CYAN),
            ("blue", palette::BLUE),
            ("orange", palette::ORANGE),
            ("red", palette::RED),
            ("yellow", palette::YELLOW),
        ],
    );
    swatch_group(
        ui,
        "text",
        &[
            ("primary", palette::TEXT_PRIMARY),
            ("secondary", palette::TEXT_SECONDARY),
            ("muted", palette::TEXT_MUTED),
            ("ghost", palette::TEXT_GHOST),
        ],
    );
    swatch_group(
        ui,
        "strokes",
        &[
            ("stroke", palette::STROKE),
            ("bright", palette::STROKE_BRIGHT),
        ],
    );
    ui.add_space(SECTION_GAP);
}

fn swatch_group(ui: &mut egui::Ui, name: &str, colors: &[(&str, Color32)]) {
    hint(ui, name);
    ui.horizontal_wrapped(|ui| {
        for (label, color) in colors {
            swatch(ui, label, *color);
        }
    });
}

/// One swatch chip: a small rounded color tile with its name and hex value.
fn swatch(ui: &mut egui::Ui, name: &str, color: Color32) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
        ui.painter().rect_filled(rect, palette::RADIUS_SMALL, color);
        ui.painter().rect_stroke(
            rect,
            palette::RADIUS_SMALL,
            Stroke::new(palette::HAIRLINE, palette::STROKE_BRIGHT),
            StrokeKind::Inside,
        );
        ui.label(
            RichText::new(format!("{name} {}", hex(color)))
                .size(10.0)
                .color(palette::TEXT_SECONDARY),
        );
    });
}

/// The surface ladder: a panel holding one demo rect per role.
fn surface_section(ui: &mut egui::Ui) {
    section_title(ui, "SURFACES");
    let (panel_rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 150.0), Sense::hover());
    surface::panel(ui.painter(), panel_rect);
    caption(
        ui,
        panel_rect.left_top() + Vec2::new(10.0, 8.0),
        Align2::LEFT_TOP,
        "panel — holds sections",
        palette::TEXT_MUTED,
    );

    let inset = 12.0;
    let cell_height = 72.0;
    let gap = palette::SPACING;
    let cell_width = (panel_rect.width() - 2.0 * inset - 2.0 * gap) / 3.0;
    let cells_top = panel_rect.bottom() - inset - cell_height;
    let roles = [
        ("surface — controls", 0),
        ("raised — hover, knobs", 1),
        ("inset — wells, readouts", 2),
    ];
    for (index, (name, role)) in roles.iter().enumerate() {
        let left = panel_rect.left() + inset + index as f32 * (cell_width + gap);
        let rect = egui::Rect::from_min_size(
            egui::pos2(left, cells_top),
            Vec2::new(cell_width, cell_height),
        );
        match role {
            0 => surface::surface(ui.painter(), rect),
            1 => surface::raised(ui.painter(), rect),
            _ => surface::inset(ui.painter(), rect),
        }
        caption(
            ui,
            rect.center(),
            Align2::CENTER_CENTER,
            name,
            palette::TEXT_SECONDARY,
        );
    }
    ui.add_space(SECTION_GAP);
}

/// Glow samples in every accent: a chip, a polyline, and an LED dot.
fn glow_section(ui: &mut egui::Ui) {
    section_title(ui, "GLOW");
    let accents: [(&str, Color32); 5] = [
        ("cyan", palette::CYAN),
        ("blue", palette::BLUE),
        ("orange", palette::ORANGE),
        ("red", palette::RED),
        ("yellow", palette::YELLOW),
    ];
    for (name, color) in accents {
        ui.horizontal(|ui| {
            ui.add_sized(
                [56.0, 16.0],
                Label::new(RichText::new(name).size(10.0).color(palette::TEXT_MUTED)),
            );
            glow_chip(ui, color);
            glow_curve(ui, color);
            glow_led(ui, color);
        });
        ui.add_space(4.0);
    }
    ui.add_space(SECTION_GAP);
}

/// A raised chip with an accent halo — the hover treatment from the sketch.
fn glow_chip(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(56.0, 24.0), Sense::hover());
    glow::glow_rect(ui.painter(), rect, palette::RADIUS, color, glow::LAYERS);
    surface::raised(ui.painter(), rect);
}

/// A glowing polyline — curve indicators, accent underlines.
fn glow_curve(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(72.0, 24.0), Sense::hover());
    let points = vec![
        egui::pos2(rect.left() + 8.0, rect.center().y),
        egui::pos2(rect.center().x, rect.bottom() - 8.0),
        egui::pos2(rect.right() - 8.0, rect.top() + 8.0),
    ];
    glow::glow_line(ui.painter(), &points, color, glow::LAYERS);
    ui.painter()
        .add(Shape::line(points, Stroke::new(1.5_f32, color)));
}

/// An LED dot with a radial halo — toggle state lights.
fn glow_led(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
    let center = rect.center();
    glow::glow_circle(ui.painter(), center, 4.0, color, glow::LAYERS);
    ui.painter().circle_filled(center, 4.0, color);
}

/// Every interactive control from 0004, driven by local state.
fn widgets_section(ui: &mut egui::Ui, s: &mut Showcase) {
    section_title(ui, "WIDGETS");
    hint(
        ui,
        "knobs: drag vertically, Shift = fine, double-click = default; DRIVE snaps",
    );

    ui.horizontal(|ui| {
        let small = knob::KnobParams {
            size: knob::KnobSize::Small,
            ..Default::default()
        };
        knob::knob_with(
            ui,
            &mut s.trim,
            "trim",
            &value_entry::format_percent,
            &small,
        );

        let snapped = knob::KnobParams {
            snap_points: &[0.25, 0.5, 0.75],
            ..Default::default()
        };
        knob::knob_with(
            ui,
            &mut s.drive,
            "drive",
            &value_entry::format_percent,
            &snapped,
        );

        knob::knob(ui, &mut s.tone, "tone", &value_entry::format_percent);

        let large = knob::KnobParams {
            size: knob::KnobSize::Large,
            accent: palette::BLUE,
            ..Default::default()
        };
        knob::knob_with(
            ui,
            &mut s.master,
            "master",
            &value_entry::format_percent,
            &large,
        );
    });
    ui.add_space(palette::SPACING);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            hint(ui, "slider");
            slider::slider(ui, &mut s.slide, 0.5, 200.0);
        });
        ui.vertical(|ui| {
            hint(ui, "range");
            slider::range_slider(ui, &mut s.range_low, &mut s.range_high, 200.0);
        });
    });
    ui.add_space(palette::SPACING);

    ui.horizontal(|ui| {
        let _ = button::button(ui, "bake");
        let _ = button::toggle(ui, &mut s.soft, "soft");
        let _ = button::toggle(ui, &mut s.link, "link");
        let _ = button::icon_button(ui, "×");
        let _ = button::icon_button(ui, "●");
    });
    ui.add_space(palette::SPACING);

    ui.horizontal(|ui| {
        let id = ui.make_persistent_id("entry-demo");
        let _ = value_entry::readout(
            ui,
            id,
            &mut s.entry,
            &value_entry::format_percent,
            &value_entry::parse_percent,
        );
        hint(
            ui,
            "value entry — click, type, Enter commits, click-away cancels",
        );
    });
    ui.add_space(SECTION_GAP);
}

/// A static composition of the kit per the layout canon — the shape of the
/// Bogdan editor rebuild (0006).
fn bogdan_preview(ui: &mut egui::Ui, s: &mut Showcase) {
    section_title(ui, "BOGDAN PREVIEW");
    let (panel_rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 216.0), Sense::hover());
    surface::panel(ui.painter(), panel_rect);

    ui.painter().text(
        egui::pos2(panel_rect.left() + 12.0, panel_rect.top() + 10.0),
        Align2::LEFT_TOP,
        "BOGDAN",
        FontId::proportional(14.0),
        palette::TEXT_PRIMARY,
    );
    caption(
        ui,
        egui::pos2(panel_rect.left() + 12.0, panel_rect.top() + 30.0),
        Align2::LEFT_TOP,
        "clipper · wavefolder",
        palette::TEXT_MUTED,
    );

    // Power state as an icon button in the header.
    let power_rect = egui::Rect::from_min_size(
        egui::pos2(panel_rect.right() - 38.0, panel_rect.top() + 8.0),
        Vec2::splat(26.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(power_rect), |ui| {
        if button::icon_button(ui, if s.preview_on { "●" } else { "○" }).clicked() {
            s.preview_on = !s.preview_on;
        }
    });

    // The scope well: an inset placeholder until graphs land (0005/0006).
    let well = egui::Rect::from_min_max(
        egui::pos2(panel_rect.left() + 12.0, panel_rect.top() + 44.0),
        egui::pos2(panel_rect.right() - 12.0, panel_rect.top() + 128.0),
    );
    surface::inset(ui.painter(), well);
    caption(
        ui,
        well.center(),
        Align2::CENTER_CENTER,
        if s.preview_on {
            "SCOPE — GRAPHS LAND WITH 0005/0006"
        } else {
            "BYPASSED"
        },
        palette::TEXT_GHOST,
    );

    // Knob row per the layout canon: M knobs, one accent each.
    let knobs_rect = egui::Rect::from_min_size(
        egui::pos2(panel_rect.left() + 12.0, panel_rect.bottom() - 82.0),
        Vec2::new(panel_rect.width() - 24.0, 82.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(knobs_rect), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 24.0;
            let total = 3.0 * 56.0 + 2.0 * 24.0;
            ui.add_space(((knobs_rect.width() - total) / 2.0).max(0.0));

            let ceiling_fmt = |value: f64| format!("{:.1} dB", -value * 24.0);

            knob::knob_with(
                ui,
                &mut s.preview_drive,
                "drive",
                &value_entry::format_percent,
                &knob::KnobParams::default(),
            );

            let ceiling = knob::KnobParams {
                accent: palette::CLIP,
                ..Default::default()
            };
            knob::knob_with(
                ui,
                &mut s.preview_ceiling,
                "ceiling",
                &ceiling_fmt,
                &ceiling,
            );

            let mix = knob::KnobParams {
                accent: palette::WARNING,
                ..Default::default()
            };
            knob::knob_with(
                ui,
                &mut s.preview_mix,
                "mix",
                &value_entry::format_percent,
                &mix,
            );
        });
    });
    ui.add_space(SECTION_GAP);
}
