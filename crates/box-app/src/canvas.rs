//! The cell-grid canvas: tiles with a box-drawn frame, wires, port nubs,
//! panning, moving tiles by their header, wiring by dragging between nubs,
//! clicking a wire to remove it, and selecting and deleting tiles.

use std::sync::Arc;

use box_core::geometry::{self, CELL};
use box_core::{ModuleDescriptor, PortSpec, Shared, Side};
use box_dsp::{Controller, Patch, PatchNode};
use box_modules::{ModuleEntry, Tile};
use egui::epaint::CubicBezierShape;
use egui::{
    Align2, Color32, FontId, Key, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, pos2, vec2,
};
use neon_ui::{fonts, glow, palette, surface, text};

use crate::edit::{self, PortEnd};

/// Port nub radius.
const NUB: f32 = 4.0;
/// Pointer distance that still hits a nub.
const NUB_HIT: f32 = NUB + 5.0;
/// Pointer distance that still hits a wire.
const WIRE_HIT: f32 = 6.0;
/// Power lamp radius and its hit box.
const LAMP: f32 = 3.5;
const LAMP_HIT: f32 = 14.0;

/// Pixel position of a cell's top-left corner.
#[must_use]
pub fn cell_origin(origin: Pos2, cell: (i32, i32)) -> Pos2 {
    origin + vec2(cell.0 as f32 * CELL, cell.1 as f32 * CELL)
}

/// The cell a tile whose top-left sits at `top_left` snaps to.
#[must_use]
pub fn nearest_cell(origin: Pos2, top_left: Pos2) -> (i32, i32) {
    let offset = top_left - origin;
    (
        (offset.x / CELL).round() as i32,
        (offset.y / CELL).round() as i32,
    )
}

/// A node's tile size; unknown kinds draw as a 1x1 placeholder.
#[must_use]
pub fn node_size(kind: &str) -> (u8, u8) {
    box_modules::registry(kind).map_or((1, 1), |descriptor| descriptor.size)
}

/// True when two cell rectangles share a cell.
#[must_use]
pub fn cells_overlap(a: (i32, i32), a_size: (u8, u8), b: (i32, i32), b_size: (u8, u8)) -> bool {
    a.0 < b.0 + i32::from(b_size.0)
        && b.0 < a.0 + i32::from(a_size.0)
        && a.1 < b.1 + i32::from(b_size.1)
        && b.1 < a.1 + i32::from(a_size.1)
}

/// True when node `uid` placed at `cell` would overlap another tile.
#[must_use]
pub fn overlaps(patch: &Patch, uid: u64, cell: (i32, i32)) -> bool {
    let Some(node) = patch.node(uid) else {
        return false;
    };
    let size = node_size(&node.kind);
    patch
        .nodes
        .iter()
        .filter(|other| other.uid != uid)
        .any(|other| cells_overlap(cell, size, other.cell, node_size(&other.kind)))
}

/// A tile's rect for a size, with its first cell's corner at `top_left`.
#[must_use]
pub fn tile_rect(top_left: Pos2, size: (u8, u8)) -> Rect {
    to_rect(top_left, geometry::tile_area(size))
}

/// A port nub's pixel position.
#[must_use]
pub fn anchor(top_left: Pos2, descriptor: &ModuleDescriptor, port: &PortSpec) -> Pos2 {
    let (x, y) = geometry::port_anchor(descriptor, port);
    top_left + vec2(x, y)
}

fn to_rect(top_left: Pos2, area: geometry::Area) -> Rect {
    Rect::from_min_size(top_left + vec2(area.x, area.y), vec2(area.w, area.h))
}

fn side_direction(side: Side) -> Vec2 {
    match side {
        Side::Left => vec2(-1.0, 0.0),
        Side::Right => vec2(1.0, 0.0),
        Side::Top => vec2(0.0, -1.0),
        Side::Bottom => vec2(0.0, 1.0),
    }
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Left => Side::Right,
        Side::Right => Side::Left,
        Side::Top => Side::Bottom,
        Side::Bottom => Side::Top,
    }
}

/// The wire between two nubs as a cubic bezier leaving each along its side.
#[must_use]
pub fn wire(from: Pos2, from_side: Side, to: Pos2, to_side: Side) -> [Pos2; 4] {
    let reach = ((to - from).length() * 0.4).clamp(24.0, 160.0);
    [
        from,
        from + side_direction(from_side) * reach,
        to + side_direction(to_side) * reach,
        to,
    ]
}

