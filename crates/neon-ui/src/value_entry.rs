//! Value entry — click a readout to edit it in place: a dark mono input where
//! Enter commits and click-away or Escape cancels. Implemented as an inline
//! edit (not a popup) so the egui 0.37 popup rework does not churn it.

use crate::{palette, surface};
use egui::{FontId, Response, Sense, TextEdit, Ui, Vec2};

/// Apply parsed `text` (trimmed) to `value`; commits clamp to 0..=1.
/// Returns `false` when `parse` rejects the input.
#[must_use]
pub fn commit_text(value: &mut f64, text: &str, parse: &dyn Fn(&str) -> Option<f64>) -> bool {
    match parse(text.trim()) {
        Some(parsed) => {
            *value = parsed.clamp(0.0, 1.0);
            true
        }
        None => false,
    }
}

/// An editable readout: `fmt` renders the idle value on a small inset pill,
/// clicking it swaps in a mono `TextEdit` bound to `parse`.
pub fn readout(
    ui: &mut Ui,
    id: egui::Id,
    value: &mut f64,
    fmt: &dyn Fn(f64) -> String,
    parse: &dyn Fn(&str) -> Option<f64>,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(76.0, 18.0), Sense::click());
    let editing = ui
        .ctx()
        .data(|data| data.get_temp::<bool>(id))
        .unwrap_or(false);

    if !editing {
        if response.clicked() {
            ui.ctx().data_mut(|data| {
                data.insert_temp(id, true);
                data.insert_temp(id.with("buffer"), fmt(*value));
            });
        }
        surface::inset(ui.painter(), rect);
        let galley = ui.painter().layout(
            fmt(*value),
            FontId::monospace(10.0),
            palette::TEXT_PRIMARY,
            f32::INFINITY,
        );
        ui.painter().galley(
            egui::Pos2::new(
                rect.center().x - galley.size().x / 2.0,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            palette::TEXT_PRIMARY,
        );
        response
    } else {
        let mut buffer: String = ui.ctx().data(|data| {
            data.get_temp::<String>(id.with("buffer"))
                .unwrap_or_default()
        });
        let edit = TextEdit::singleline(&mut buffer)
            .font(egui::TextStyle::Monospace)
            .desired_width(rect.width() - 4.0)
            .clip_text(true);
        let edit_response = ui.put(rect, edit);
        let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
        let escape = ui.input(|input| input.key_pressed(egui::Key::Escape));

        if escape {
            stop_editing(ui, id);
        } else if enter || edit_response.lost_focus() {
            // Invalid input simply cancels the edit; the old value stands.
            let _ = commit_text(value, &buffer, parse);
            stop_editing(ui, id);
        } else {
            ui.ctx()
                .data_mut(|data| data.insert_temp(id.with("buffer"), buffer.clone()));
        }
        edit_response
    }
}

fn stop_editing(ui: &Ui, id: egui::Id) {
    ui.ctx().data_mut(|data| data.insert_temp(id, false));
}

/// Percent formatter for demos: `42%`.
pub fn format_percent(value: f64) -> String {
    format!("{}%", (value * 100.0).round() as u16)
}

/// Percent parser companion to [`format_percent`]; accepts an optional `%`.
pub fn parse_percent(text: &str) -> Option<f64> {
    let trimmed = text.trim().trim_end_matches('%');
    trimmed.parse::<f64>().ok().map(|number| number / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_text_accepts_valid_input_and_clamps() {
        let mut value = 0.5;
        assert!(commit_text(&mut value, " 25% ", &parse_percent));
        assert!((value - 0.25).abs() < 1e-9);
        assert!(commit_text(&mut value, "900", &parse_percent));
        assert_eq!(value, 1.0);
    }

    #[test]
    fn commit_text_rejects_garbage() {
        let mut value = 0.5;
        assert!(!commit_text(&mut value, "loud", &parse_percent));
        assert!((value - 0.5).abs() < 1e-9);
    }

    #[test]
    fn percent_format_parse_round_trips() {
        for value in [0.0, 0.07, 0.25, 0.5, 0.99, 1.0] {
            let text = format_percent(value);
            assert_eq!(
                parse_percent(&text),
                Some(value),
                "round-trip failed for {text}"
            );
        }
    }
}
