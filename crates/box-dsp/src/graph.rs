//! The installed graph and its allocation-free executor.

use std::sync::Arc;

use box_core::{BlockIo, MAX_PORTS, Module, ParamValues, Shared};
use musictools_core::finite_or;

use crate::{Plan, Step};

/// One module instance in run order.
pub(crate) struct Slot {
    pub uid: u64,
    /// `None` while waiting to be relinked from the previous graph.
    pub module: Option<Box<dyn Module>>,
    pub shared: Arc<Shared>,
    /// This block's param snapshot.
    pub values: Vec<f32>,
    pub powered: bool,
    pub inputs: [usize; MAX_PORTS],
    pub outputs: [usize; MAX_PORTS],
    pub input_count: usize,
    pub output_count: usize,
    pub connected: u32,
}

/// A compiled patch with its modules and preallocated buffers.
pub(crate) struct Graph {
    pub slots: Vec<Slot>,
    pub steps: Vec<Step>,
    pub sum_sources: Vec<usize>,
    pub buffers: Vec<Vec<f32>>,
    pub sinks: Vec<[usize; 2]>,
}

impl Graph {
    /// A graph that plays silence.
    pub fn empty() -> Self {
        Self {
            slots: Vec::new(),
            steps: Vec::new(),
            sum_sources: Vec::new(),
            buffers: Vec::new(),
            sinks: Vec::new(),
        }
    }

    /// Builds a graph from a plan; `module_for` returns a fresh module or
    /// `None` for one that survives from the installed graph.
    pub fn build(
        plan: &Plan,
        max_block: usize,
        mut parts: impl FnMut(u64) -> (Option<Box<dyn Module>>, Arc<Shared>),
    ) -> Self {
        let slots = plan
            .nodes
            .iter()
            .map(|node| {
                let (module, shared) = parts(node.uid);
                Slot {
                    uid: node.uid,
                    module,
                    values: vec![0.0; shared.params.len()],
                    shared,
                    powered: node.powered,
                    inputs: node.inputs,
                    outputs: node.outputs,
                    input_count: node.descriptor.inputs.len(),
                    output_count: node.descriptor.outputs.len(),
                    connected: node.connected,
                }
            })
            .collect();
        Self {
            slots,
            steps: plan.steps.clone(),
            sum_sources: plan.sum_sources.clone(),
            buffers: vec![vec![0.0; max_block]; plan.buffer_count],
            sinks: plan.sinks.clone(),
        }
    }

    /// Moves modules for `None` slots out of `old`, matched by uid.
    pub fn relink(&mut self, old: &mut Self) {
        for slot in self.slots.iter_mut().filter(|slot| slot.module.is_none()) {
            if let Some(previous) = old
                .slots
                .iter_mut()
                .find(|previous| previous.uid == slot.uid)
            {
                slot.module = previous.module.take();
            }
        }
    }

    /// Runs one block of `len <= max_block` samples and adds the sinks into
    /// `left` and `right` (which the caller zeroes).
    pub fn run(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());
        let Self {
            slots,
            steps,
            sum_sources,
            buffers,
            sinks,
        } = self;
        if buffers.first().is_some_and(|zero| zero.len() < len) {
            return;
        }
        for step in steps.iter() {
            match step {
                Step::Sum { into, sources } => {
                    let mut target = std::mem::take(&mut buffers[*into]);
                    target[..len].fill(0.0);
                    for &source in &sum_sources[sources.clone()] {
                        for (sum, &sample) in target[..len].iter_mut().zip(&buffers[source][..len])
                        {
                            *sum += sample;
                        }
                    }
                    buffers[*into] = target;
                }
                Step::Run(index) => run_slot(&mut slots[*index], buffers, len),
            }
        }
        for [l, r] in sinks.iter() {
            for (out, &sample) in left[..len].iter_mut().zip(&buffers[*l][..len]) {
                *out += finite_or(sample, 0.0);
            }
            for (out, &sample) in right[..len].iter_mut().zip(&buffers[*r][..len]) {
                *out += finite_or(sample, 0.0);
            }
        }
    }
}

fn run_slot(slot: &mut Slot, buffers: &mut [Vec<f32>], len: usize) {
    // Outputs move out of the pool for the call; inputs stay borrowed from
    // it. The compiler guarantees the two never share a buffer.
    let mut outputs: [Vec<f32>; MAX_PORTS] = Default::default();
    for (port, owned) in outputs.iter_mut().enumerate().take(slot.output_count) {
        *owned = std::mem::take(&mut buffers[slot.outputs[port]]);
    }
    {
        let mut inputs: [&[f32]; MAX_PORTS] = [&[]; MAX_PORTS];
        for (port, input) in inputs.iter_mut().enumerate().take(slot.input_count) {
            *input = &buffers[slot.inputs[port]][..len];
        }
        let mut slices = outputs.each_mut().map(|buffer| {
            let end = len.min(buffer.len());
            &mut buffer[..end]
        });
        match slot.module.as_mut() {
            Some(module) if slot.powered => {
                slot.shared.params.snapshot_into(&mut slot.values);
                let mut io = BlockIo {
                    inputs: &inputs[..slot.input_count],
                    outputs: &mut slices[..slot.output_count],
                    len,
                    connected: slot.connected,
                };
                module.process(&mut io, &ParamValues(&slot.values), &slot.shared.meters);
            }
            // Bypassed (or not relinked): inputs pass through, extra
            // outputs stay silent.
            _ => {
                for (port, output) in slices.iter_mut().enumerate().take(slot.output_count) {
                    if port < slot.input_count {
                        output.copy_from_slice(inputs[port]);
                    } else {
                        output.fill(0.0);
                    }
                }
            }
        }
    }
    for (port, owned) in outputs.into_iter().enumerate().take(slot.output_count) {
        buffers[slot.outputs[port]] = owned;
    }
}
