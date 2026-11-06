//! Output — the stereo sink that feeds the device or host (1x1).
//!
//! Inputs are hard-limited to ±1 before the level, so the output never
//! exceeds the Level setting. R follows L while R is unconnected.

mod dsp;
mod view;

pub use dsp::Output;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;
use crate::format::short_db;

/// Stable kind id.
pub const KIND: &str = "box.output";

const LEVEL: usize = 0;
/// Meter: held peak of both channels after the level.
const PEAK: usize = 0;

static INPUTS: [PortSpec; 2] = [
    PortSpec::audio("l", "L", Side::Left, 0),
    PortSpec::audio("r", "R", Side::Left, 1),
];
static OUTPUTS: [PortSpec; 2] = [
    PortSpec::audio("l", "L", Side::Right, 0),
    PortSpec::audio("r", "R", Side::Right, 1),
];
static PARAMS: [ParamSpec; 1] =
    [
        ParamSpec::float("level", "LEVEL", 0.0, 1.0, 0.5, Taper::Skew(2.0))
            .formatter(short_db)
            .smoothing(20.0),
    ];

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "OUT",
    category: "Output",
    size: (1, 1),
    inputs: &INPUTS,
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &["peak"],
    role: Role::Sink,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};
