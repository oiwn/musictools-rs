//! Module descriptors: the static facts a module registers.

use std::fmt;

use crate::{MAX_PORTS, Module, ParamSpec, PortSpec, Side};

/// How the engine treats a module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// An ordinary module: runs when a sink depends on it.
    Normal,
    /// A terminal node (Output): its two outputs, L and R, are the graph
    /// output, summed across sinks, and are never drawn as nubs.
    Sink,
}

/// Everything box knows about a module kind before creating one.
#[derive(Clone, Copy)]
pub struct ModuleDescriptor {
    /// Stable id used by saved patches; never rename it.
    pub kind: &'static str,
    /// Tile label.
    pub name: &'static str,
    /// Registry menu group.
    pub category: &'static str,
    /// Tile size in cells: (columns, rows).
    pub size: (u8, u8),
    pub inputs: &'static [PortSpec],
    pub outputs: &'static [PortSpec],
    pub params: &'static [ParamSpec],
    /// Names of the meter values the module publishes for its view.
    pub meters: &'static [&'static str],
    pub role: Role,
    /// Builds a fresh instance, already `reset` for this rate and block size.
    pub create: fn(sample_rate: f32, max_block: usize) -> Box<dyn Module>,
}

impl fmt::Debug for ModuleDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleDescriptor")
            .field("kind", &self.kind)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

/// Why a descriptor is malformed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DescriptorError {
    EmptySize,
    TooManyPorts,
    DuplicatePort(&'static str),
    DuplicateParam(&'static str),
    /// Two ports share a slot, or a slot leaves a gap on its edge.
    BadSlot(&'static str),
    /// A sink needs exactly two outputs (L, R).
    SinkOutputs,
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySize => write!(f, "tile size must be at least 1x1"),
            Self::TooManyPorts => write!(f, "more than {MAX_PORTS} inputs or outputs"),
            Self::DuplicatePort(id) => write!(f, "duplicate port id `{id}`"),
            Self::DuplicateParam(id) => write!(f, "duplicate param id `{id}`"),
            Self::BadSlot(id) => write!(f, "port `{id}` has a clashing or gapped slot"),
            Self::SinkOutputs => write!(f, "a sink needs exactly two outputs"),
        }
    }
}

impl std::error::Error for DescriptorError {}

impl ModuleDescriptor {
    /// Index of input `id`.
    #[must_use]
    pub fn input_index(&self, id: &str) -> Option<usize> {
        self.inputs.iter().position(|port| port.id == id)
    }

    /// Index of output `id`.
    #[must_use]
    pub fn output_index(&self, id: &str) -> Option<usize> {
        self.outputs.iter().position(|port| port.id == id)
    }

    /// The ports drawn on the tile: all inputs, plus outputs unless sink.
    pub fn visible_ports(&self) -> impl Iterator<Item = &'static PortSpec> {
        let outputs: &'static [PortSpec] = if self.role == Role::Sink {
            &[]
        } else {
            self.outputs
        };
        self.inputs.iter().chain(outputs.iter())
    }

    /// Number of drawn ports on `side`.
    #[must_use]
    pub fn ports_on(&self, side: Side) -> usize {
        self.visible_ports()
            .filter(|port| port.side == side)
            .count()
    }

    /// Checks the descriptor's invariants; registries test every entry.
    pub fn validate(&self) -> Result<(), DescriptorError> {
        if self.size.0 == 0 || self.size.1 == 0 {
            return Err(DescriptorError::EmptySize);
        }
        if self.inputs.len() > MAX_PORTS || self.outputs.len() > MAX_PORTS {
            return Err(DescriptorError::TooManyPorts);
        }
        if self.role == Role::Sink && self.outputs.len() != 2 {
            return Err(DescriptorError::SinkOutputs);
        }
        for list in [self.inputs, self.outputs] {
            for (index, port) in list.iter().enumerate() {
                if list[..index].iter().any(|other| other.id == port.id) {
                    return Err(DescriptorError::DuplicatePort(port.id));
                }
            }
        }
        for (index, param) in self.params.iter().enumerate() {
            if self.params[..index]
                .iter()
                .any(|other| other.id == param.id)
            {
                return Err(DescriptorError::DuplicateParam(param.id));
            }
        }
        for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
            let count = self.ports_on(side);
            let ports: Vec<&PortSpec> = self
                .visible_ports()
                .filter(|port| port.side == side)
                .collect();
            for (index, port) in ports.iter().enumerate() {
                let clash = ports[..index].iter().any(|other| other.slot == port.slot);
                if clash || usize::from(port.slot) >= count {
                    return Err(DescriptorError::BadSlot(port.id));
                }
            }
        }
        Ok(())
    }
}
