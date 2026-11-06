//! The default patch and JSON save/load.

use std::path::Path;

use box_dsp::{Cable, Patch, PatchNode};
use box_modules::{clock, env, filter, mix4, output, scaler, seq, sine};

/// Where patches go when no path argument is given.
pub const DEFAULT_PATH: &str = "box-patch.json";

/// A node with some params set by id.
fn node(uid: u64, kind: &str, cell: (i32, i32), params: &[(&str, f32)]) -> PatchNode {
    let mut node = PatchNode::new(uid, kind, cell);
    for (id, value) in params {
        node.params.insert((*id).to_owned(), *value);
    }
    node
}

/// The first-launch patch: a clocked, scaled 8-step line through a filter,
/// laid out as a snake so every row reads left to right:
/// Clock → Seq → Scaler → Sine on top; Env (gated by Seq right above it)
/// → Filter in the middle; Mix4 → Output at the bottom.
#[must_use]
pub fn demo_patch() -> Patch {
    let melody = [0.0, 3.0, 7.0, 10.0, 12.0, 7.0, 5.0, 3.0];
    let mut seq_params: Vec<(String, f32)> = melody
        .iter()
        .enumerate()
        .map(|(step, &semitones)| (format!("pitch{}", step + 1), semitones))
        .collect();
    seq_params.push(("gate6".to_owned(), 0.0));
    let mut sequencer = PatchNode::new(2, seq::KIND, (2, 0));
    sequencer.params = seq_params.into_iter().collect();
    Patch {
        nodes: vec![
            node(1, clock::KIND, (0, 0), &[("bpm", 112.0), ("div", 1.0)]),
            sequencer,
            // Minor pentatonic, rooted on A.
            node(3, scaler::KIND, (7, 0), &[("root", 9.0), ("scale", 10.0)]),
            node(4, sine::KIND, (10, 0), &[("freq", 130.81), ("level", 0.8)]),
            node(
                5,
                env::KIND,
                (7, 2),
                &[
                    ("attack", 2.0),
                    ("decay", 180.0),
                    ("sustain", 0.25),
                    ("release", 160.0),
                ],
            ),
            node(
                6,
                filter::KIND,
                (10, 2),
                &[("cutoff", 1400.0), ("q", 2.0), ("type", 0.0)],
            ),
            node(7, mix4::KIND, (7, 4), &[("level1", 0.8)]),
            node(8, output::KIND, (12, 4), &[]),
        ],
        cables: vec![
            Cable::new(1, "clock", 2, "clock"),
            Cable::new(2, "pitch", 3, "pitch"),
            Cable::new(3, "pitch", 4, "pitch"),
            Cable::new(4, "out", 5, "in"),
            Cable::new(2, "gate", 5, "gate"),
            Cable::new(5, "out", 6, "in"),
            Cable::new(6, "out", 7, "1"),
            Cable::new(7, "l", 8, "l"),
            Cable::new(7, "r", 8, "r"),
        ],
    }
}

/// The basic gated sine (Sine and Gates into Env into Output). The tests
/// build on it; the app opens [`demo_patch`].
#[cfg(test)]
#[must_use]
pub fn default_patch() -> Patch {
    use box_modules::gates;
    Patch {
        nodes: vec![
            PatchNode::new(1, sine::KIND, (1, 1)),
            PatchNode::new(2, gates::KIND, (1, 2)),
            PatchNode::new(3, env::KIND, (3, 1)),
            PatchNode::new(4, output::KIND, (6, 1)),
        ],
        cables: vec![
            Cable::new(1, "out", 3, "in"),
            Cable::new(2, "gate", 3, "gate"),
            Cable::new(3, "out", 4, "l"),
        ],
    }
}

/// Reads a patch file.
pub fn load(path: &Path) -> Result<Patch, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Patch::from_json(&text).map_err(|error| format!("bad patch {}: {error}", path.display()))
}

/// Writes a patch file.
pub fn save(path: &Path, patch: &Patch) -> Result<(), String> {
    std::fs::write(path, patch.to_json())
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_patch_compiles_with_every_node_and_known_params() {
        let patch = demo_patch();
        let plan = box_dsp::compile(&patch, box_modules::registry).unwrap();
        assert_eq!(plan.nodes.len(), patch.nodes.len());
        assert_eq!(plan.skipped_cables, 0);
        // Every saved param id exists on its module.
        for node in &patch.nodes {
            let descriptor = box_modules::registry(&node.kind).unwrap();
            for id in node.params.keys() {
                assert!(
                    descriptor.params.iter().any(|spec| spec.id == id),
                    "{} has no param {id}",
                    node.kind
                );
            }
        }
        // The scaler starts on minor pentatonic.
        assert_eq!(box_modules::scaler::scales::NAMES[10], "Minor pentatonic");
    }

    #[test]
    fn demo_tiles_do_not_overlap() {
        let patch = demo_patch();
        for node in &patch.nodes {
            assert!(
                !crate::canvas::overlaps(&patch, node.uid, node.cell),
                "{} overlaps",
                node.kind
            );
        }
    }

    #[test]
    fn default_patch_compiles_with_every_node() {
        let plan = box_dsp::compile(&default_patch(), box_modules::registry).unwrap();
        assert_eq!(plan.nodes.len(), 4);
        assert_eq!(plan.skipped_cables, 0);
        assert_eq!(plan.sinks.len(), 1);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("box-app-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("patch.json");
        let mut patch = default_patch();
        patch.nodes[0].params.insert("freq".into(), 330.0);
        patch.nodes[2].powered = false;
        save(&path, &patch).unwrap();
        assert_eq!(load(&path).unwrap(), patch);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_bad_file_reports_an_error() {
        assert!(load(Path::new("/definitely/not/here.json")).is_err());
    }
}