/// A wire's bezier flattened to a polyline for drawing and hit-testing.
fn flatten(points: [Pos2; 4]) -> Vec<Pos2> {
    CubicBezierShape::from_points_stroke(points, false, Color32::TRANSPARENT, Stroke::NONE)
        .flatten(Some(0.5))
}

/// What the canvas changed this frame.
#[derive(Debug, Default)]
pub struct Outcome {
    /// A tile moved (the patch needs saving, not recompiling).
    pub moved: bool,
    /// The graph changed (power, wires, deleted tiles): re-apply the patch.
    pub repatch: bool,
    /// A short message for the status line.
    pub hint: Option<&'static str>,
    /// Right-click on empty canvas: open the module browser at this screen
    /// position for this cell.
    pub browse: Option<(Pos2, (i32, i32))>,
}

/// The cell under a screen position.
#[must_use]
pub fn cell_at(origin: Pos2, point: Pos2) -> (i32, i32) {
    let offset = point - origin;
    (
        (offset.x / CELL).floor() as i32,
        (offset.y / CELL).floor() as i32,
    )
}

/// A tile being dragged by its header.
#[derive(Clone, Copy, Debug)]
struct TileDrag {
    uid: u64,
    /// Pointer position minus the tile's top-left at grab time.
    grab: Vec2,
}

/// A wire being drawn from a nub.
#[derive(Clone, Copy, Debug)]
struct Wiring {
    from: PortEnd,
    /// The loose end relative to the starting nub (follows the pointer).
    /// Relative, so the live wire stays attached if the view pans.
    reach: Vec2,
}

/// Canvas view state.
#[derive(Debug, Default)]
pub struct Canvas {
    pub pan: Vec2,
    drag: Option<TileDrag>,
    selected: Option<u64>,
    wiring: Option<Wiring>,
}

struct Placed<'a> {
    node: &'a PatchNode,
    top_left: Pos2,
    entry: Option<&'static ModuleEntry>,
}

/// One drawn port nub.
#[derive(Clone, Copy, Debug)]
struct Nub {
    end: PortEnd,
    center: Pos2,
    side: Side,
    name: &'static str,
}

/// Edits gathered while drawing, applied once the patch is free to change.
#[derive(Debug, Default)]
struct Actions {
    toggle_power: Option<u64>,
    select: Option<Option<u64>>,
    drag_start: Option<TileDrag>,
    drag_end: Option<(u64, Pos2)>,
    connect: Option<(PortEnd, PortEnd)>,
    remove_cable: Option<usize>,
    delete: Option<u64>,
    browse: Option<(Pos2, (i32, i32))>,
}

impl Canvas {
    /// Draws the patch into the whole of `ui` and handles interaction.
    pub fn show(&mut self, ui: &mut Ui, patch: &mut Patch, controller: &mut Controller) -> Outcome {
        let rect = ui.max_rect();
        let background = ui.interact(rect, ui.scope_id().with("canvas"), Sense::click_and_drag());
        if background.dragged() && self.wiring.is_none() {
            self.pan += background.drag_delta();
        }
        let origin = rect.min + self.pan;
        let painter = ui.painter_at(rect);
        paint_grid(&painter, rect, origin);

        let pointer = ui.input(|input| input.pointer.interact_pos());
        let placed: Vec<Placed<'_>> = patch
            .nodes
            .iter()
            .map(|node| {
                let mut top_left = cell_origin(origin, node.cell);
                if let (Some(drag), Some(pointer)) = (self.drag, pointer)
                    && drag.uid == node.uid
                {
                    top_left = pointer - drag.grab;
                }
                Placed {
                    node,
                    top_left,
                    entry: box_modules::find(&node.kind),
                }
            })
            .collect();

        let mut actions = Actions::default();
        // The dragged tile draws last so it floats above the others.
        let mut order: Vec<usize> = (0..placed.len()).collect();
        order.sort_by_key(|&index| {
            self.drag
                .is_some_and(|drag| drag.uid == placed[index].node.uid)
        });
        for &index in &order {
            let tile = &placed[index];
            let shared = tile.entry.and_then(|_| controller.shared(tile.node));
            let selected = self.selected == Some(tile.node.uid);
            let responses = draw_tile(ui, tile, shared, selected);
            if responses.power.clicked() {
                actions.toggle_power = Some(tile.node.uid);
            }
            if responses.header.clicked() || responses.header.drag_started() {
                actions.select = Some(Some(tile.node.uid));
            }
            if responses.header.drag_started()
                && let Some(pointer) = pointer
            {
                actions.drag_start = Some(TileDrag {
                    uid: tile.node.uid,
                    grab: pointer - tile.top_left,
                });
            }
            if responses.header.drag_stopped() {
                actions.drag_end = Some((tile.node.uid, tile.top_left));
            }
        }

        // Wires, with the one under the pointer lit when clicking would
        // remove it (empty canvas only, so knobs keep their clicks).
        let lines = wire_lines(patch, &placed);
        let hovered_wire = pointer
            .filter(|_| background.hovered() && self.wiring.is_none())
            .and_then(|pointer| {
                lines
                    .iter()
                    .map(|(index, line)| (*index, edit::distance_to_line(line, pointer)))
                    .filter(|(_, distance)| *distance <= WIRE_HIT)
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(index, _)| index)
            });
        for (index, line) in &lines {
            let lit = hovered_wire == Some(*index);
            paint_wire(&painter, line, lit);
        }
        if background.clicked() {
            match hovered_wire {
                Some(index) => actions.remove_cable = Some(index),
                None => actions.select = Some(None),
            }
        }
        if background.secondary_clicked()
            && let Some(pointer) = pointer
        {
            actions.browse = Some((pointer, cell_at(origin, pointer)));
        }

