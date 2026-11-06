//! Patch edits behind the canvas: wiring, unwiring, deleting, and the
//! geometry to hit-test ports and wires. Pure functions over a `Patch`.

use box_dsp::{Cable, CompileError, Patch, PortRef};
use egui::Pos2;

/// One end of a wire being drawn: a port of a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortEnd {
    pub uid: u64,
    pub port: &'static str,
    /// True for an input port.
    pub input: bool,
}

/// Why a wire wasn't made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    /// Both ends are inputs, or both outputs.
    SameDirection,
    /// The wire would close a loop.
    Cycle,
}

impl WireError {
    /// A short hint for the status line.
    #[must_use]
    pub fn hint(self) -> &'static str {
        match self {
            Self::SameDirection => "wire an output to an input",
            Self::Cycle => "that wire would close a loop",
        }
    }
}

/// Wires `a` to `b` in either drag direction. Returns `Ok(false)` when the
/// same wire already exists.
pub fn connect(patch: &mut Patch, a: PortEnd, b: PortEnd) -> Result<bool, WireError> {
    let (from, to) = match (a.input, b.input) {
        (false, true) => (a, b),
        (true, false) => (b, a),
        _ => return Err(WireError::SameDirection),
    };
    let cable = Cable::new(from.uid, from.port, to.uid, to.port);
    if patch.cables.contains(&cable) {
        return Ok(false);
    }
    patch.cables.push(cable);
    match box_dsp::compile(patch, box_modules::registry) {
        Err(CompileError::Cycle) => {
            patch.cables.pop();
            Err(WireError::Cycle)
        }
        _ => Ok(true),
    }
}

/// Removes the node `uid` and every wire touching it.
pub fn delete_node(patch: &mut Patch, uid: u64) -> bool {
    let before = patch.nodes.len();
    patch.nodes.retain(|node| node.uid != uid);
    patch
        .cables
        .retain(|cable| cable.from.uid != uid && cable.to.uid != uid);
    patch.nodes.len() != before
}

/// True when `end` is the given end of `cable`.
#[must_use]
pub fn touches(end: &PortRef, uid: u64, port: &str) -> bool {
    end.uid == uid && end.port == port
}

/// Distance from `point` to the polyline `line`.
#[must_use]
pub fn distance_to_line(line: &[Pos2], point: Pos2) -> f32 {
    line.windows(2)
        .map(|segment| distance_to_segment(segment[0], segment[1], point))
        .fold(f32::INFINITY, f32::min)
}

fn distance_to_segment(a: Pos2, b: Pos2, point: Pos2) -> f32 {
    let along = b - a;
    let length = along.length_sq();
    if length <= f32::EPSILON {
        return (point - a).length();
    }
    let t = ((point - a).dot(along) / length).clamp(0.0, 1.0);
    (point - (a + along * t)).length()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch_io::default_patch;
    use box_modules::{env, gates, output, sine};
    use egui::pos2;

    fn out(uid: u64, port: &'static str) -> PortEnd {
        PortEnd {
            uid,
            port,
            input: false,
        }
    }

    fn input(uid: u64, port: &'static str) -> PortEnd {
        PortEnd {
            uid,
            port,
            input: true,
        }
    }

    fn empty() -> Patch {
        let mut patch = default_patch();
        patch.cables.clear();
        patch
    }

    #[test]
    fn wires_go_output_to_input_in_either_drag_direction() {
        let mut patch = empty();
        assert_eq!(connect(&mut patch, out(1, "out"), input(3, "in")), Ok(true));
        assert_eq!(connect(&mut patch, input(4, "l"), out(3, "out")), Ok(true));
        assert_eq!(patch.cables[1], Cable::new(3, "out", 4, "l"));
        assert_eq!(
            connect(&mut patch, out(1, "out"), input(3, "in")),
            Ok(false)
        );
        assert_eq!(patch.cables.len(), 2);
    }

    #[test]
    fn same_direction_wires_are_refused() {
        let mut patch = empty();
        assert_eq!(
            connect(&mut patch, out(1, "out"), out(2, "gate")),
            Err(WireError::SameDirection)
        );
        assert_eq!(
            connect(&mut patch, input(3, "in"), input(4, "l")),
            Err(WireError::SameDirection)
        );
        assert!(patch.cables.is_empty());
    }

    #[test]
    fn a_loop_is_refused_and_leaves_the_patch_alone() {
        let mut patch = empty();
        patch
            .nodes
            .push(box_dsp::PatchNode::new(9, env::KIND, (3, 3)));
        connect(&mut patch, out(3, "out"), input(9, "in")).unwrap();
        assert_eq!(
            connect(&mut patch, out(9, "out"), input(3, "in")),
            Err(WireError::Cycle)
        );
        assert_eq!(patch.cables.len(), 1);
        // A module into itself is a loop too.
        assert_eq!(
            connect(&mut patch, out(3, "out"), input(3, "gate")),
            Err(WireError::Cycle)
        );
    }

    #[test]
    fn deleting_a_tile_drops_its_wires() {
        let mut patch = default_patch();
        assert!(delete_node(&mut patch, 3));
        assert_eq!(patch.nodes.len(), 3);
        assert!(patch.cables.is_empty(), "every default wire touches Env");
        assert!(!delete_node(&mut patch, 3));
        let kinds: Vec<&str> = patch.nodes.iter().map(|node| node.kind.as_str()).collect();
        assert_eq!(kinds, [sine::KIND, gates::KIND, output::KIND]);
    }

    #[test]
    fn edits_round_trip_through_json() {
        let mut patch = empty();
        connect(&mut patch, out(2, "gate"), input(3, "gate")).unwrap();
        delete_node(&mut patch, 1);
        let back = Patch::from_json(&patch.to_json()).unwrap();
        assert_eq!(back, patch);
    }

    #[test]
    fn distance_to_a_polyline() {
        let line = [pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0)];
        assert_eq!(distance_to_line(&line, pos2(5.0, 3.0)), 3.0);
        assert_eq!(distance_to_line(&line, pos2(12.0, 5.0)), 2.0);
        assert_eq!(distance_to_line(&line, pos2(-4.0, 3.0)), 5.0);
        assert!(distance_to_line(&[], pos2(0.0, 0.0)).is_infinite());
    }
}
