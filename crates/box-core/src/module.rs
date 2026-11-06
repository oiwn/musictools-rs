//! The `Module` trait: block processing over mono port buffers.

use crate::{Meters, ParamValues};

/// One block's port buffers. `inputs[i]` and `outputs[i]` follow the
/// descriptor's port order; each slice is `len` samples long. An
/// unconnected input reads silence; [`BlockIo::is_connected`] tells them
/// apart.
pub struct BlockIo<'a, 'b> {
    pub inputs: &'a [&'b [f32]],
    pub outputs: &'a mut [&'b mut [f32]],
    pub len: usize,
    /// Bit `i` set when input `i` has at least one wire.
    pub connected: u32,
}

impl BlockIo<'_, '_> {
    /// True when input `index` has at least one wire.
    #[must_use]
    pub fn is_connected(&self, index: usize) -> bool {
        index < 32 && self.connected & (1 << index) != 0
    }
}

/// A module instance's DSP. Lives on the audio thread once installed.
///
/// `process` and `clear` run on the audio thread: no allocation, no locks,
/// no I/O. `reset` runs off the audio thread before installation and may
/// allocate.
pub trait Module: Send {
    /// Prepares for a sample rate and the largest block `process` will see.
    fn reset(&mut self, sample_rate: f32, max_block: usize);

    /// Drops all signal state (envelopes, phases, delay lines); realtime-safe.
    fn clear(&mut self);

    /// Samples of output still ringing after the input goes silent.
    fn tail(&self) -> usize {
        0
    }

    /// Processes one block. Write every output sample; inputs may hold any
    /// finite value.
    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters);
}
