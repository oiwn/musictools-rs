//! Sine — a phase-accumulator sine oscillator (1x1).
//!
//! Frequency is `FREQ × 2^pitch`: with nothing in `pitch` it plays FREQ;
//! a sequencer's pitch (octaves, 0 = C4) plays true notes at the default
//! FREQ of C4.

mod dsp;
mod view;

pub use dsp::Sine;

use box_core::pitch::C4_HZ;
use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;
use crate::format::{short_hz, short_percent};

/// Stable kind id.
pub const KIND: &str = "box.sine";

const FREQ: usize = 0;
const LEVEL: usize = 1;

const PITCH: usize = 0;

static INPUTS: [PortSpec; 1] = [PortSpec::audio("pitch", "Pitch", Side::Left, 0)];
static OUTPUTS: [PortSpec; 1] = [PortSpec::audio("out", "Out", Side::Right, 0)];
static PARAMS: [ParamSpec; 2] = [
    ParamSpec::float("freq", "FREQ", 20.0, 20_000.0, C4_HZ, Taper::Log)
        .unit("Hz")
        .formatter(short_hz)
        .smoothing(20.0),
    ParamSpec::float("level", "LEVEL", 0.0, 1.0, 0.5, Taper::Linear)
        .formatter(short_percent)
        .smoothing(20.0),
];

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "SINE",
    category: "Sources",
    size: (1, 1),
    inputs: &INPUTS,
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &[],
    role: Role::Normal,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};
