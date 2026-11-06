//! box-modules — the module library. Each module lives in its own folder:
//! `mod.rs` (kind, ports, params, descriptor and registry entry), `dsp.rs`
//! (the `Module` impl, egui-free) and `view.rs` (the tile view). Adding a
//! module is a new folder plus one line in [`MODULES`]. See `specs/box.md`.

pub mod clock;
pub mod env;
pub mod filter;
pub mod format;
pub mod gates;
pub mod mix4;
pub mod output;
pub mod scaler;
pub mod seq;
pub mod sine;
pub mod tile;

use box_core::ModuleDescriptor;

pub use tile::{LayoutFn, Tile, ViewFn};

/// A registered module: its descriptor and its tile view.
#[derive(Clone, Copy, Debug)]
pub struct ModuleEntry {
    pub descriptor: &'static ModuleDescriptor,
    /// Draws the tile body (and, optionally, the free part of the header).
    pub view: ViewFn,
    /// Widget rects the view places inside a body rect; tests check they fit.
    pub layout: LayoutFn,
}

/// Every module box knows, in menu order.
pub static MODULES: &[ModuleEntry] = &[
    sine::ENTRY,
    gates::ENTRY,
    clock::ENTRY,
    seq::ENTRY,
    scaler::ENTRY,
    env::ENTRY,
    filter::ENTRY,
    mix4::ENTRY,
    output::ENTRY,
];

/// The entry for a module kind.
#[must_use]
pub fn find(kind: &str) -> Option<&'static ModuleEntry> {
    MODULES.iter().find(|entry| entry.descriptor.kind == kind)
}

/// Descriptor lookup with the signature the engine's compiler takes.
#[must_use]
pub fn registry(kind: &str) -> Option<&'static ModuleDescriptor> {
    find(kind).map(|entry| entry.descriptor)
}

#[cfg(test)]
mod tests {
    use box_core::geometry::{self, body_area};
    use box_core::{BlockIo, Module, ParamValues, Shared};
    use egui::{Rect, pos2, vec2};

    use super::*;

    #[test]
    fn registered_kinds_are_unique_and_valid() {
        for (index, entry) in MODULES.iter().enumerate() {
            let descriptor = entry.descriptor;
            assert_eq!(descriptor.validate(), Ok(()), "{}", descriptor.kind);
            assert!(
                MODULES[..index]
                    .iter()
                    .all(|other| other.descriptor.kind != descriptor.kind),
                "duplicate kind {}",
                descriptor.kind
            );
            assert_eq!(
                registry(descriptor.kind).map(|d| d.kind),
                Some(descriptor.kind)
            );
        }
    }

    #[test]
    fn every_port_sits_on_its_tile() {
        for entry in MODULES {
            let descriptor = entry.descriptor;
            let tile = geometry::tile_area(descriptor.size);
            for port in descriptor.visible_ports() {
                let (x, y) = geometry::port_anchor(descriptor, port);
                let on_edge =
                    x == tile.x || x == tile.x + tile.w || y == tile.y || y == tile.y + tile.h;
                assert!(
                    tile.contains(x, y) && on_edge,
                    "{}.{}",
                    descriptor.kind,
                    port.id
                );
            }
        }
    }

    #[test]
    fn each_view_fits_its_tile() {
        for entry in MODULES {
            let area = body_area(entry.descriptor.size);
            let body = Rect::from_min_size(pos2(area.x, area.y), vec2(area.w, area.h));
            let rects = (entry.layout)(body);
            assert!(!rects.is_empty(), "{}", entry.descriptor.kind);
            for rect in &rects {
                assert!(
                    body.expand(0.01).contains_rect(*rect),
                    "{}: {rect:?} outside {body:?}",
                    entry.descriptor.kind
                );
            }
            for (index, rect) in rects.iter().enumerate() {
                for other in &rects[..index] {
                    assert!(
                        !rect.shrink(0.01).intersects(other.shrink(0.01)),
                        "{}",
                        entry.descriptor.kind
                    );
                }
            }
        }
    }

