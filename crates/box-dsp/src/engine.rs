//! The UI-side [`Controller`] and the audio-side [`Engine`], joined by two
//! lock-free queues: new graphs go to audio, old graphs come back.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use box_core::Shared;

use crate::graph::Graph;
use crate::{CompileError, Patch, PatchNode, Plan, Registry, compile};

/// Graphs that may wait in either queue.
const QUEUE: usize = 8;

/// Why a patch wasn't applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    Compile(CompileError),
    /// The audio side hasn't taken the previous graphs yet (is it running?).
    QueueFull,
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(error) => error.fmt(f),
            Self::QueueFull => write!(f, "the audio engine is not taking new patches"),
        }
    }
}

impl std::error::Error for ApplyError {}

impl From<CompileError> for ApplyError {
    fn from(error: CompileError) -> Self {
        Self::Compile(error)
    }
}

/// A connected controller and engine for one sample rate and block size.
#[must_use]
pub fn new_engine(registry: Registry, sample_rate: f32, max_block: usize) -> (Controller, Engine) {
    let max_block = max_block.max(1);
    let (to_audio, from_ui) = rtrb::RingBuffer::new(QUEUE);
    let (to_ui, from_audio) = rtrb::RingBuffer::new(QUEUE);
    let controller = Controller {
        registry,
        sample_rate,
        max_block,
        shared: HashMap::new(),
        installed: HashMap::new(),
        to_audio,
        from_audio,
    };
    let engine = Engine {
        graph: Box::new(Graph::empty()),
        from_ui,
        to_ui,
        pending: None,
        max_block,
    };
    (controller, engine)
}

/// The UI side: compiles patches, owns each node's [`Shared`], and sends
/// graphs to the engine.
pub struct Controller {
    registry: Registry,
    sample_rate: f32,
    max_block: usize,
    /// Each node's live state, with the kind it was made for.
    shared: HashMap<u64, (&'static str, Arc<Shared>)>,
    /// Uid → kind of the modules living in the last graph sent.
    installed: HashMap<u64, &'static str>,
    to_audio: rtrb::Producer<Box<Graph>>,
    from_audio: rtrb::Consumer<Box<Graph>>,
}

impl Controller {
    /// The sample rate modules are created for.
    #[must_use]
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// The largest block the engine runs in one go.
    #[must_use]
    pub fn max_block(&self) -> usize {
        self.max_block
    }

    /// The live params and meters of `node`, created on first use from the
    /// node's saved params. `None` for an unknown kind. A uid reused for a
    /// different kind gets fresh state.
    pub fn shared(&mut self, node: &PatchNode) -> Option<Arc<Shared>> {
        if let Some((kind, shared)) = self.shared.get(&node.uid)
            && *kind == node.kind
        {
            return Some(Arc::clone(shared));
        }
        let descriptor = (self.registry)(&node.kind)?;
        let shared = Arc::new(Shared::new(descriptor));
        for (id, &value) in &node.params {
            if let Some(index) = shared.params.index_of(id) {
                shared.params.set(index, value);
            }
        }
        self.shared
            .insert(node.uid, (descriptor.kind, Arc::clone(&shared)));
        Some(shared)
    }

    /// Forgets every instance, so the next [`apply`] builds fresh modules
    /// and loads params from the patch (used when loading a saved patch).
    ///
    /// [`apply`]: Controller::apply
    pub fn forget_instances(&mut self) {
        self.shared.clear();
        self.installed.clear();
    }

    /// Compiles `patch` and sends it to the engine. Surviving modules keep
    /// their state; new ones are created here, off the audio thread.
    pub fn apply(&mut self, patch: &Patch) -> Result<Plan, ApplyError> {
        self.collect();
        let plan = compile(patch, self.registry)?;
        if self.to_audio.is_full() {
            return Err(ApplyError::QueueFull);
        }
        for node in &patch.nodes {
            let _ = self.shared(node);
        }
        let installed = &self.installed;
        let shared = &self.shared;
        let (sample_rate, max_block) = (self.sample_rate, self.max_block);
        let graph = Graph::build(&plan, max_block, |uid| {
            let node_shared = Arc::clone(&shared[&uid].1);
            let descriptor = plan
                .nodes
                .iter()
                .find(|node| node.uid == uid)
                .map(|node| node.descriptor);
            let module = match descriptor {
                // Same uid and kind: the engine moves the running module over.
                Some(descriptor) if installed.get(&uid) == Some(&descriptor.kind) => None,
                Some(descriptor) => Some((descriptor.create)(sample_rate, max_block)),
                None => None,
            };
            (module, node_shared)
        });
        if self.to_audio.push(Box::new(graph)).is_err() {
            return Err(ApplyError::QueueFull);
        }
        self.installed = plan
            .nodes
            .iter()
            .map(|node| (node.uid, node.descriptor.kind))
            .collect();
        let kept: HashSet<u64> = patch.nodes.iter().map(|node| node.uid).collect();
        self.shared.retain(|uid, _| kept.contains(uid));
        Ok(plan)
    }

