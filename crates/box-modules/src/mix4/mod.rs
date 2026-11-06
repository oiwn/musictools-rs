//! Mix4 — four mono channels with level and pan into a stereo pair (4x1).
//!
//! Pan is constant-power: −3 dB per side at center, full level on one side
//! at the extremes.

mod dsp;
mod view;

pub use dsp::Mix4;

use box_core::{ModuleDescriptor, ParamSpec, PortSpec, Role, Side, Taper};

use crate::ModuleEntry;
use crate::format::short_db;

/// Stable kind id.
pub const KIND: &str = "box.mix4";

/// Channels.
const CHANNELS: usize = 4;

/// Param index of channel `channel`'s level; its pan follows.
const fn level(channel: usize) -> usize {
    2 * channel
}

const fn pan(channel: usize) -> usize {
    2 * channel + 1
}

static INPUTS: [PortSpec; CHANNELS] = [
    PortSpec::audio("1", "In 1", Side::Left, 0),
    PortSpec::audio("2", "In 2", Side::Left, 1),
    PortSpec::audio("3", "In 3", Side::Left, 2),
    PortSpec::audio("4", "In 4", Side::Left, 3),
];
static OUTPUTS: [PortSpec; 2] = [
    PortSpec::audio("l", "L", Side::Right, 0),
    PortSpec::audio("r", "R", Side::Right, 1),
];

const fn level_spec(id: &'static str, name: &'static str) -> ParamSpec {
    ParamSpec::float(id, name, 0.0, 1.0, 0.7, Taper::Skew(2.0))
        .formatter(short_db)
        .smoothing(20.0)
}

const fn pan_spec(id: &'static str, name: &'static str) -> ParamSpec {
    ParamSpec::float(id, name, -1.0, 1.0, 0.0, Taper::Linear)
        .formatter(format_pan)
        .smoothing(20.0)
}

static PARAMS: [ParamSpec; 2 * CHANNELS] = [
    level_spec("level1", "LVL 1"),
    pan_spec("pan1", "PAN 1"),
    level_spec("level2", "LVL 2"),
    pan_spec("pan2", "PAN 2"),
    level_spec("level3", "LVL 3"),
    pan_spec("pan3", "PAN 3"),
    level_spec("level4", "LVL 4"),
    pan_spec("pan4", "PAN 4"),
];

/// `C`, `L40`, `R100`.
fn format_pan(value: f32) -> String {
    let percent = (value * 100.0).round();
    if percent == 0.0 {
        "C".to_owned()
    } else if percent < 0.0 {
        format!("L{:.0}", -percent)
    } else {
        format!("R{percent:.0}")
    }
}

pub static DESCRIPTOR: ModuleDescriptor = ModuleDescriptor {
    kind: KIND,
    name: "MIX4",
    category: "Mixing",
    size: (4, 1),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_reads_left_center_right() {
        assert_eq!(format_pan(0.0), "C");
        assert_eq!(format_pan(-0.4), "L40");
        assert_eq!(format_pan(1.0), "R100");
    }
}
