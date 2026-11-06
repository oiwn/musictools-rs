//! Ports: named mono signal connections on a tile edge.

/// Most ports one module may have on each of its input and output lists.
/// The engine sizes its per-step stack arrays with this.
pub const MAX_PORTS: usize = 8;

/// What a port carries. v1 has only audio-rate signals (a gate is a 0/1
/// audio signal); new kinds, such as modulation, can join later.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    /// One mono audio-rate `f32` stream.
    Audio,
}

/// Tile edge a port's nub sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// One port of a module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortSpec {
    /// Stable id used by saved patches; never rename it.
    pub id: &'static str,
    /// Short label for hints.
    pub name: &'static str,
    pub kind: PortKind,
    pub side: Side,
    /// Position along its edge, 0-based; the ports on one edge spread evenly.
    pub slot: u8,
}

impl PortSpec {
    /// An audio port.
    #[must_use]
    pub const fn audio(id: &'static str, name: &'static str, side: Side, slot: u8) -> Self {
        Self {
            id,
            name,
            kind: PortKind::Audio,
            side,
            slot,
        }
    }
}