        let nubs = nubs(&placed);
        self.handle_nubs(ui, patch, &nubs, pointer, &mut actions);
        if let Some(wiring) = self.wiring
            && let Some(start) = nubs.iter().find(|nub| nub.end == wiring.from)
        {
            let live = flatten(wire(
                start.center,
                start.side,
                start.center + wiring.reach,
                opposite(start.side),
            ));
            paint_wire(&painter, &live, true);
        }

        if let Some(uid) = self.selected
            && ui.input(|input| input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace))
            && !ui.ctx().egui_wants_keyboard_input()
        {
            actions.delete = Some(uid);
        }
        drop(placed);
        self.apply(actions, patch, origin)
    }

    /// Nub interaction: drag from a nub to start a wire, drop on another
    /// nub to connect. Draws the nubs, lighting valid targets while wiring.
    fn handle_nubs(
        &mut self,
        ui: &Ui,
        patch: &Patch,
        nubs: &[Nub],
        pointer: Option<Pos2>,
        actions: &mut Actions,
    ) {
        let painter = ui.painter();
        for nub in nubs {
            let id = ui
                .scope_id()
                .with(("nub", nub.end.uid, nub.end.port, nub.end.input));
            let response = ui
                .interact(
                    Rect::from_center_size(nub.center, Vec2::splat(2.0 * NUB_HIT)),
                    id,
                    Sense::click_and_drag(),
                )
                .on_hover_text(nub.name);
            if response.drag_started() {
                self.wiring = Some(Wiring {
                    from: nub.end,
                    reach: Vec2::ZERO,
                });
            }
            if response.dragged()
                && let (Some(wiring), Some(pointer)) = (self.wiring.as_mut(), pointer)
            {
                wiring.reach = pointer - nub.center;
            }
            if response.drag_stopped()
                && let Some(wiring) = self.wiring.take()
            {
                let target = pointer.and_then(|pointer| nub_at(nubs, pointer));
                if let Some(target) = target
                    && target.end != wiring.from
                {
                    actions.connect = Some((wiring.from, target.end));
                }
            }

            let connected = patch.cables.iter().any(|cable| {
                let end = if nub.end.input {
                    &cable.to
                } else {
                    &cable.from
                };
                edit::touches(end, nub.end.uid, nub.end.port)
            });
            let target = self
                .wiring
                .is_some_and(|wiring| wiring.from.input != nub.end.input);
            painter.circle_filled(nub.center, NUB, palette::INSET);
            let ring = if target {
                palette::HIGHLIGHT
            } else if connected || response.hovered() {
                palette::ACCENT
            } else {
                palette::STROKE_BRIGHT
            };
            painter.circle_stroke(nub.center, NUB, Stroke::new(1.5, ring));
        }
    }

    /// Applies the frame's gathered edits to the patch.
    fn apply(&mut self, actions: Actions, patch: &mut Patch, origin: Pos2) -> Outcome {
        let mut outcome = Outcome {
            browse: actions.browse,
            ..Outcome::default()
        };
        if let Some(select) = actions.select {
            self.selected = select;
        }
        if let Some(drag) = actions.drag_start {
            self.drag = Some(drag);
        }
        if let Some((uid, top_left)) = actions.drag_end {
            self.drag = None;
            let target = nearest_cell(origin, top_left);
            if overlaps(patch, uid, target) {
                outcome.hint = Some("tiles can't overlap: moved back");
            } else if let Some(node) = patch.node_mut(uid)
                && node.cell != target
            {
                node.cell = target;
                outcome.moved = true;
            }
        }
        if let Some(uid) = actions.toggle_power
            && let Some(node) = patch.node_mut(uid)
        {
            node.powered = !node.powered;
            outcome.repatch = true;
        }
        if let Some((a, b)) = actions.connect {
            match edit::connect(patch, a, b) {
                Ok(added) => outcome.repatch |= added,
                Err(error) => outcome.hint = Some(error.hint()),
            }
        }
        if let Some(index) = actions.remove_cable
            && index < patch.cables.len()
        {
            patch.cables.remove(index);
            outcome.repatch = true;
        }
        if let Some(uid) = actions.delete
            && edit::delete_node(patch, uid)
        {
            self.selected = None;
            outcome.repatch = true;
        }
        outcome
    }
}

