//! Cell-grid geometry in logical pixels, without any UI framework.
//!
//! Coordinates are relative to the top-left corner of a module's first cell.

use crate::{ModuleDescriptor, PortSpec, Side};

/// Edge of one grid cell.
pub const CELL: f32 = 96.0;
/// Gap between a tile and its cell boundary, per side.
pub const TILE_INSET: f32 = 2.0;
/// Height of the box-drawn header strip (power switch and label).
pub const HEADER: f32 = 18.0;

/// An axis-aligned rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Area {
    /// True when `(x, y)` lies inside or on the edge.
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

/// The tile for a size in cells: its cells inset by [`TILE_INSET`].
#[must_use]
pub fn tile_area(size: (u8, u8)) -> Area {
    Area {
        x: TILE_INSET,
        y: TILE_INSET,
        w: f32::from(size.0) * CELL - 2.0 * TILE_INSET,
        h: f32::from(size.1) * CELL - 2.0 * TILE_INSET,
    }
}

/// The header strip box draws at the top of a tile.
#[must_use]
pub fn header_area(size: (u8, u8)) -> Area {
    let tile = tile_area(size);
    Area { h: HEADER, ..tile }
}

/// The part of a tile the module's view fills.
#[must_use]
pub fn body_area(size: (u8, u8)) -> Area {
    let tile = tile_area(size);
    Area {
        y: tile.y + HEADER,
        h: tile.h - HEADER,
        ..tile
    }
}

/// Where a port's nub sits: on the tile edge for its side, spread evenly
/// by slot (across the body height for left and right ports).
#[must_use]
pub fn port_anchor(descriptor: &ModuleDescriptor, port: &PortSpec) -> (f32, f32) {
    let tile = tile_area(descriptor.size);
    let body = body_area(descriptor.size);
    let count = descriptor.ports_on(port.side).max(1) as f32;
    let fraction = (f32::from(port.slot) + 0.5) / count;
    match port.side {
        Side::Left => (tile.x, body.y + fraction * body.h),
        Side::Right => (tile.x + tile.w, body.y + fraction * body.h),
        Side::Top => (tile.x + fraction * tile.w, tile.y),
        Side::Bottom => (tile.x + fraction * tile.w, tile.y + tile.h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_cell_tile_leaves_a_92_by_74_body() {
        let body = body_area((1, 1));
        assert_eq!((body.w, body.h), (92.0, 74.0));
        assert_eq!(tile_area((2, 1)).w, 188.0);
    }
}