    /// Drops graphs the engine has finished with.
    pub fn collect(&mut self) {
        while self.from_audio.pop().is_ok() {}
    }

    /// Writes the live param values back into `patch` for saving.
    pub fn capture(&self, patch: &mut Patch) {
        for node in &mut patch.nodes {
            if let Some((_, shared)) = self.shared.get(&node.uid) {
                for (index, spec) in shared.params.specs().iter().enumerate() {
                    node.params
                        .insert(spec.id.to_owned(), shared.params.get(index));
                }
            }
        }
    }
}

/// The audio side. Call [`Engine::process`] from the audio callback; it
/// never allocates, frees, or locks.
pub struct Engine {
    graph: Box<Graph>,
    from_ui: rtrb::Consumer<Box<Graph>>,
    to_ui: rtrb::Producer<Box<Graph>>,
    /// An old graph the return queue had no room for yet.
    pending: Option<Box<Graph>>,
    max_block: usize,
}

impl Engine {
    /// Renders `left.len().min(right.len())` samples, overwriting both.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.install();
        let len = left.len().min(right.len());
        left[..len].fill(0.0);
        right[..len].fill(0.0);
        let mut start = 0;
        while start < len {
            let end = (start + self.max_block).min(len);
            self.graph
                .run(&mut left[start..end], &mut right[start..end]);
            start = end;
        }
    }

    /// Installs the newest waiting graph at this block boundary.
    fn install(&mut self) {
        if let Some(old) = self.pending.take()
            && let Err(rtrb::PushError::Full(old)) = self.to_ui.push(old)
        {
            self.pending = Some(old);
            return;
        }
        while let Ok(mut next) = self.from_ui.pop() {
            next.relink(&mut self.graph);
            let old = std::mem::replace(&mut self.graph, next);
            if let Err(rtrb::PushError::Full(old)) = self.to_ui.push(old) {
                self.pending = Some(old);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toys::{self, registry};
    use crate::{Cable, PatchNode};

    fn render(engine: &mut Engine, len: usize) -> (Vec<f32>, Vec<f32>) {
        let mut left = vec![0.0; len];
        let mut right = vec![0.0; len];
        engine.process(&mut left, &mut right);
        (left, right)
    }

    fn counter_patch() -> Patch {
        Patch {
            nodes: vec![
                PatchNode::new(1, toys::COUNTER, (0, 0)),
                PatchNode::new(2, toys::SINK, (1, 0)),
            ],
            cables: vec![Cable::new(1, "out", 2, "l")],
        }
    }

    #[test]
    fn a_chain_matches_direct_chaining() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let mut patch = counter_patch();
        patch.nodes.push(PatchNode::new(3, toys::GAIN, (0, 1)));
        patch.nodes.push(PatchNode::new(4, toys::GAIN, (0, 2)));
        patch.cables = vec![
            Cable::new(1, "out", 3, "in"),
            Cable::new(3, "out", 4, "in"),
            Cable::new(4, "out", 2, "l"),
        ];
        controller.apply(&patch).unwrap();
        let gain = controller.shared(&patch.nodes[2]).unwrap();
        gain.params.set(0, 3.0);
        let (left, right) = render(&mut engine, 100);
        for (index, (&l, &r)) in left.iter().zip(&right).enumerate() {
            let expected = index as f32 * 3.0 * 0.5;
            assert_eq!(l, expected, "sample {index}");
            assert_eq!(r, expected, "R mirrors L when unconnected");
        }
    }

    #[test]
    fn fan_in_sums() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let patch = Patch {
            nodes: vec![
                PatchNode::new(1, toys::CONST, (0, 0)),
                PatchNode::new(2, toys::CONST, (0, 1)),
                PatchNode::new(3, toys::SINK, (1, 0)),
            ],
            cables: vec![Cable::new(1, "out", 3, "l"), Cable::new(2, "out", 3, "l")],
        };
        controller.apply(&patch).unwrap();
        let (left, _) = render(&mut engine, 16);
        assert!(left.iter().all(|&sample| sample == 1.0), "{left:?}");
    }

    #[test]
    fn culled_nodes_do_not_run() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let mut patch = counter_patch();
        patch.nodes.push(PatchNode::new(9, toys::CONST, (3, 3)));
        controller.apply(&patch).unwrap();
        render(&mut engine, 64);
        let lonely = controller.shared(&patch.nodes[2]).unwrap();
        let counter = controller.shared(&patch.nodes[0]).unwrap();
        assert_eq!(lonely.meters.get(0), 0.0);
        assert_eq!(counter.meters.get(0), 1.0);
    }

