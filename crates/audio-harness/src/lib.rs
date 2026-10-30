//! Checked test harness for nice-plug audio processing — the successor to
//! truce's `AudioBuffer::from_slices_checked` and its test `ProcessContext`.
//!
//! The `unsafe` blocks the nice-plug `Buffer` API needs live here and only
//! here: a `Buffer` view is built once over leaked, never-reallocated storage
//! and reused for every process call, so steady-state processing allocates
//! nothing and allocation audits are honest. All sample access goes through
//! that view, keeping the contained borrow sound. Every other workspace crate
//! keeps `unsafe_code = "forbid"`.

use nice_plug::params::InternalParamMut;
use nice_plug::prelude::*;

/// Move `param` to `plain` the way host automation does: the value changes
/// and its smoother retargets, gliding there over the param's smoothing time
/// at `sample_rate` (as the wrappers do on a host parameter change).
///
/// Tests use this to change parameters between process calls; nice-plug only
/// exposes the setter to its own wrappers, behind `unsafe`.
pub fn automate<T: InternalParamMut>(param: &T, plain: T::Plain, sample_rate: f32) {
    // SAFETY: the setters are `unsafe` only to keep plugin code from calling
    // them by accident (nice-plug's docs); setting a value and retargeting its
    // smoother is exactly the wrapper's host-automation path, and a test owns
    // the params with no audio thread running concurrently.
    unsafe {
        param._internal_set_plain_value(plain);
        param._internal_update_smoother(sample_rate, false);
    }
}

/// Minimal `ProcessContext` for unit tests: an empty transport, no events,
/// and no-op host callbacks.
pub struct StubContext<P: Plugin> {
    transport: Transport,
    marker: std::marker::PhantomData<fn() -> P>,
}

impl<P: Plugin> StubContext<P> {
    #[must_use]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            transport: Transport::new(sample_rate),
            marker: std::marker::PhantomData,
        }
    }
}

impl<P: Plugin> ProcessContext<P> for StubContext<P> {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Standalone
    }

    fn execute_background(&self, _task: P::BackgroundTask) {}

    fn execute_gui(&self, _task: P::BackgroundTask) {}

    fn transport(&self) -> &Transport {
        &self.transport
    }

    fn next_event(&mut self) -> Option<PluginNoteEvent<P>> {
        None
    }

    fn try_send_event(
        &mut self,
        event: PluginNoteEvent<P>,
    ) -> Result<
        (),
        (
            PluginNoteEvent<P>,
            nice_plug::context::process::SendEventError,
        ),
    > {
        Err((
            event,
            nice_plug::context::process::SendEventError::NoOutputBuffer,
        ))
    }

    fn set_latency_samples(&self, _samples: u32) {}

    fn request_restart(&self) {}

    fn set_current_voice_capacity(&self, _capacity: u32) {}
}

/// Owns the audio buffers and hands the plugin a reused `Buffer` view of
/// them, with inputs written in first — mirroring how hosts deliver buffers
/// (the input is pre-copied onto the output).
pub struct Harness<P: Plugin> {
    num_samples: usize,
    context: StubContext<P>,
    /// The process-call view over the leaked storage. Declared last so it is
    /// dropped before the context on harness drop.
    buffer: Buffer<'static>,
}

impl<P: Plugin> Harness<P> {
    #[must_use]
    pub fn new(channels: usize, num_samples: usize) -> Self {
        // The storage must outlive the buffer view; leak it on purpose (a few
        // KiB per harness instance, test-only).
        let buffers: &'static mut Vec<Vec<f32>> =
            Box::leak(Box::new(vec![vec![0.0; num_samples]; channels]));
        let mut buffer = Buffer::default();
        // SAFETY: every channel slice is exactly `num_samples` long, the
        // storage is leaked (never freed, never reallocated), and the slices
        // are only ever reached through `buffer`.
        unsafe {
            buffer.set_slices(num_samples, |slices| {
                *slices = buffers.iter_mut().map(Vec::as_mut_slice).collect();
            });
        }
        Self {
            num_samples,
            context: StubContext::new(48_000.0),
            buffer,
        }
    }

    /// Replace a channel's samples, which serve as the process input.
    pub fn set_input(&mut self, channel: usize, samples: &[f32]) {
        assert!(channel < self.channels(), "no such channel {channel}");
        assert_eq!(
            samples.len(),
            self.num_samples,
            "input length must match the harness block size"
        );
        self.buffer.as_slice()[channel].copy_from_slice(samples);
    }

    /// Run one process call over the reused buffer view. Allocation-free in
    /// steady state.
    pub fn process(&mut self, plugin: &mut P) -> ProcessStatus {
        let mut aux = AuxiliaryBuffers {
            inputs: &mut [],
            outputs: &mut [],
        };
        let Self {
            buffer, context, ..
        } = self;
        plugin.process(buffer, &mut aux, context)
    }

    /// A channel's samples after processing.
    #[must_use]
    pub fn output(&self, channel: usize) -> &[f32] {
        self.buffer.as_slice_immutable()[channel]
    }

    /// Number of channels in the harness.
    #[must_use]
    pub fn channels(&self) -> usize {
        self.buffer.channels()
    }

    /// Block size of the harness.
    #[must_use]
    pub fn num_samples(&self) -> usize {
        self.num_samples
    }
}
