//! Tile view helpers shared by module views: knob rows bound to a
//! module's `ParamStore`, and header lamps.

use box_core::Shared;
use egui::{Color32, Pos2, Rect, Response, Ui, UiBuilder, pos2};
use neon_ui::knob::{self, KnobParams, KnobSize};
use neon_ui::{glow, palette};

/// What a module's view draws into.
pub struct Tile<'a> {
    /// The tile below the header strip.
    pub body: Rect,
    /// The header strip right of box's power switch and label; free for a
    /// small display (Env draws its curve there). May be empty.
    pub header_extra: Rect,
    pub shared: &'a Shared,
}

/// A module's tile view.
pub type ViewFn = fn(&mut Ui, &Tile<'_>);

/// Widget rects a view places inside a body rect.
pub type LayoutFn = fn(Rect) -> Vec<Rect>;

/// The knob size every tile uses.
pub const KNOB: KnobSize = KnobSize::Small;

/// `count` knob footprints side by side, centered in `area`.
#[must_use]
pub fn knob_row(area: Rect, count: usize) -> Vec<Rect> {
    let size = KNOB.footprint();
    let total = size.x * count as f32;
    let left = area.center().x - total / 2.0;
    let top = area.center().y - size.y / 2.0;
    (0..count)
        .map(|index| Rect::from_min_size(pos2(left + size.x * index as f32, top), size))
        .collect()
}

/// A knob for param `index` in `rect`, reading and writing the store.
pub fn param_knob(
    ui: &mut Ui,
    rect: Rect,
    shared: &Shared,
    index: usize,
    accent: Color32,
) -> Response {
    let spec = shared.params.specs()[index];
    let mut value = f64::from(shared.params.get_normalized(index));
    let mut child = ui.new_child(
        UiBuilder::new()
            .scope_id(ui.scope_id().with(("knob", index)))
            .max_rect(rect),
    );
    let params = KnobParams {
        size: KNOB,
        accent,
        snap_points: &[],
        default: f64::from(spec.normalize(spec.default_value())),
        pill: false,
    };
    let format = |position: f64| spec.display(spec.denormalize(position as f32));
    let response = knob::knob_with(&mut child, &mut value, spec.name, &format, &params);
    if response.changed() {
        shared.params.set_normalized(index, value as f32);
    }
    response
}

/// Draws one knob per param, in a centered row: the whole view of most
/// small modules.
pub fn knob_view(ui: &mut Ui, tile: &Tile<'_>, params: &[usize], accent: Color32) {
    for (rect, &index) in knob_row(tile.body, params.len()).into_iter().zip(params) {
        param_knob(ui, rect, tile.shared, index, accent);
    }
}

/// A small status lamp: glowing `color` when on, a dim dot when off.
pub fn lamp(ui: &Ui, center: Pos2, radius: f32, on: bool, color: Color32) {
    let painter = ui.painter();
    if on {
        glow::glow_circle(painter, center, radius, color, 2);
        painter.circle_filled(center, radius, color);
    } else {
        painter.circle_filled(center, radius, palette::STROKE_BRIGHT);
    }
}

/// The lamp spot at the right end of the free header strip, if there is room.
#[must_use]
pub fn header_lamp_spot(tile: &Tile<'_>) -> Option<Pos2> {
    (tile.header_extra.width() > 8.0).then(|| {
        pos2(
            tile.header_extra.right() - 6.0,
            tile.header_extra.center().y,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    #[test]
    fn knob_rows_are_centered() {
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(92.0, 74.0));
        let row = knob_row(area, 2);
        assert_eq!(row[0].left(), 2.0);
        assert_eq!(row[1].right(), 90.0);
        assert_eq!(row[0].top(), 0.0);
    }
}
