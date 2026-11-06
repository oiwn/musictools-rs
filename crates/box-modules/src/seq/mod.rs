//! Seq — an 8-step pitch and gate sequencer (4x2).
//!
//! Each rising `clock` edge moves to the next step (the first clock after
//! start or reset plays step 1). `pitch` holds the step's pitch in
//! octaves (whole semitones); `gate` follows the clock while the step is
//! on. A rising `reset` makes the next clock play step 1 again.

mod dsp;
mod view;

pub use dsp::Seq;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;

/// Stable kind id.
pub const KIND: &str = "box.seq";

/// Steps in the sequence.
pub const STEPS: usize = 8;

/// Param index of step `step`'s pitch (semitones).
const fn pitch(step: usize) -> usize {
    step
}

/// Param index of step `step`'s gate on/off.
const fn gate(step: usize) -> usize {
    STEPS + step
}

const CLOCK: usize = 0;
const RESET: usize = 1;

/// Meter: the current step (0-based), or −1 before the first clock.
const STEP_METER: usize = 0;

static INPUTS: [PortSpec; 2] = [
    PortSpec::audio("clock", "Clock", Side::Left, 0),
    PortSpec::audio("reset", "Reset", Side::Left, 1),
];
static OUTPUTS: [PortSpec; 2] = [
    PortSpec::audio("pitch", "Pitch", Side::Right, 0),
    PortSpec::audio("gate", "Gate", Side::Right, 1),
];

const fn pitch_spec(id: &'static str, name: &'static str) -> ParamSpec {
    ParamSpec::float(id, name, -24.0, 24.0, 0.0, Taper::Linear).formatter(format_semitones)
}

static PARAMS: [ParamSpec; 2 * STEPS] = [
    pitch_spec("pitch1", "1"),
    pitch_spec("pitch2", "2"),
    pitch_spec("pitch3", "3"),
    pitch_spec("pitch4", "4"),
    pitch_spec("pitch5", "5"),
    pitch_spec("pitch6", "6"),
    pitch_spec("pitch7", "7"),
    pitch_spec("pitch8", "8"),
    ParamSpec::bool("gate1", "G1", true),
    ParamSpec::bool("gate2", "G2", true),
    ParamSpec::bool("gate3", "G3", true),
    ParamSpec::bool("gate4", "G4", true),
    ParamSpec::bool("gate5", "G5", true),
    ParamSpec::bool("gate6", "G6", true),
    ParamSpec::bool("gate7", "G7", true),
    ParamSpec::bool("gate8", "G8", true),
];

/// A step's pitch as whole semitones: `+7 st`, `0 st`, `-12 st`.
fn format_semitones(value: f32) -> String {
    let semitones = value.round();
    if semitones > 0.0 {
        format!("+{semitones:.0} st")
    } else {
        format!("{semitones:.0} st")
    }
}

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "SEQ",
    category: "Sequencing",
    size: (4, 2),
    inputs: &INPUTS,
    outputs: &OUTPUTS,
    params: &PARAMS,
    meters: &["step"],
    role: Role::Normal,
    create: dsp::create,
};

pub const ENTRY: ModuleEntry = ModuleEntry {
    descriptor: &DESCRIPTOR,
    view: view::view,
    layout: view::layout,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semitones_read_signed_and_whole() {
        assert_eq!(format_semitones(7.2), "+7 st");
        assert_eq!(format_semitones(0.0), "0 st");
        assert_eq!(format_semitones(-11.6), "-12 st");
    }
}