/// Faint dots on every cell corner.
fn paint_grid(painter: &Painter, rect: Rect, origin: Pos2) {
    painter.rect_filled(rect, 0.0, palette::BG);
    let first_col = ((rect.left() - origin.x) / CELL).floor() as i32;
    let first_row = ((rect.top() - origin.y) / CELL).floor() as i32;
    let cols = (rect.width() / CELL).ceil() as i32 + 1;
    let rows = (rect.height() / CELL).ceil() as i32 + 1;
    for col in first_col..=first_col + cols {
        for row in first_row..=first_row + rows {
            let point = cell_origin(origin, (col, row));
            if rect.contains(point) {
                painter.circle_filled(point, 1.0, palette::STROKE_BRIGHT);
            }
        }
    }
}

struct TileResponses {
    header: Response,
    power: Response,
}

fn draw_tile(
    ui: &mut Ui,
    tile: &Placed<'_>,
    shared: Option<Arc<Shared>>,
    selected: bool,
) -> TileResponses {
    let size = node_size(&tile.node.kind);
    let rect = tile_rect(tile.top_left, size);
    let header = to_rect(tile.top_left, geometry::header_area(size));
    let body = to_rect(tile.top_left, geometry::body_area(size));
    let scope = ui.scope_id().with(("tile", tile.node.uid));
    let painter = ui.painter().clone();

    surface::panel(&painter, rect);
    painter.rect_filled(
        header.shrink(1.0),
        egui::CornerRadius {
            nw: palette::RADIUS as u8,
            ne: palette::RADIUS as u8,
            sw: 0,
            se: 0,
        },
        palette::SURFACE,
    );

    let header_response = ui.interact(header, scope.with("header"), Sense::click_and_drag());
    let lamp_center = pos2(header.left() + 9.0, header.center().y);
    let power = ui.interact(
        Rect::from_center_size(lamp_center, vec2(LAMP_HIT, LAMP_HIT)),
        scope.with("power"),
        Sense::click(),
    );
    if tile.node.powered {
        glow::glow_circle(&painter, lamp_center, LAMP, palette::ACCENT, 1);
        painter.circle_filled(lamp_center, LAMP, palette::ACCENT);
    } else {
        painter.circle_stroke(lamp_center, LAMP, Stroke::new(1.0, palette::TEXT_MUTED));
    }
    let name = tile.entry.map_or("?", |entry| entry.descriptor.name);
    let label = text::tracked(
        &painter,
        pos2(header.left() + 18.0, header.center().y),
        Align2::LEFT_CENTER,
        name,
        fonts::heading(10.0),
        if tile.node.powered {
            palette::TEXT_SECONDARY
        } else {
            palette::TEXT_MUTED
        },
        1.0,
    );
    let header_extra = Rect::from_min_max(
        pos2(label.right() + 6.0, header.top()),
        pos2(header.right() - 4.0, header.bottom()),
    );

    match (tile.entry, shared) {
        (Some(entry), Some(shared)) => {
            let mut child =
                ui.new_child(UiBuilder::new().scope_id(scope.with("body")).max_rect(rect));
            (entry.view)(
                &mut child,
                &Tile {
                    body,
                    header_extra,
                    shared: &shared,
                },
            );
        }
        _ => {
            painter.text(
                body.center(),
                Align2::CENTER_CENTER,
                &tile.node.kind,
                FontId::proportional(9.0),
                palette::TEXT_MUTED,
            );
        }
    }
    if !tile.node.powered {
        painter.rect_filled(body, 0.0, Color32::from_black_alpha(110));
    }
    if selected {
        glow::glow_rect(&painter, rect, palette::RADIUS, palette::ACCENT, 2);
        painter.rect_stroke(
            rect,
            palette::RADIUS,
            Stroke::new(1.5, palette::ACCENT),
            StrokeKind::Inside,
        );
    }
    TileResponses {
        header: header_response,
        power,
    }
}

