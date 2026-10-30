//! NeonButton, selectable button, LED toggle, switch, and IconButton — all on
//! one state ladder: idle (lit Surface), hover (lit Raised), pressed (Inset
//! with an accent hairline), disabled (flat, Muted text). Selected buttons
//! take the blue gradient with a cyan glow, the sketch's "Active".

use crate::{glow, palette, surface, text};
use egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};

/// Interaction state of a clickable control, derived from its response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Hovered,
    Pressed,
    Disabled,
}

impl ButtonState {
    /// The state `response` is in this frame.
    #[must_use]
    pub fn of(ui: &Ui, response: &Response) -> Self {
        if !ui.is_enabled() {
            Self::Disabled
        } else if response.is_pointer_button_down_on() {
            Self::Pressed
        } else if response.hovered() {
            Self::Hovered
        } else {
            Self::Idle
        }
    }

    /// Label color for this state; `add_enabled_ui` already fades disabled
    /// widgets, so Muted (not Ghost) keeps them readable.
    #[must_use]
    pub fn text_color(self) -> Color32 {
        match self {
            Self::Disabled => palette::TEXT_MUTED,
            _ => palette::TEXT_PRIMARY,
        }
    }
}

const LABEL_SIZE: f32 = 11.0;
const LABEL_TRACKING: f32 = 0.6;

/// Push button with an ALL-CAPS label; check `Response::clicked`.
pub fn button(ui: &mut Ui, label: &str) -> Response {
    selectable_button(ui, false, label)
}

/// Button that shows a selected state: the blue gradient, cyan glow, and
/// bright stroke. The caller flips `selected` on `clicked()`.
pub fn selectable_button(ui: &mut Ui, selected: bool, label: &str) -> Response {
    let galley = label_galley(ui.painter(), label);
    let (rect, response) =
        ui.allocate_exact_size(galley.size() + Vec2::new(28.0, 14.0), Sense::click());
    let state = ButtonState::of(ui, &response);
    paint_body(ui.painter(), rect, state, selected);
    let nudge = if state == ButtonState::Pressed {
        0.5
    } else {
        0.0
    };
    ui.painter().galley(
        Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0 + nudge,
        ),
        galley,
        state.text_color(),
    );
    response
}

/// Toggle with an LED dot: accent when on, Ghost when off.
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let galley = label_galley(ui.painter(), label);
    let led = 6.0;
    let size = Vec2::new(12.0 + led + 8.0 + galley.size().x + 12.0, 26.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    let state = ButtonState::of(ui, &response);
    paint_body(ui.painter(), rect, state, false);
    let led_center = Pos2::new(rect.left() + 12.0 + led / 2.0, rect.center().y);
    let lit = *on && state != ButtonState::Disabled;
    if lit {
        glow::glow_circle(ui.painter(), led_center, led / 2.0, palette::ACCENT, 2);
    }
    ui.painter().circle_filled(
        led_center,
        led / 2.0,
        if lit {
            palette::ACCENT
        } else {
            palette::TEXT_GHOST
        },
    );
    ui.painter().galley(
        Pos2::new(
            rect.left() + 12.0 + led + 8.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        if *on {
            state.text_color()
        } else {
            palette::TEXT_SECONDARY
        },
    );
    response
}

/// Pill switch: the knob slides right and the track takes the blue gradient
/// when on.
pub fn switch(ui: &mut Ui, on: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(38.0, 20.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let state = ButtonState::of(ui, &response);
    let painter = ui.painter();
    let radius = rect.height() / 2.0;
    let enabled = state != ButtonState::Disabled;

    if *on && enabled {
        glow::glow_rect(painter, rect, radius, palette::ACCENT, 2);
        active_fill(painter, rect, radius);
    } else {
        painter.rect_filled(rect, radius, palette::INSET);
    }
    let stroke = match state {
        ButtonState::Hovered | ButtonState::Pressed => palette::STROKE_BRIGHT,
        _ if *on && enabled => palette::CYAN.gamma_multiply(0.7),
        _ => palette::STROKE,
    };
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(palette::HAIRLINE, stroke),
        StrokeKind::Inside,
    );

    let knob_x = if *on {
        rect.right() - radius
    } else {
        rect.left() + radius
    };
    let knob = Pos2::new(knob_x, rect.center().y);
    let knob_color = match (enabled, *on) {
        (false, _) => palette::TEXT_MUTED,
        (true, true) => palette::TEXT_PRIMARY,
        (true, false) => palette::TEXT_SECONDARY,
    };
    painter.circle_filled(knob, radius - 3.0, knob_color);
    response
}

/// Square icon button on the same ladder; the glyph lights accent on hover
/// and press.
pub fn icon_button(ui: &mut Ui, glyph: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
    let state = ButtonState::of(ui, &response);
    paint_body(ui.painter(), rect, state, false);
    let color = match state {
        ButtonState::Hovered | ButtonState::Pressed => palette::ACCENT,
        ButtonState::Idle => palette::TEXT_SECONDARY,
        ButtonState::Disabled => palette::TEXT_MUTED,
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(13.0),
        color,
    );
    response
}

/// The blue gradient behind selected buttons and switched-on switches.
pub fn active_fill(painter: &Painter, rect: Rect, radius: f32) {
    surface::gradient_rect(
        painter,
        rect,
        radius,
        surface::lift(palette::BLUE),
        palette::BLUE.lerp_to_gamma(palette::BG, 0.35),
    );
}

/// Body fill and stroke for one ladder state; `selected` overrides the fill
/// with the active gradient while enabled.
pub fn paint_body(painter: &Painter, rect: Rect, state: ButtonState, selected: bool) {
    let radius = palette::RADIUS;
    if selected && state != ButtonState::Disabled {
        glow::glow_rect(painter, rect, radius, palette::ACCENT, 2);
        active_fill(painter, rect, radius);
        let stroke = if state == ButtonState::Pressed {
            palette::CYAN.gamma_multiply(0.5)
        } else {
            palette::CYAN.gamma_multiply(0.8)
        };
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(palette::HAIRLINE, stroke),
            StrokeKind::Inside,
        );
        return;
    }
    match state {
        ButtonState::Idle => surface::surface_lit(painter, rect),
        ButtonState::Hovered => surface::raised_lit(painter, rect),
        ButtonState::Pressed => {
            surface::inset(painter, rect);
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(palette::HAIRLINE, palette::ACCENT.gamma_multiply(0.5)),
                StrokeKind::Inside,
            );
        }
        ButtonState::Disabled => surface::surface(painter, rect),
    }
}

/// Laid out with the placeholder color so `Painter::galley`'s color argument
/// sets the state color at paint time.
fn label_galley(painter: &Painter, label: &str) -> std::sync::Arc<egui::Galley> {
    text::tracked_galley(
        painter,
        &label.to_uppercase(),
        FontId::proportional(LABEL_SIZE),
        Color32::PLACEHOLDER,
        LABEL_TRACKING,
    )
}
