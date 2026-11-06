//! The graph compiler: patch → flat run order with reused buffers.
//!
//! Runs off the audio thread. Buffer 0 is always silence (unconnected
//! inputs read it); every other buffer is handed out by liveness, so a long
//! chain needs only a couple of buffers however many modules it has.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::ops::Range;

use box_core::{MAX_PORTS, ModuleDescriptor, Role};

use crate::Patch;

/// Looks a module kind up in the registry.
pub type Registry = fn(&str) -> Option<&'static ModuleDescriptor>;

/// The silent buffer every unconnected input reads.
pub const ZERO_BUFFER: usize = 0;

/// Why a patch can't compile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// The cables form a cycle; v1 graphs are acyclic.
    Cycle,
    /// Two nodes share a uid.
    DuplicateUid(u64),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cycle => write!(f, "that wire would close a loop"),
            Self::DuplicateUid(uid) => write!(f, "two modules share uid {uid}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// One step of the run order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Sum `plan.sum_sources[sources]` into buffer `into` (fan-in).
    Sum { into: usize, sources: Range<usize> },
    /// Run `plan.nodes[index]`.
    Run(usize),
}

/// One node of the run order, with its buffers.
#[derive(Clone, Copy, Debug)]
pub struct PlanNode {
    pub uid: u64,
    pub descriptor: &'static ModuleDescriptor,
    pub powered: bool,
    /// Buffer read by each input (`ZERO_BUFFER` when unconnected).
    pub inputs: [usize; MAX_PORTS],
    /// Buffer written by each output.
    pub outputs: [usize; MAX_PORTS],
    /// Bit `i` set when input `i` has a wire.
    pub connected: u32,
}

/// A compiled patch: what to run, in which order, over which buffers.
#[derive(Clone, Debug)]
pub struct Plan {
    /// Nodes in run order (culled nodes are absent).
    pub nodes: Vec<PlanNode>,
    pub steps: Vec<Step>,
    pub sum_sources: Vec<usize>,
    /// Buffers needed, including the silent buffer 0.
    pub buffer_count: usize,
    /// (L, R) output buffers of every sink.
    pub sinks: Vec<[usize; 2]>,
    /// Cables ignored because an end is missing or unknown.
    pub skipped_cables: usize,
}

struct Edge {
    from_node: usize,
    from_port: usize,
    to_node: usize,
    to_port: usize,
}

/// Hands out buffers by liveness.
struct Buffers {
    refs: Vec<u32>,
    free: Vec<usize>,
}

impl Buffers {
    fn alloc(&mut self, refs: u32) -> usize {
        if let Some(index) = self.free.pop() {
            self.refs[index] = refs;
            index
        } else {
            self.refs.push(refs);
            self.refs.len() - 1
        }
    }

    fn release(&mut self, index: usize) {
        if index == ZERO_BUFFER || self.refs[index] == u32::MAX {
            return;
        }
        self.refs[index] = self.refs[index].saturating_sub(1);
        if self.refs[index] == 0 {
            self.free.push(index);
        }
    }
}

