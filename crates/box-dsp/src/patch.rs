//! The saved patch: nodes on the cell grid and the cables between ports.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A whole patch, as saved to JSON.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    #[serde(default)]
    pub nodes: Vec<PatchNode>,
    #[serde(default)]
    pub cables: Vec<Cable>,
}

/// One module placed on the grid.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatchNode {
    /// Unique within the patch; survives re-patching and save/load.
    pub uid: u64,
    /// The module's stable `kind` id.
    pub kind: String,
    /// Top-left cell (column, row).
    pub cell: (i32, i32),
    /// Plain param values by stable id; missing ids take defaults.
    #[serde(default)]
    pub params: BTreeMap<String, f32>,
    /// False bypasses the module (inputs copied through, extra outputs silent).
    #[serde(default = "powered_default")]
    pub powered: bool,
}

fn powered_default() -> bool {
    true
}

/// One port of one node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRef {
    pub uid: u64,
    pub port: String,
}

/// A wire from an output port to an input port.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cable {
    pub from: PortRef,
    pub to: PortRef,
}

impl Cable {
    /// A cable from `(from_uid, from_port)` to `(to_uid, to_port)`.
    #[must_use]
    pub fn new(from_uid: u64, from_port: &str, to_uid: u64, to_port: &str) -> Self {
        Self {
            from: PortRef {
                uid: from_uid,
                port: from_port.to_owned(),
            },
            to: PortRef {
                uid: to_uid,
                port: to_port.to_owned(),
            },
        }
    }
}

impl PatchNode {
    /// A powered node with default params.
    #[must_use]
    pub fn new(uid: u64, kind: &str, cell: (i32, i32)) -> Self {
        Self {
            uid,
            kind: kind.to_owned(),
            cell,
            params: BTreeMap::new(),
            powered: true,
        }
    }
}

impl Patch {
    /// The node with `uid`.
    #[must_use]
    pub fn node(&self, uid: u64) -> Option<&PatchNode> {
        self.nodes.iter().find(|node| node.uid == uid)
    }

    /// The node with `uid`, mutably.
    pub fn node_mut(&mut self, uid: u64) -> Option<&mut PatchNode> {
        self.nodes.iter_mut().find(|node| node.uid == uid)
    }

    /// A uid no node uses yet.
    #[must_use]
    pub fn next_uid(&self) -> u64 {
        self.nodes
            .iter()
            .map(|node| node.uid)
            .max()
            .map_or(1, |max| max + 1)
    }

    /// Pretty JSON.
    ///
    /// # Panics
    /// Never in practice: every field serializes.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a patch always serializes")
    }

    /// Parses JSON written by [`Patch::to_json`].
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let mut node = PatchNode::new(3, "box.sine", (1, 2));
        node.params.insert("freq".into(), 220.0);
        let patch = Patch {
            nodes: vec![node, PatchNode::new(4, "box.output", (3, 0))],
            cables: vec![Cable::new(3, "out", 4, "l")],
        };
        let back = Patch::from_json(&patch.to_json()).unwrap();
        assert_eq!(back, patch);
        assert_eq!(back.next_uid(), 5);
    }

    #[test]
    fn missing_fields_take_defaults() {
        let patch = Patch::from_json(r#"{"nodes":[{"uid":1,"kind":"x","cell":[0,0]}]}"#).unwrap();
        assert!(patch.nodes[0].powered);
        assert!(patch.nodes[0].params.is_empty());
        assert!(patch.cables.is_empty());
    }
}
