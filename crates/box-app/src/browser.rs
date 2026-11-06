//! The module browser: right-click the canvas, type to search, Enter or
//! click to place the module at the nearest free cell (VCV Rack style).

use box_dsp::{Patch, PatchNode};
use box_modules::{MODULES, ModuleEntry};
use egui::{Align2, Color32, FontId, Key, Modifiers, Pos2, Sense, Stroke, TextEdit, vec2};
use neon_ui::palette;

use crate::canvas::{cells_overlap, node_size};

/// How far (in cells) placement looks for a free spot.
const SEARCH_RADIUS: i32 = 64;
/// Browser width in px.
const WIDTH: f32 = 230.0;

/// Modules matching `query`, best first. Matching is case-insensitive over
/// name, category and kind: a name prefix ranks first, then a name
/// substring, then a category or kind match. An empty query lists every
/// module, grouped by category in registry order.
#[must_use]
pub fn search(query: &str) -> Vec<&'static ModuleEntry> {
    let query = query.trim().to_lowercase();
    let mut ranked: Vec<(u8, usize, &'static ModuleEntry)> = MODULES
        .iter()
        .enumerate()
        .filter_map(|(order, entry)| {
            let descriptor = entry.descriptor;
            let name = descriptor.name.to_lowercase();
            let rank = if query.is_empty() || name.starts_with(&query) {
                0
            } else if name.contains(&query) {
                1
            } else if descriptor.category.to_lowercase().contains(&query)
                || descriptor.kind.to_lowercase().contains(&query)
            {
                2
            } else {
                return None;
            };
            Some((rank, order, entry))
        })
        .collect();
    if query.is_empty() {
        let categories: Vec<&str> = MODULES.iter().map(|e| e.descriptor.category).collect();
        let first_seen = |category: &str| categories.iter().position(|c| *c == category);
        ranked.sort_by_key(|(_, order, entry)| (first_seen(entry.descriptor.category), *order));
    } else {
        ranked.sort_by_key(|(rank, order, _)| (*rank, *order));
    }
    ranked.into_iter().map(|(_, _, entry)| entry).collect()
}

/// The free cell nearest `want` where a tile of `size` fits without
/// overlapping any node.
#[must_use]
pub fn nearest_free_cell(patch: &Patch, size: (u8, u8), want: (i32, i32)) -> (i32, i32) {
    let free = |cell: (i32, i32)| {
        !patch
            .nodes
            .iter()
            .any(|node| cells_overlap(cell, size, node.cell, node_size(&node.kind)))
    };
    for radius in 0..=SEARCH_RADIUS {
        let mut best: Option<((i32, i32), i32)> = None;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let cell = (want.0 + dx, want.1 + dy);
                let distance = dx * dx + dy * dy;
                if free(cell) && best.is_none_or(|(_, d)| distance < d) {
                    best = Some((cell, distance));
                }
            }
        }
        if let Some((cell, _)) = best {
            return cell;
        }
    }
    want
}

/// Adds a `kind` node at the free cell nearest `want`; returns its uid.
pub fn place(patch: &mut Patch, kind: &str, want: (i32, i32)) -> u64 {
    let cell = nearest_free_cell(patch, node_size(kind), want);
    let uid = patch.next_uid();
    patch.nodes.push(PatchNode::new(uid, kind, cell));
    uid
}

/// An open browser.
#[derive(Debug)]
struct Open {
    /// Screen position of the right-click.
    at: Pos2,
    /// The right-clicked cell.
    cell: (i32, i32),
    query: String,
    highlight: usize,
    /// Focus the search field on the first frame.
    fresh: bool,
}

/// The browser layer.
#[derive(Debug, Default)]
pub struct Browser {
    open: Option<Open>,
}

impl Browser {
    /// Opens at a screen position for a canvas cell.
    pub fn open(&mut self, at: Pos2, cell: (i32, i32)) {
        self.open = Some(Open {
            at,
            cell,
            query: String::new(),
            highlight: 0,
            fresh: true,
        });
    }

    /// Draws the browser if open; returns true when it placed a module.
    pub fn show(&mut self, ctx: &egui::Context, patch: &mut Patch) -> bool {
        let Some(open) = self.open.as_mut() else {
            return false;
        };
        let results = search(&open.query);
        // Keys the search field would otherwise eat.
        let (down, up, enter, escape) = ctx.input_mut(|input| {
            (
                input.consume_key(Modifiers::NONE, Key::ArrowDown),
                input.consume_key(Modifiers::NONE, Key::ArrowUp),
                input.consume_key(Modifiers::NONE, Key::Enter),
                input.consume_key(Modifiers::NONE, Key::Escape),
            )
        });
        if down {
            open.highlight = (open.highlight + 1).min(results.len().saturating_sub(1));
        }
        if up {
            open.highlight = open.highlight.saturating_sub(1);
        }
        let mut chosen = enter
            .then(|| results.get(open.highlight).copied())
            .flatten();
        let mut close = escape;

        let area = egui::Area::new(egui::Id::unique("box-module-browser"))
            .fixed_pos(open.at)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(palette::PANEL)
                    .stroke(Stroke::new(1.0, palette::STROKE_BRIGHT))
                    .corner_radius(palette::RADIUS)
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.set_width(WIDTH);
                        let field = ui.add(
                            TextEdit::singleline(&mut open.query)
                                .hint_text("search modules")
                                .desired_width(f32::INFINITY),
                        );
                        if open.fresh {
                            field.request_focus();
                            open.fresh = false;
                        }
                        if field.changed() {
                            open.highlight = 0;
                        }
                        ui.add_space(4.0);
                        if results.is_empty() {
                            ui.label(
                                egui::RichText::new("no modules match").color(palette::TEXT_MUTED),
                            );
                        }
                        let mut category = "";
                        for (index, entry) in results.iter().enumerate() {
                            let descriptor = entry.descriptor;
                            if descriptor.category != category {
                                category = descriptor.category;
                                ui.label(
                                    egui::RichText::new(category.to_uppercase())
                                        .size(9.0)
                                        .color(palette::TEXT_MUTED),
                                );
                            }
                            if row(ui, entry, index == open.highlight).clicked() {
                                chosen = Some(*entry);
                            }
                        }
                    });
            });
        if area.response.clicked_elsewhere() && chosen.is_none() {
            close = true;
        }

        let cell = open.cell;
        if let Some(entry) = chosen {
            place(patch, entry.descriptor.kind, cell);
            self.open = None;
            return true;
        }
        if close {
            self.open = None;
        }
        false
    }
}