/// Compiles `patch`. Nodes of unknown kinds and cables to missing ports are
/// skipped; a cycle anywhere refuses the whole patch.
pub fn compile(patch: &Patch, registry: Registry) -> Result<Plan, CompileError> {
    // Known nodes, in patch order.
    let mut known: Vec<(usize, &'static ModuleDescriptor)> = Vec::new();
    let mut index_of: HashMap<u64, usize> = HashMap::new();
    for (patch_index, node) in patch.nodes.iter().enumerate() {
        if index_of.contains_key(&node.uid) {
            return Err(CompileError::DuplicateUid(node.uid));
        }
        if let Some(descriptor) = registry(&node.kind) {
            index_of.insert(node.uid, known.len());
            known.push((patch_index, descriptor));
        }
    }

    let mut edges = Vec::new();
    let mut skipped_cables = 0;
    for cable in &patch.cables {
        let edge = (|| {
            let from_node = *index_of.get(&cable.from.uid)?;
            let to_node = *index_of.get(&cable.to.uid)?;
            let from = known[from_node].1;
            if from.role == Role::Sink {
                return None;
            }
            Some(Edge {
                from_node,
                from_port: from.output_index(&cable.from.port)?,
                to_node,
                to_port: known[to_node].1.input_index(&cable.to.port)?,
            })
        })();
        match edge {
            Some(edge) => edges.push(edge),
            None => skipped_cables += 1,
        }
    }

    // Kahn's sort over every known node, so a cycle is refused even where
    // it would be culled.
    let count = known.len();
    let mut indegree = vec![0_usize; count];
    for edge in &edges {
        indegree[edge.to_node] += 1;
    }
    let mut queue: VecDeque<usize> = (0..count).filter(|&n| indegree[n] == 0).collect();
    let mut order = Vec::with_capacity(count);
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for edge in edges.iter().filter(|edge| edge.from_node == node) {
            indegree[edge.to_node] -= 1;
            if indegree[edge.to_node] == 0 {
                queue.push_back(edge.to_node);
            }
        }
    }
    if order.len() < count {
        return Err(CompileError::Cycle);
    }

    // Keep only nodes with a path to a sink.
    let mut live = vec![false; count];
    let mut stack: Vec<usize> = (0..count)
        .filter(|&n| known[n].1.role == Role::Sink)
        .collect();
    while let Some(node) = stack.pop() {
        if std::mem::replace(&mut live[node], true) {
            continue;
        }
        stack.extend(
            edges
                .iter()
                .filter(|edge| edge.to_node == node)
                .map(|edge| edge.from_node),
        );
    }
    order.retain(|&n| live[n]);

    let mut buffers = Buffers {
        refs: vec![u32::MAX],
        free: Vec::new(),
    };
    let mut output_buffer: HashMap<(usize, usize), usize> = HashMap::new();
    let mut plan = Plan {
        nodes: Vec::with_capacity(order.len()),
        steps: Vec::new(),
        sum_sources: Vec::new(),
        buffer_count: 0,
        sinks: Vec::new(),
        skipped_cables,
    };

    for &node in &order {
        let (patch_index, descriptor) = known[node];
        let mut planned = PlanNode {
            uid: patch.nodes[patch_index].uid,
            descriptor,
            powered: patch.nodes[patch_index].powered,
            inputs: [ZERO_BUFFER; MAX_PORTS],
            outputs: [ZERO_BUFFER; MAX_PORTS],
            connected: 0,
        };

        for port in 0..descriptor.inputs.len() {
            let sources: Vec<usize> = edges
                .iter()
                .filter(|edge| edge.to_node == node && edge.to_port == port)
                .map(|edge| output_buffer[&(edge.from_node, edge.from_port)])
                .collect();
            planned.inputs[port] = match sources.as_slice() {
                [] => ZERO_BUFFER,
                [single] => *single,
                many => {
                    let into = buffers.alloc(1);
                    let start = plan.sum_sources.len();
                    plan.sum_sources.extend_from_slice(many);
                    plan.steps.push(Step::Sum {
                        into,
                        sources: start..plan.sum_sources.len(),
                    });
                    for &source in many {
                        buffers.release(source);
                    }
                    into
                }
            };
            if !sources.is_empty() {
                planned.connected |= 1 << port;
            }
        }

        for port in 0..descriptor.outputs.len() {
            let consumers = if descriptor.role == Role::Sink {
                u32::MAX
            } else {
                edges
                    .iter()
                    .filter(|edge| {
                        edge.from_node == node && edge.from_port == port && live[edge.to_node]
                    })
                    .count() as u32
            };
            let buffer = buffers.alloc(consumers.max(1));
            planned.outputs[port] = buffer;
            output_buffer.insert((node, port), buffer);
        }

        plan.steps.push(Step::Run(plan.nodes.len()));
        for port in 0..descriptor.inputs.len() {
            buffers.release(planned.inputs[port]);
        }
        for port in 0..descriptor.outputs.len() {
            let buffer = planned.outputs[port];
            let consumed = edges
                .iter()
                .any(|edge| edge.from_node == node && edge.from_port == port && live[edge.to_node]);
            if !consumed {
                buffers.release(buffer);
            }
        }
        if descriptor.role == Role::Sink {
            plan.sinks.push([planned.outputs[0], planned.outputs[1]]);
        }
        plan.nodes.push(planned);
    }
    plan.buffer_count = buffers.refs.len();
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toys::{self, registry};
    use crate::{Cable, PatchNode};

    fn chain(gains: usize) -> Patch {
        let mut patch = Patch::default();
        patch.nodes.push(PatchNode::new(1, toys::CONST, (0, 0)));
        for index in 0..gains {
            let uid = index as u64 + 2;
            patch.nodes.push(PatchNode::new(uid, toys::GAIN, (0, 0)));
            patch.cables.push(Cable::new(uid - 1, "out", uid, "in"));
        }
        let sink = gains as u64 + 2;
        patch.nodes.push(PatchNode::new(sink, toys::SINK, (0, 0)));
        patch.cables.push(Cable::new(sink - 1, "out", sink, "l"));
        patch
    }

    #[test]
    fn long_chains_reuse_buffers() {
        let plan = compile(&chain(32), registry).unwrap();
        assert_eq!(plan.nodes.len(), 34);
        // Silence, two ping-pong buffers, and the sink's two outputs.
        assert!(plan.buffer_count <= 5, "{} buffers", plan.buffer_count);
    }

    #[test]
    fn a_cycle_is_refused() {
        let mut patch = chain(2);
        patch.cables.push(Cable::new(3, "out", 2, "in"));
        assert_eq!(compile(&patch, registry).unwrap_err(), CompileError::Cycle);
    }

    #[test]
    fn nodes_without_a_path_to_a_sink_are_culled() {
        let mut patch = chain(1);
        patch.nodes.push(PatchNode::new(50, toys::COUNTER, (0, 0)));
        let plan = compile(&patch, registry).unwrap();
        assert!(plan.nodes.iter().all(|node| node.uid != 50));
    }

    #[test]
    fn fan_in_adds_a_sum_step() {
        let mut patch = chain(0);
        patch.nodes.push(PatchNode::new(9, toys::CONST, (0, 0)));
        patch.cables.push(Cable::new(9, "out", 2, "l"));
        let plan = compile(&patch, registry).unwrap();
        assert!(
            matches!(plan.steps[2], Step::Sum { .. }),
            "{:?}",
            plan.steps
        );
    }

    #[test]
    fn unknown_kinds_and_ports_are_skipped() {
        let mut patch = chain(1);
        patch.nodes.push(PatchNode::new(70, "nope", (0, 0)));
        patch.cables.push(Cable::new(70, "out", 2, "in"));
        patch.cables.push(Cable::new(1, "missing", 2, "in"));
        let plan = compile(&patch, registry).unwrap();
        assert_eq!(plan.skipped_cables, 2);
        assert_eq!(plan.nodes.len(), 3);
    }
}