    /// Runs one block of `module` with `inputs`; returns its outputs.
    pub(crate) fn run_block(
        module: &mut dyn Module,
        shared: &Shared,
        inputs: &[&[f32]],
        connected: u32,
        outputs: usize,
        len: usize,
    ) -> Vec<Vec<f32>> {
        let mut values = vec![0.0; shared.params.len()];
        shared.params.snapshot_into(&mut values);
        let mut owned = vec![vec![0.0; len]; outputs];
        let mut slices: Vec<&mut [f32]> = owned.iter_mut().map(Vec::as_mut_slice).collect();
        let mut io = BlockIo {
            inputs,
            outputs: &mut slices,
            len,
            connected,
        };
        module.process(&mut io, &ParamValues(&values), &shared.meters);
        owned
    }

    /// A fresh instance and shared state of `entry` at 48 kHz.
    pub(crate) fn instance(entry: &ModuleEntry) -> (Box<dyn Module>, Shared) {
        let descriptor = entry.descriptor;
        ((descriptor.create)(48_000.0, 4096), Shared::new(descriptor))
    }

    #[cfg(feature = "rt-audit")]
    #[test]
    fn the_basic_patch_renders_without_allocating() {
        use box_dsp::{Cable, Patch, PatchNode, new_engine};
        let patch = Patch {
            nodes: vec![
                PatchNode::new(1, gates::KIND, (0, 0)),
                PatchNode::new(2, sine::KIND, (0, 1)),
                PatchNode::new(3, env::KIND, (1, 0)),
                PatchNode::new(4, output::KIND, (3, 0)),
            ],
            cables: vec![
                Cable::new(1, "gate", 3, "gate"),
                Cable::new(2, "out", 3, "in"),
                Cable::new(3, "out", 4, "l"),
            ],
        };
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 512);
        controller.apply(&patch).unwrap();
        let mut left = vec![0.0; 512];
        let mut right = vec![0.0; 512];
        let mut peak = 0.0_f32;
        let (_, allocations) = musictools_core::rt_audit::audit(|| {
            for _ in 0..200 {
                engine.process(&mut left, &mut right);
                peak = left
                    .iter()
                    .fold(peak, |peak, sample| peak.max(sample.abs()));
            }
        });
        assert_eq!(allocations, 0);
        assert!(peak > 0.05 && peak <= 0.5 + 1e-6, "peak {peak}");
        assert_eq!(left, right);
    }

    /// The app's demo chain: every module from the 2026-11-01 round in one
    /// patch, rendered for ~2 s with zero allocations.
    #[cfg(feature = "rt-audit")]
    #[test]
    fn the_demo_chain_renders_without_allocating() {
        use box_dsp::{Cable, Patch, PatchNode, new_engine};
        let patch = Patch {
            nodes: vec![
                PatchNode::new(1, clock::KIND, (0, 0)),
                PatchNode::new(2, seq::KIND, (2, 0)),
                PatchNode::new(3, scaler::KIND, (7, 0)),
                PatchNode::new(4, sine::KIND, (10, 0)),
                PatchNode::new(5, env::KIND, (7, 2)),
                PatchNode::new(6, filter::KIND, (10, 2)),
                PatchNode::new(7, mix4::KIND, (7, 4)),
                PatchNode::new(8, output::KIND, (12, 4)),
            ],
            cables: vec![
                Cable::new(1, "clock", 2, "clock"),
                Cable::new(2, "pitch", 3, "pitch"),
                Cable::new(3, "pitch", 4, "pitch"),
                Cable::new(4, "out", 5, "in"),
                Cable::new(2, "gate", 5, "gate"),
                Cable::new(5, "out", 6, "in"),
                Cable::new(6, "out", 7, "1"),
                Cable::new(7, "l", 8, "l"),
                Cable::new(7, "r", 8, "r"),
            ],
        };
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 512);
        controller.apply(&patch).unwrap();
        let mut left = vec![0.0; 512];
        let mut right = vec![0.0; 512];
        let mut peak = 0.0_f32;
        let (_, allocations) = musictools_core::rt_audit::audit(|| {
            for _ in 0..200 {
                engine.process(&mut left, &mut right);
                peak = left
                    .iter()
                    .chain(&right)
                    .fold(peak, |peak, sample| peak.max(sample.abs()));
            }
        });
        assert_eq!(allocations, 0);
        assert!(peak > 0.01 && peak.is_finite(), "peak {peak}");
    }
}
