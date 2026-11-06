//! Gates — a free-running 0/1 gate generator (1x1).

mod dsp;
mod view;

pub use dsp::Gates;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;
use crate::format::{short_hz, short_percent};

/// Stable kind id.
pub const KIND: &str = "box.gates";

const RATE: usize = 0;
const WIDTH: usize = 1;

/// Meter: the gate's latest value, for the header lamp.
const GATE_METER: usize = 0;

static OUTPUTS: [PortSpec; 1] = [PortSpec::audio("gate", "Gate", Side::Right, 0)];
static PARAMS: [ParamSpec; 2] = [
    ParamSpec::float("rate", "RATE", 0.1, 20.0, 2.0, Taper::Log)
        .unit("Hz")
        .formatter(short_hz),
    ParamSpec::float("width", "WIDTH", 0.01, 0.99, 0.5, Taper::Linear).formatter(short_percent),
];

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "GATES",
    category: "Sources",
    size: (1, 1),
    inputs: &[],
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &["gate"],
    role: Role::Normal,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};
