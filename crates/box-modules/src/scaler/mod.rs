//! Scaler — quantizes pitch to a root and scale (2x1).
//!
//! The catalog in [`scales`] holds diatonic modes, exotic 12-tone scales
//! and microtonal ones (quarter-tone maqamat, gamelan tunings, 19- and
//! 24-TET); every scale is a list of cents.

mod dsp;
pub mod scales;
mod view;

pub use dsp::Scaler;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side};

use crate::ModuleEntry;

/// Stable kind id.
pub const KIND: &str = "box.scaler";

const ROOT: usize = 0;
const SCALE: usize = 1;

const PITCH_IN: usize = 0;

/// Root note names, C first.
const ROOTS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

static INPUTS: [PortSpec; 1] = [PortSpec::audio("pitch", "Pitch", Side::Left, 0)];
static OUTPUTS: [PortSpec; 1] = [PortSpec::audio("pitch", "Pitch", Side::Right, 0)];
static PARAMS: [ParamSpec; 2] = [
    ParamSpec::choice("root", "ROOT", &ROOTS, 0),
    ParamSpec::choice("scale", "SCALE", &scales::NAMES, 0).formatter(scale_number),
];

/// The knob shows the scale's number; the tile prints its name.
fn scale_number(value: f32) -> String {
    format!("{} / {}", value.round() as usize + 1, scales::SCALES.len())
}

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "SCALER",
    category: "Pitch",
    size: (2, 1),
    inputs: &INPUTS,
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &["degree"],
    role: Role::Normal,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};
