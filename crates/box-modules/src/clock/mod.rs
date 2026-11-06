//! Clock — a BPM clock (1x1).
//!
//! `clock` is a 50 % gate at the chosen division; `beat` is a 50 % gate
//! on quarter notes. Both come from one beat phase, so they line up on
//! every downbeat and never drift apart.

mod dsp;
mod view;

pub use dsp::Clock;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;

/// Stable kind id.
pub const KIND: &str = "box.clock";

const BPM: usize = 0;
const DIV: usize = 1;

/// Division choices and how many clock periods each fits in a beat.
const DIVISIONS: [&str; 4] = ["1/4", "1/8", "1/16", "1/32"];
const PER_BEAT: [f64; 4] = [1.0, 2.0, 4.0, 8.0];

/// Meter: the clock output, for the header lamp.
const CLOCK_METER: usize = 0;

static OUTPUTS: [PortSpec; 2] = [
    PortSpec::audio("clock", "Clock", Side::Right, 0),
    PortSpec::audio("beat", "Beat", Side::Right, 1),
];
static PARAMS: [ParamSpec; 2] = [
    ParamSpec::float("bpm", "BPM", 20.0, 300.0, 120.0, Taper::Linear).formatter(format_bpm),
    ParamSpec::choice("div", "DIV", &DIVISIONS, 1),
];

fn format_bpm(value: f32) -> String {
    format!("{value:.0}")
}

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "CLOCK",
    category: "Sources",
    size: (1, 1),
    inputs: &[],
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &["clock"],
    role: Role::Normal,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};
