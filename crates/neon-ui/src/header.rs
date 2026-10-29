//! Header bar chrome — decorative traffic lights, plugin title with a Muted
//! subtitle, an A/B compare toggle, and a kebab menu opening a dark popup.

use crate::select::popup_menu;
use crate::{palette, surface};
use egui::{Align2, FontId, Pos2, Rect, Response, Sense, Ui, Vec2};

const HEIGHT: f32 = 38.0;
const KEBAB_SIZE: f32 = 26.0;
const AB_WIDTH: f32 = 44.0;
const AB_HEIGHT: f32 = 20.0;

/// Result of a header bar: the bar's hover response plus the kebab-menu pick,
/// if any, for this frame.
pub struct HeaderBarResponse {
    pub response: Response,
    pub menu_choice: Option<usize>,
}

/// Draw the header bar. `ab` selects the B side when true. `menu_items` feeds
/// the kebab popup; the chosen index arrives in `menu_choice`.
pub fn header_bar(
    ui: &mut Ui,
    id: egui::Id,
    title: &str,
    subtitle: &str,
    ab: &mut bool,
    menu_items: &[&str],
) -> HeaderBarResponse {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), HEIGHT), Sense::hover());
    surface::panel(ui.painter(), rect);

    let lights = [palette::RED, palette::YELLOW, palette::CYAN];
    for (index, color) in lights.iter().enumerate() {
        ui.painter().circle_filled(
            Pos2::new(rect.left() + 14.0 + index as f32 * 12.0, rect.center().y),
            3.5,
            *color,
        );
    }

    ui.painter().text(
        Pos2::new(rect.left() + 56.0, rect.center().y - 7.0),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.0),
        palette::TEXT_PRIMARY,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 56.0, rect.center().y + 8.0),
        Align2::LEFT_CENTER,
        subtitle,
        FontId::proportional(9.0),
        palette::TEXT_MUTED,
    );

    let kebab_rect = Rect::from_min_size(
        Pos2::new(
            rect.right() - 8.0 - KEBAB_SIZE,
            rect.top() + (HEIGHT - KEBAB_SIZE) / 2.0,
        ),
        Vec2::splat(KEBAB_SIZE),
    );
    let ab_rect = Rect::from_min_size(
        Pos2::new(
            kebab_rect.left() - 8.0 - AB_WIDTH,
            rect.top() + (HEIGHT - AB_HEIGHT) / 2.0,
        ),
        Vec2::new(AB_WIDTH, AB_HEIGHT),
    );

    paint_ab(ui, ab_rect, ab);
    let menu_choice = kebab(ui, id, kebab_rect, menu_items);

    HeaderBarResponse {
        response,
        menu_choice,
    }
}

fn paint_ab(ui: &mut Ui, rect: Rect, ab: &mut bool) {
    let response = ui.allocate_rect(rect, Sense::click());
    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        *ab = pos.x > rect.left() + rect.width() / 2.0;
    }

    let half = rect.width() / 2.0;
    for (index, label) in ["A", "B"].iter().enumerate() {
        let seg = Rect::from_min_size(
            Pos2::new(rect.left() + index as f32 * half, rect.top()),
            Vec2::new(half, rect.height()),
        );
        let active = (*ab as usize) == index;
        if active {
            ui.painter()
                .rect_filled(seg, palette::RADIUS_SMALL, palette::SURFACE);
            ui.painter().rect_filled(
                Rect::from_min_size(
                    Pos2::new(seg.left(), seg.bottom() - 1.5),
                    Vec2::new(seg.width(), 1.5),
                ),
                0.0,
                palette::ACCENT,
            );
        }
        ui.painter().text(
            seg.center(),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(9.0),
            if active {
                palette::TEXT_PRIMARY
            } else {
                palette::TEXT_MUTED
            },
        );
    }
}

fn kebab(ui: &mut Ui, id: egui::Id, rect: Rect, items: &[&str]) -> Option<usize> {
    let response = ui.allocate_rect(rect, Sense::click());
    let popup_id = id.with("kebab");
    if response.clicked() {
        egui::Popup::toggle_id(ui.ctx(), popup_id);
    }
    if ui.rect_contains_pointer(rect) {
        ui.painter()
            .rect_filled(rect, palette::RADIUS_SMALL, palette::RAISED);
    }
    let dot_color = if ui.rect_contains_pointer(rect) {
        palette::ACCENT
    } else {
        palette::TEXT_GHOST
    };
    for dot in 0..3 {
        ui.painter().circle_filled(
            Pos2::new(rect.center().x, rect.top() + 8.0 + dot as f32 * 5.0),
            1.2,
            dot_color,
        );
    }
    popup_menu(ui, popup_id, &response, 120.0, items)
}
