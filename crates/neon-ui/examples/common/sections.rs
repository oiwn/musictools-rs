//! Showcase sections shared by the `showcase` window and the `snapshot` CLI,
//! so both draw the same code. Each section is a plain function over a `Ui`
//! and the demo [`State`]; titles and spacing between sections belong to the
//! caller.

use eframe::egui::{
    self, Align2, Color32, FontId, Label, RichText, Sense, Shape, Stroke, StrokeKind, Vec2,
};
use neon_ui::{
    button, glow, grid, header as chrome, knob, meter, palette, select, slider, surface, tabs,
    value_entry,
};

/// Demo state: every interactive section drives plain values.
#[derive(Default)]
pub struct State {
    trim: f64,
    drive: f64,
    tone: f64,
    master: f64,
    slide: f64,
    range_low: f64,
    range_high: f64,
    soft: bool,
    link: bool,
    selected: bool,
    switch_on: bool,
    switch_off: bool,
    entry: f64,
    mode: usize,
    tab: usize,
    preview_on: bool,
    preview_ab: bool,
    preview_drive: f64,
    preview_ceiling: f64,
    preview_mix: f64,
}

impl State {
    pub fn new() -> Self {
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
            selected: true,
            switch_on: true,
            switch_off: false,
            entry: 0.42,
            mode: 0,
            tab: 0,
            preview_on: true,
            preview_ab: false,
            preview_drive: 0.55,
            preview_ceiling: 0.5,
            preview_mix: 0.3,
        }
    }
}

pub fn section_title(ui: &mut egui::Ui, title: &str) {
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

pub fn swatches(ui: &mut egui::Ui) {
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
pub fn surfaces(ui: &mut egui::Ui) {
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
}

/// Glow samples in every accent: a chip, a polyline, and an LED dot.
pub fn glow_samples(ui: &mut egui::Ui) {
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

/// Knob row: Small, snapping Medium, plain Medium, and a blue Large.
pub fn knobs(ui: &mut egui::Ui, s: &mut State) {
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
}

/// Single slider and range slider.
pub fn sliders(ui: &mut egui::Ui, s: &mut State) {
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
}

/// Push button, selectable button, LED toggles, switch, and icon buttons.
pub fn buttons(ui: &mut egui::Ui, s: &mut State) {
    ui.horizontal(|ui| {
        let _ = button::button(ui, "bake");
        if button::selectable_button(ui, s.selected, "active").clicked() {
            s.selected = !s.selected;
        }
        let _ = button::toggle(ui, &mut s.soft, "soft");
        let _ = button::toggle(ui, &mut s.link, "link");
        let _ = button::switch(ui, &mut s.switch_on);
        let _ = button::switch(ui, &mut s.switch_off);
        let _ = button::icon_button(ui, "×");
        let _ = button::icon_button(ui, "●");
    });
}

/// Click-to-edit value readout.
pub fn entry(ui: &mut egui::Ui, s: &mut State) {
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
}

/// Closed select, horizontal tabs, and the stacked tab variant.
pub fn select_tabs(ui: &mut egui::Ui, s: &mut State) {
    ui.horizontal(|ui| {
        let id = ui.make_persistent_id("demo-select");
        let _ = select::select(ui, id, &mut s.mode, &["clean", "tape", "tube"]);
        ui.add_space(palette::SPACING);
        tabs::tabs(ui, &mut s.tab, &["main", "mix", "scope"]);
        ui.add_space(palette::SPACING);
        // Stacked variant: fills the height of the column it stands beside.
        let height = knob::KnobSize::Medium.footprint().y;
        let _ = tabs::tabs_vertical(ui, &mut s.tab, &["main", "mix", "scope"], height);
    });
}

/// Animated meter: wobbling levels with an occasional clip; click resets.
pub fn meters(ui: &mut egui::Ui) {
    let t = ui.input(|input| input.time);
    let left = -26.0 + 20.0 * (t * 1.7).sin() + 4.0 * (t * 7.3).sin();
    let right = -30.0 + 18.0 * (t * 1.3 + 1.0).sin() + 4.0 * (t * 5.1).sin();
    let left = if (t * 0.7).sin() > 0.95 { 0.0 } else { left };
    let right = if (t * 0.9 + 2.0).sin() > 0.96 {
        0.0
    } else {
        right
    };
    let id = ui.make_persistent_id("demo-meter");
    let _ = meter::meter(ui, id, [left, right]);
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(16));
    hint(
        ui,
        "meter — animated; click to reset peaks and the clip latch",
    );
}

/// Grid wells: linear, log-frequency, and spectrum.
pub fn graphs(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let gap = palette::SPACING;
        let width = ((ui.available_width() - 2.0 * gap) / 3.0).max(80.0);
        for (index, kind) in ["linear", "log freq", "spectrum"].iter().enumerate() {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 96.0), Sense::hover());
            surface::inset(ui.painter(), rect);
            let inner = rect.shrink(3.0);
            match index {
                0 => grid::paint_linear(ui.painter(), inner, 16.0, 16.0),
                1 => grid::paint_log_freq(ui.painter(), inner, 20.0, 20_000.0),
                _ => grid::paint_spectrum(ui.painter(), inner, 20.0, 20_000.0, -60, 0),
            }
            caption(
                ui,
                rect.center(),
                Align2::CENTER_CENTER,
                kind,
                palette::TEXT_GHOST,
            );
            if index < 2 {
                ui.add_space(gap);
            }
        }
    });
}

/// A static composition of the kit per the layout canon — the shape of the
/// Bogdan editor rebuild (0006).
pub fn bogdan_preview(ui: &mut egui::Ui, s: &mut State) {
    // Vertical budget: header, gap, scope well, gap, knob row, bottom pad.
    // The knob row takes the full knob footprint so labels never touch the
    // panel edge and arcs keep clear of the well.
    const HEADER: f32 = 38.0;
    const WELL: f32 = 84.0;
    const GAP: f32 = 12.0;
    let knob_row = knob::KnobSize::Medium.footprint().y;
    let panel_height = HEADER + GAP + WELL + GAP + knob_row + GAP;
    let (panel_rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), panel_height),
        Sense::hover(),
    );
    surface::panel(ui.painter(), panel_rect);

    // Kit header bar (0005 chrome); the kebab's second entry is bypass.
    let header_rect =
        egui::Rect::from_min_size(panel_rect.left_top(), Vec2::new(panel_rect.width(), HEADER));
    ui.scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
        let id = ui.make_persistent_id("bogdan-header");
        let bar = chrome::header_bar(
            ui,
            id,
            "PPC_BOGDAN",
            "clipper · wavefolder",
            &mut s.preview_ab,
            &["reset peaks", "bypass"],
        );
        if bar.menu_choice == Some(1) {
            s.preview_on = !s.preview_on;
        }
    });

    // The scope well: an inset placeholder until graphs land (0006).
    let well = egui::Rect::from_min_max(
        egui::pos2(panel_rect.left() + GAP, panel_rect.top() + HEADER + GAP),
        egui::pos2(
            panel_rect.right() - GAP,
            panel_rect.top() + HEADER + GAP + WELL,
        ),
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
        egui::pos2(panel_rect.left() + GAP, well.bottom() + GAP),
        Vec2::new(panel_rect.width() - 2.0 * GAP, knob_row),
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
}