fn find<'a>(placed: &'a [Placed<'a>], uid: u64) -> Option<&'a Placed<'a>> {
    placed.iter().find(|tile| tile.node.uid == uid)
}

/// Every drawable wire as (cable index, polyline).
fn wire_lines(patch: &Patch, placed: &[Placed<'_>]) -> Vec<(usize, Vec<Pos2>)> {
    patch
        .cables
        .iter()
        .enumerate()
        .filter_map(|(index, cable)| {
            let from = find(placed, cable.from.uid)?;
            let to = find(placed, cable.to.uid)?;
            let from_descriptor = from.entry?.descriptor;
            let to_descriptor = to.entry?.descriptor;
            let from_port =
                &from_descriptor.outputs[from_descriptor.output_index(&cable.from.port)?];
            let to_port = &to_descriptor.inputs[to_descriptor.input_index(&cable.to.port)?];
            let points = wire(
                anchor(from.top_left, from_descriptor, from_port),
                from_port.side,
                anchor(to.top_left, to_descriptor, to_port),
                to_port.side,
            );
            Some((index, flatten(points)))
        })
        .collect()
}

fn paint_wire(painter: &Painter, line: &[Pos2], lit: bool) {
    let (color, width) = if lit {
        (palette::HIGHLIGHT, 2.5)
    } else {
        (palette::ACCENT, 2.0)
    };
    glow::glow_line(painter, line, color, 1);
    painter.add(egui::Shape::line(line.to_vec(), Stroke::new(width, color)));
}

/// Every visible port of every known tile.
fn nubs(placed: &[Placed<'_>]) -> Vec<Nub> {
    let mut nubs = Vec::new();
    for tile in placed {
        let Some(entry) = tile.entry else {
            continue;
        };
        let descriptor = entry.descriptor;
        let inputs = descriptor.inputs.iter().map(|port| (port, true));
        let outputs = descriptor
            .visible_ports()
            .skip(descriptor.inputs.len())
            .map(|port| (port, false));
        for (port, input) in inputs.chain(outputs) {
            nubs.push(Nub {
                end: PortEnd {
                    uid: tile.node.uid,
                    port: port.id,
                    input,
                },
                center: anchor(tile.top_left, descriptor, port),
                side: port.side,
                name: port.name,
            });
        }
    }
    nubs
}

/// The nub nearest `pointer` within reach.
fn nub_at(nubs: &[Nub], pointer: Pos2) -> Option<&Nub> {
    nubs.iter()
        .map(|nub| (nub, (nub.center - pointer).length()))
        .filter(|(_, distance)| *distance <= NUB_HIT)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(nub, _)| nub)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::patch_io::default_patch;
    use box_core::geometry::HEADER;
    use box_modules::{env, gates, output, sine};

    const ORIGIN: Pos2 = pos2(10.0, 20.0);

    fn placed(patch: &Patch, origin: Pos2) -> Vec<Placed<'_>> {
        patch
            .nodes
            .iter()
            .map(|node| Placed {
                node,
                top_left: cell_origin(origin, node.cell),
                entry: box_modules::find(&node.kind),
            })
            .collect()
    }

    #[test]
    fn cells_and_pixels_round_trip() {
        assert_eq!(cell_origin(ORIGIN, (2, 1)), pos2(202.0, 116.0));
        assert_eq!(nearest_cell(ORIGIN, pos2(202.0, 116.0)), (2, 1));
        // A drop within half a cell snaps to the nearer cell.
        assert_eq!(
            nearest_cell(ORIGIN, pos2(202.0 + 47.0, 116.0 - 47.0)),
            (2, 1)
        );
        assert_eq!(nearest_cell(ORIGIN, pos2(202.0 + 49.0, 116.0)), (3, 1));
        assert_eq!(nearest_cell(ORIGIN, pos2(-100.0, 20.0)), (-1, 0));
    }

    #[test]
    fn tile_frames_follow_their_size() {
        let rect = tile_rect(ORIGIN, env::DESCRIPTOR.size);
        assert_eq!(rect.min, pos2(12.0, 22.0));
        assert_eq!(rect.size(), vec2(188.0, 92.0));
        let header = to_rect(ORIGIN, geometry::header_area((1, 1)));
        assert_eq!(header.height(), HEADER);
    }

    #[test]
    fn nubs_sit_on_the_tile_frame() {
        for descriptor in [
            &sine::DESCRIPTOR,
            &gates::DESCRIPTOR,
            &env::DESCRIPTOR,
            &output::DESCRIPTOR,
        ] {
            let rect = tile_rect(ORIGIN, descriptor.size);
            for port in descriptor.visible_ports() {
                let point = anchor(ORIGIN, descriptor, port);
                let on_frame = point.x == rect.left() || point.x == rect.right();
                assert!(
                    on_frame && rect.y_range().contains(point.y),
                    "{}.{}",
                    descriptor.kind,
                    port.id
                );
            }
        }
        // Env's two inputs split the body height.
        let gate = anchor(ORIGIN, &env::DESCRIPTOR, &env::DESCRIPTOR.inputs[1]);
        assert_eq!(gate.y, ORIGIN.y + 2.0 + HEADER + 74.0 * 0.75);
    }

    #[test]
    fn ports_are_hit_under_pan() {
        let patch = default_patch();
        let panned = ORIGIN + vec2(-130.0, 55.0);
        let tiles = placed(&patch, panned);
        let all = nubs(&tiles);
        // Sine's output sits on its right edge, mid-body.
        let sine_out = all
            .iter()
            .find(|nub| nub.end.uid == 1 && !nub.end.input)
            .unwrap();
        let hit = nub_at(&all, sine_out.center + vec2(3.0, -2.0)).unwrap();
        assert_eq!(hit.end, sine_out.end);
        assert!(nub_at(&all, sine_out.center + vec2(30.0, 0.0)).is_none());
        // Output's hidden L/R outputs are never nubs.
        assert!(!all.iter().any(|nub| nub.end.uid == 4 && !nub.end.input));
    }

    #[test]
    fn wires_are_hit_near_their_line() {
        let patch = default_patch();
        let tiles = placed(&patch, ORIGIN);
        let lines = wire_lines(&patch, &tiles);
        assert_eq!(lines.len(), 3);
        let (index, line) = &lines[0];
        let middle = line[line.len() / 2];
        assert!(edit::distance_to_line(line, middle + vec2(0.0, 4.0)) <= WIRE_HIT);
        assert_eq!(*index, 0);
        assert!(edit::distance_to_line(line, middle + vec2(0.0, 40.0)) > WIRE_HIT);
    }

    #[test]
    fn overlapping_drops_are_refused() {
        let patch = default_patch();
        // Env (2x1) sits at (3, 1); Sine is uid 1 at (1, 1).
        assert!(overlaps(&patch, 1, (4, 1)));
        assert!(overlaps(&patch, 1, (3, 1)));
        assert!(!overlaps(&patch, 1, (5, 1)));
        assert!(!overlaps(&patch, 1, (1, 1)), "its own cell is free");
        assert!(!overlaps(&patch, 1, (3, 0)));
        assert!(cells_overlap((0, 0), (2, 2), (1, 1), (1, 1)));
        assert!(!cells_overlap((0, 0), (2, 1), (0, 1), (1, 1)));
    }

    #[test]
    fn wires_leave_along_their_sides() {
        let points = wire(pos2(0.0, 0.0), Side::Right, pos2(200.0, 0.0), Side::Left);
        assert!(points[1].x > 0.0 && points[1].y == 0.0);
        assert!(points[2].x < 200.0);
    }

    #[test]
    fn unknown_kinds_are_one_cell_placeholders() {
        assert_eq!(node_size("box.nope"), (1, 1));
        assert_eq!(node_size(env::KIND), (2, 1));
    }

    /// Renders `patch` offscreen to `target/snapshots/<name>.png`, after
    /// `setup` adjusts the canvas state.
    pub(crate) fn snapshot(name: &str, patch: Patch, setup: impl FnOnce(&mut Canvas)) {
        snapshot_with(name, patch, setup, |_| {});
    }

    /// [`snapshot`] with the module browser drawn over the canvas.
    pub(crate) fn snapshot_with(
        name: &str,
        mut patch: Patch,
        setup: impl FnOnce(&mut Canvas),
        browser_setup: impl FnOnce(&mut crate::browser::Browser),
    ) {
        let (mut controller, _engine) = box_dsp::new_engine(box_modules::registry, 48_000.0, 512);
        controller.apply(&patch).unwrap();
        let mut canvas = Canvas::default();
        setup(&mut canvas);
        let mut browser = crate::browser::Browser::default();
        browser_setup(&mut browser);
        // At least 8x4 cells, grown to fit every tile plus a margin cell.
        let (cols, rows) = patch.nodes.iter().fold((8, 4), |(cols, rows), node| {
            let size = node_size(&node.kind);
            (
                cols.max(node.cell.0 + i32::from(size.0) + 1),
                rows.max(node.cell.1 + i32::from(size.1) + 1),
            )
        });
        let mut harness = egui_kittest::Harness::builder()
            .with_size(vec2(cols as f32 * CELL, rows as f32 * CELL))
            .with_pixels_per_point(2.0)
            .with_theme(egui::Theme::Dark)
            .with_accessibility_check(false)
            .allow_missing_glyphs()
            .with_render_cursor(false)
            .with_context_setup(|ctx| {
                ctx.set_visuals(neon_ui::visuals::dark_neon_visuals());
                neon_ui::fonts::install(ctx);
            })
            .build_ui(move |ui| {
                canvas.show(ui, &mut patch, &mut controller);
                browser.show(&ui.ctx().clone(), &mut patch);
            });
        harness.run_steps(3);
        let image = harness.render().expect("offscreen render");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        let mut flat = image::RgbaImage::from_pixel(
            image.width(),
            image.height(),
            image::Rgba([palette::BG.r(), palette::BG.g(), palette::BG.b(), 255]),
        );
        image::imageops::overlay(&mut flat, &image, 0, 0);
        flat.save(dir.join(format!("{name}.png"))).unwrap();
    }

    /// Offscreen renders for review; they need a GPU adapter, so they run
    /// only on request: `cargo test -p box-app -- --ignored snapshot`.
    #[test]
    #[ignore = "renders a PNG for review; needs a GPU adapter"]
    fn snapshot_default_patch() {
        snapshot("box-canvas", default_patch(), |_| {});
    }

    #[test]
    #[ignore = "renders a PNG for review; needs a GPU adapter"]
    fn snapshot_demo_patch() {
        snapshot("box-demo", crate::patch_io::demo_patch(), |_| {});
    }

    #[test]
    #[ignore = "renders a PNG for review; needs a GPU adapter"]
    fn snapshot_live_wire_and_selection() {
        let mut patch = default_patch();
        patch.cables.remove(2);
        snapshot("box-canvas-wiring", patch, |canvas| {
            // Env's output dragged down-right, toward the Output.
            canvas.wiring = Some(Wiring {
                from: PortEnd {
                    uid: 3,
                    port: "out",
                    input: false,
                },
                reach: vec2(60.0, 50.0),
            });
            canvas.selected = Some(1);
        });
    }

    /// Every registered module once, packed row by row, for a layout review.
    #[test]
    #[ignore = "renders a PNG for review; needs a GPU adapter"]
    fn snapshot_module_gallery() {
        let mut patch = Patch::default();
        let (mut col, mut row, mut row_height) = (0_i32, 0_i32, 1_i32);
        for (index, entry) in box_modules::MODULES.iter().enumerate() {
            let (width, height) = entry.descriptor.size;
            if col + i32::from(width) > 8 {
                col = 0;
                row += row_height;
                row_height = 1;
            }
            patch.nodes.push(PatchNode::new(
                index as u64 + 1,
                entry.descriptor.kind,
                (col, row),
            ));
            col += i32::from(width);
            row_height = row_height.max(i32::from(height));
        }
        snapshot("box-gallery", patch, |_| {});
    }
}