/// One result row: name, then the tile size, lit when highlighted.
fn row(ui: &mut egui::Ui, entry: &ModuleEntry, highlighted: bool) -> egui::Response {
    let descriptor = entry.descriptor;
    let (rect, response) = ui.allocate_exact_size(vec2(WIDTH, 20.0), Sense::click());
    let painter = ui.painter();
    if highlighted || response.hovered() {
        painter.rect_filled(rect, palette::RADIUS_SMALL, palette::RAISED);
    }
    let name_color: Color32 = if highlighted {
        palette::ACCENT
    } else {
        palette::TEXT_PRIMARY
    };
    painter.text(
        rect.left_center() + vec2(6.0, 0.0),
        Align2::LEFT_CENTER,
        descriptor.name,
        FontId::proportional(12.0),
        name_color,
    );
    painter.text(
        rect.right_center() - vec2(6.0, 0.0),
        Align2::RIGHT_CENTER,
        format!("{}x{}", descriptor.size.0, descriptor.size.1),
        FontId::proportional(10.0),
        palette::TEXT_MUTED,
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch_io::default_patch;
    use box_modules::{env, gates, output, sine};

    fn kinds(results: &[&ModuleEntry]) -> Vec<&'static str> {
        results.iter().map(|entry| entry.descriptor.kind).collect()
    }

    #[test]
    fn empty_query_lists_everything_by_category() {
        let all = kinds(&search(""));
        assert_eq!(all.len(), MODULES.len());
        // Sources (Sine, Gates) stay together ahead of later categories.
        assert_eq!(&all[..2], [sine::KIND, gates::KIND]);
    }

    #[test]
    fn search_is_case_insensitive_and_ranked() {
        assert_eq!(kinds(&search("env")), [env::KIND]);
        assert_eq!(kinds(&search("ENV")), [env::KIND]);
        assert_eq!(kinds(&search("ut")), [output::KIND]);
        // "sources" only matches the category: every source, registry order.
        let sources = search("sources");
        assert!(sources.len() >= 2);
        assert!(sources.iter().all(|e| e.descriptor.category == "Sources"));
        assert_eq!(&kinds(&sources)[..2], [sine::KIND, gates::KIND]);
        // A name prefix beats a category or kind hit.
        let s = kinds(&search("s"));
        assert_eq!(s[0], sine::KIND);
        assert!(search("zzz").is_empty());
    }

    #[test]
    fn placement_finds_the_nearest_free_cell() {
        let patch = default_patch();
        // (1, 1) holds Sine: a 1x1 lands on a neighbor.
        let cell = nearest_free_cell(&patch, (1, 1), (1, 1));
        assert_eq!((cell.0 - 1).abs().max((cell.1 - 1).abs()), 1);
        // A free cell is used as is.
        assert_eq!(nearest_free_cell(&patch, (1, 1), (0, 5)), (0, 5));
        // Env occupies (3, 1)-(4, 1); a 2x1 wanted at (3, 1) can't sit on
        // row 1 between Env and Output either, so it moves off the row.
        let wide = nearest_free_cell(&patch, (2, 1), (3, 1));
        assert!(!cells_overlap(wide, (2, 1), (3, 1), (2, 1)));
        assert!(!cells_overlap(wide, (2, 1), (6, 1), (1, 1)));
    }

    #[test]
    fn placing_adds_exactly_one_node() {
        let mut patch = default_patch();
        let before = patch.nodes.len();
        let uid = place(&mut patch, env::KIND, (0, 0));
        assert_eq!(patch.nodes.len(), before + 1);
        let node = patch.node(uid).unwrap();
        assert_eq!(node.kind, env::KIND);
        assert_eq!(node.cell, (0, 0));
        assert_eq!(uid, 5);
    }

    #[test]
    #[ignore = "renders a PNG for review; needs a GPU adapter"]
    fn snapshot_open_browser() {
        crate::canvas::tests::snapshot_with(
            "box-browser",
            default_patch(),
            |_| {},
            |browser| {
                browser.open(egui::pos2(420.0, 140.0), (4, 1));
                if let Some(open) = browser.open.as_mut() {
                    open.query = "s".to_owned();
                    open.highlight = 1;
                }
            },
        );
    }
}
