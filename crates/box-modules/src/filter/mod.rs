//! Filter — Roman's biquad as a box tile (2x1): the first plugin DSP in
//! box. Lowpass, highpass, bandpass or notch at 12 or 24 dB/oct.
//!
//! The `cutoff` input is a pitch-style modulation: the cutoff is
//! multiplied by `2^input`, so +1.0 doubles it.

mod dsp;
mod view;

pub use dsp::Filter;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;
use crate::format::short_hz;

/// Stable kind id.
pub const KIND: &str = "box.filter";

const CUTOFF: usize = 0;
const Q: usize = 1;
const TYPE: usize = 2;
const SLOPE: usize = 3;

const IN: usize = 0;
const CUTOFF_IN: usize = 1;

/// Filter types offered on the TYPE knob, in `roman_dsp::FilterType`
/// index order.
const TYPES: [&str; 4] = ["LP", "HP", "BP", "NOTCH"];

static INPUTS: [PortSpec; 2] = [
    PortSpec::audio("in", "In", Side::Left, 0),
    PortSpec::audio("cutoff", "Cutoff", Side::Left, 1),
];
static OUTPUTS: [PortSpec; 1] = [PortSpec::audio("out", "Out", Side::Right, 0)];
static PARAMS: [ParamSpec; 4] = [
    ParamSpec::float("cutoff", "CUTOFF", 20.0, 20_000.0, 1000.0, Taper::Log)
        .unit("Hz")
        .formatter(short_hz)
        .smoothing(30.0),
    ParamSpec::float("q", "Q", 0.3, 12.0, 0.707, Taper::Log)
        .formatter(format_q)
        .smoothing(30.0),
    ParamSpec::choice("type", "TYPE", &TYPES, 0),
    ParamSpec::bool("slope24", "24 dB", false),
];

fn format_q(value: f32) -> String {
    format!("{value:.2}")
}

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "FILTER",
    category: "Filters",
    size: (2, 1),
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
