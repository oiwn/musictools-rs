//! NeonSelect — a Surface pill with a painted chevron that opens a dark popup
//! menu: Panel fill, hairline stroke, accent on hover and on the selected row.

use crate::{palette, surface};
use egui::{FontId, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

const HEIGHT: f32 = 22.0;
const PADDING: f32 = 10.0;

/// Select over `options`; `current` is the selected index.
pub fn select(ui: &mut Ui, id: egui::Id, current: &mut usize, options: &[&str]) -> Response {
    let text = options.get(*current).copied().unwrap_or("");
    let galley = ui.painter().layout(
        text.to_string(),
        FontId::proportional(10.0),
        palette::TEXT_PRIMARY,
        f32::INFINITY,
    );
    let width = galley.size().x + 2.0 * PADDING + 16.0;
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, HEIGHT), Sense::click());

    let popup_id = id.with("popup");
    if response.clicked() {
        egui::Popup::toggle_id(ui.ctx(), popup_id);
    }

    surface::surface(ui.painter(), rect);
    let open = egui::Popup::position_of_id(ui.ctx(), popup_id).is_some();
    chevron(ui.painter(), rect, open);
    ui.painter().galley(
        Pos2::new(
            rect.left() + PADDING,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        palette::TEXT_PRIMARY,
    );

    if let Some(index) = popup_menu(ui, popup_id, &response, width + 8.0, options) {
        *current = index;
        response.mark_changed();
    }
    response
}

/// The dark menu used by the select and the header kebab: rows on Panel with
/// an accent marker on the selected entry. Returns the picked row, if any.
pub(crate) fn popup_menu(
    ui: &mut Ui,
    popup_id: egui::Id,
    anchor: &Response,
    width: f32,
    items: &[&str],
) -> Option<usize> {
    let popup = egui::Popup::new(popup_id, ui.ctx().clone(), anchor, anchor.layer_id)
        .open_memory(Option::<egui::SetOpenCommand>::None)
        .kind(egui::PopupKind::Menu);
    let mut picked = None;
    popup.show(|ui| {
        ui.set_min_width(width);
        for (index, item) in items.iter().enumerate() {
            let (row_rect, row) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::click());
            if row.hovered() {
                ui.painter()
                    .rect_filled(row_rect, palette::RADIUS_SMALL, palette::SURFACE);
            }
            ui.painter().text(
                Pos2::new(row_rect.left() + 12.0, row_rect.center().y),
                egui::Align2::LEFT_CENTER,
                *item,
                FontId::proportional(10.0),
                if row.hovered() {
                    palette::TEXT_PRIMARY
                } else {
                    palette::TEXT_SECONDARY
                },
            );
            if row.clicked() {
                picked = Some(index);
            }
        }
    });
    if picked.is_some() {
        egui::Popup::close_id(ui.ctx(), popup_id);
    }
    picked
}

/// Small chevron from line segments: closed points down, open points up.
fn chevron(painter: &Painter, rect: Rect, open: bool) {
    let cx = rect.right() - 9.0;
    let cy = rect.center().y;
    let (left, tip, right) = if open {
        (
            Pos2::new(cx - 3.0, cy + 1.0),
            Pos2::new(cx, cy - 2.0),
            Pos2::new(cx + 3.0, cy + 1.0),
        )
    } else {
        (
            Pos2::new(cx - 3.0, cy - 1.0),
            Pos2::new(cx, cy + 2.0),
            Pos2::new(cx + 3.0, cy - 1.0),
        )
    };
    let stroke = Stroke::new(1.0_f32, palette::TEXT_GHOST);
    painter.line_segment([left, tip], stroke);
    painter.line_segment([tip, right], stroke);
}
