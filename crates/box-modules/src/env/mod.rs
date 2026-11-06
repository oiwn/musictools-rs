//! Env — an ADSR envelope that shapes its input (2x1).
//!
//! `out = in × envelope`, or the envelope itself when `in` is unconnected.
//! A rising gate edge (re)starts the attack from the current level; a
//! falling edge releases.

mod dsp;
mod view;

pub use dsp::Env;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;
use crate::format::{short_ms, short_percent};

/// Stable kind id.
pub const KIND: &str = "box.env";

const ATTACK: usize = 0;
const DECAY: usize = 1;
const SUSTAIN: usize = 2;
const RELEASE: usize = 3;

const IN: usize = 0;
const GATE: usize = 1;

/// Meter: the envelope level, for the dot beside the curve.
const LEVEL_METER: usize = 0;

static INPUTS: [PortSpec; 2] = [
    PortSpec::audio("in", "In", Side::Left, 0),
    PortSpec::audio("gate", "Gate", Side::Left, 1),
];
static OUTPUTS: [PortSpec; 1] = [PortSpec::audio("out", "Out", Side::Right, 0)];
static PARAMS: [ParamSpec; 4] = [
    ParamSpec::float("attack", "ATK", 0.5, 5000.0, 5.0, Taper::Log)
        .unit("ms")
        .formatter(short_ms),
    ParamSpec::float("decay", "DEC", 1.0, 5000.0, 200.0, Taper::Log)
        .unit("ms")
        .formatter(short_ms),
    ParamSpec::float("sustain", "SUS", 0.0, 1.0, 0.5, Taper::Linear).formatter(short_percent),
    ParamSpec::float("release", "REL", 1.0, 10_000.0, 300.0, Taper::Log)
        .unit("ms")
        .formatter(short_ms),
];

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "ENV",
    category: "Modulation",
    size: (2, 1),
    inputs: &INPUTS,
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &["level"],
    role: Role::Normal,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};