    #[test]
    fn a_repatch_keeps_surviving_module_state() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let mut patch = counter_patch();
        controller.apply(&patch).unwrap();
        render(&mut engine, 10);
        patch.nodes.push(PatchNode::new(3, toys::GAIN, (0, 1)));
        patch.cables = vec![Cable::new(1, "out", 3, "in"), Cable::new(3, "out", 2, "l")];
        controller.apply(&patch).unwrap();
        let (left, _) = render(&mut engine, 4);
        assert_eq!(left, [5.0, 5.5, 6.0, 6.5]);
    }

    #[test]
    fn a_uid_reused_for_another_kind_gets_a_fresh_module() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let mut patch = counter_patch();
        controller.apply(&patch).unwrap();
        render(&mut engine, 10);
        patch.nodes[0].kind = toys::CONST.to_owned();
        controller.apply(&patch).unwrap();
        let (left, _) = render(&mut engine, 2);
        assert_eq!(left, [0.5, 0.5]);
        assert_eq!(controller.shared(&patch.nodes[0]).unwrap().params.len(), 1);
    }

    #[test]
    fn forgetting_instances_restarts_state_and_reloads_params() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let patch = counter_patch();
        controller.apply(&patch).unwrap();
        render(&mut engine, 10);
        controller.forget_instances();
        controller.apply(&patch).unwrap();
        let (left, _) = render(&mut engine, 2);
        assert_eq!(left, [0.0, 1.0]);
    }

    #[test]
    fn bypassed_modules_pass_input_through() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let mut patch = counter_patch();
        patch.nodes.push(PatchNode::new(3, toys::GAIN, (0, 1)));
        patch.nodes[2].powered = false;
        patch.cables = vec![Cable::new(1, "out", 3, "in"), Cable::new(3, "out", 2, "l")];
        controller.apply(&patch).unwrap();
        let (left, _) = render(&mut engine, 3);
        assert_eq!(left, [0.0, 1.0, 2.0]);
    }

    #[test]
    fn non_finite_output_stays_finite() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let patch = Patch {
            nodes: vec![
                PatchNode::new(1, toys::NAN, (0, 0)),
                PatchNode::new(2, toys::SINK, (1, 0)),
            ],
            cables: vec![Cable::new(1, "out", 2, "l")],
        };
        controller.apply(&patch).unwrap();
        let (left, right) = render(&mut engine, 32);
        assert!(left.iter().chain(&right).all(|sample| sample.is_finite()));
    }

    #[test]
    fn blocks_above_max_block_are_chunked() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 16);
        controller.apply(&counter_patch()).unwrap();
        let (left, _) = render(&mut engine, 50);
        assert_eq!(left[49], 49.0);
    }

    #[test]
    fn saved_params_load_and_capture() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        let mut patch = Patch {
            nodes: vec![
                PatchNode::new(1, toys::CONST, (0, 0)),
                PatchNode::new(2, toys::SINK, (1, 0)),
            ],
            cables: vec![Cable::new(1, "out", 2, "l")],
        };
        patch.nodes[0].params.insert("value".into(), 0.25);
        patch.nodes[0].params.insert("gone".into(), 9.0);
        controller.apply(&patch).unwrap();
        let (left, _) = render(&mut engine, 4);
        assert_eq!(left, [0.25; 4]);
        controller
            .shared(&patch.nodes[0])
            .unwrap()
            .params
            .set(0, -1.0);
        controller.capture(&mut patch);
        assert_eq!(patch.nodes[0].params["value"], -1.0);
    }

    #[test]
    fn old_graphs_come_back_to_the_ui() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 64);
        for _ in 0..(QUEUE * 3) {
            controller.apply(&counter_patch()).unwrap();
            render(&mut engine, 8);
        }
        let (left, _) = render(&mut engine, 1);
        assert_eq!(left, [(QUEUE * 3 * 8) as f32]);
    }

    #[cfg(feature = "rt-audit")]
    #[test]
    fn process_and_swaps_allocate_nothing() {
        let (mut controller, mut engine) = new_engine(registry, 48_000.0, 256);
        let mut patch = counter_patch();
        patch.nodes.push(PatchNode::new(3, toys::GAIN, (0, 1)));
        patch.nodes.push(PatchNode::new(4, toys::CONST, (0, 2)));
        patch.cables = vec![
            Cable::new(1, "out", 3, "in"),
            Cable::new(3, "out", 2, "l"),
            Cable::new(4, "out", 2, "l"),
        ];
        let mut left = vec![0.0; 1024];
        let mut right = vec![0.0; 1024];
        for round in 0..4 {
            patch.nodes[2].powered = round % 2 == 0;
            controller.apply(&patch).unwrap();
            let (_, allocations) = musictools_core::rt_audit::audit(|| {
                engine.process(&mut left, &mut right);
                engine.process(&mut left[..100], &mut right[..100]);
            });
            assert_eq!(allocations, 0, "round {round}");
        }
    }
}
