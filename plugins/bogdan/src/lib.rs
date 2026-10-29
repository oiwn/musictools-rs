//! nice-plug wrapper for the Bogdan detail-preserving clipper.

use std::num::NonZeroU32;
use std::sync::Arc;

use bogdan_dsp::{ClipMode, DetailClipper, DetailSettings, FoldShape};
use musictools_core::scope_tap::ScopeTap;
use nice_plug::prelude::*;

mod editor;

/// Scope history the tap keeps: matches the editor's frame history.
const SCOPE_TAP_CAPACITY: usize = 2 * 8_192;

/// Peak-reshaping voice, mirrors [`bogdan_dsp::ClipMode`] on the framework side.
#[derive(Enum, Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Clean,
    Detail,
    Fold,
}

impl Mode {
    fn to_dsp(self) -> ClipMode {
        match self {
            Mode::Clean => ClipMode::Clean,
            Mode::Detail => ClipMode::Detail,
            Mode::Fold => ClipMode::Fold,
        }
    }
}

/// Wavefolder transfer function, mirrors [`bogdan_dsp::FoldShape`].
#[derive(Enum, Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Sine,
    Triangle,
}

impl Shape {
    fn to_dsp(self) -> FoldShape {
        match self {
            Shape::Sine => FoldShape::Sine,
            Shape::Triangle => FoldShape::Triangle,
        }
    }
}

#[derive(Params)]
pub struct BogdanParams {
    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "ceiling"]
    pub ceiling: FloatParam,

    #[id = "mode"]
    pub mode: EnumParam<Mode>,

    #[id = "detail"]
    pub detail: FloatParam,

    #[id = "amount"]
    pub amount: FloatParam,

    #[id = "shape"]
    pub shape: EnumParam<Shape>,

    /// Interleaved `[driven, processed]` frames for the oscilloscope.
    pub scope_tap: Arc<ScopeTap<f32>>,
}

impl Default for BogdanParams {
    fn default() -> Self {
        let format_hz = |value: f32| musictools_core::format_hz(f64::from(value));
        Self {
            drive: FloatParam::new(
                "Drive",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_smoother(SmoothingStyle::Exponential(5.0))
            .with_unit(" dB")
            .with_step_size(0.01),
            ceiling: FloatParam::new(
                "Ceiling",
                -0.1,
                FloatRange::Linear {
                    min: -24.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Exponential(5.0))
            .with_unit(" dB")
            .with_step_size(0.01),
            mode: EnumParam::new("Mode", Mode::Detail),
            // The truce range was `log(20, 2000)`. `Skewed` normalizes as
            // ((v-min)/(max-min)).powf(factor), so factor 0.232 puts the
            // 1000 Hz default at ~85% of knob travel (the log feel).
            detail: FloatParam::new(
                "Detail",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 2000.0,
                    factor: 0.232,
                },
            )
            .with_smoother(SmoothingStyle::Exponential(20.0))
            .with_unit(" Hz")
            .with_value_to_string(Arc::new(format_hz))
            .with_step_size(0.01),
            amount: FloatParam::new("Amount", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Exponential(5.0))
                .with_unit(" %")
                .with_step_size(0.01),
            shape: EnumParam::new("Shape", Shape::Sine),
            scope_tap: Arc::new(ScopeTap::new(SCOPE_TAP_CAPACITY)),
        }
    }
}

/// Per-instance filter state for Bogdan's supported mono/stereo layouts.
#[derive(Default)]
pub struct BogdanState {
    channels: [DetailClipper; 2],
    scope_transfer: Vec<f32>,
    sample_rate: f64,
}

impl BogdanState {
    fn reset(&mut self, sample_rate: f64, max_block_size: usize) {
        self.sample_rate = sample_rate;
        for channel in &mut self.channels {
            channel.reset(sample_rate);
        }
        self.scope_transfer.resize(max_block_size * 2, 0.0);
    }

    fn reset_channels(&mut self) {
        for channel in &mut self.channels {
            channel.reset(self.sample_rate);
        }
    }
}

/// dB to linear gain, sanitized.
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(musictools_core::finite_or(db, 0.0) / 20.0)
}

/// Bogdan, the detail-preserving clipper and wavefolder.
pub struct Bogdan {
    params: Arc<BogdanParams>,
    state: BogdanState,
}

impl Default for Bogdan {
    fn default() -> Self {
        Self {
            params: Arc::new(BogdanParams::default()),
            state: BogdanState::default(),
        }
    }
}

impl Bogdan {
    /// Size buffers, reset filters, and snap smoothers to their current
    /// values (the wrapper does the smoother snap on activation; tests call
    /// this directly instead).
    fn configure(&mut self, sample_rate: f64, max_block_size: usize) {
        self.state.reset(sample_rate, max_block_size);
        self.params.scope_tap.clear();
        let params = &self.params;
        params.drive.smoothed.reset(params.drive.value());
        params.ceiling.smoothed.reset(params.ceiling.value());
        params.detail.smoothed.reset(params.detail.value());
        params.amount.smoothed.reset(params.amount.value());
    }
}

impl Plugin for Bogdan {
    const NAME: &'static str = "ppc_bogdan";
    const VENDOR: &'static str = musictools_core::VENDOR_NAME;
    const URL: &'static str = "https://github.com/oiwn/musictools-rs";
    const EMAIL: &'static str = "oiwn@localhost";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    type Editor = nice_plug_egui::EguiEditor<editor::BogdanEditor>;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.configure(
            f64::from(buffer_config.sample_rate),
            buffer_config.max_buffer_size as usize,
        );
        true
    }

    fn reset(&mut self) {
        self.state.reset_channels();
        self.params.scope_tap.clear();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let num_samples = buffer.samples();
        debug_assert!(self.state.scope_transfer.len() >= num_samples * 2);

        for (sample_index, mut samples) in buffer.iter_samples().enumerate() {
            // Advance every smoothed parameter exactly once per frame.
            let settings = DetailSettings {
                drive: db_to_linear(self.params.drive.smoothed.next()),
                ceiling: db_to_linear(self.params.ceiling.smoothed.next()),
                mode: self.params.mode.value().to_dsp(),
                detail_hz: musictools_core::finite_or(self.params.detail.smoothed.next(), 20.0),
                amount: musictools_core::finite_or(self.params.amount.smoothed.next(), 0.0),
                shape: self.params.shape.value().to_dsp(),
            };

            debug_assert!(samples.len() <= 2);
            for (channel, sample) in samples.iter_mut().enumerate() {
                let frame = self.state.channels[channel].process(*sample, settings);
                if channel == 0 {
                    let scope_index = sample_index * 2;
                    self.state.scope_transfer[scope_index] = frame.driven;
                    self.state.scope_transfer[scope_index + 1] = frame.output;
                }
                *sample = frame.output;
            }
        }

        if buffer.channels() > 0 {
            self.params
                .scope_tap
                .push_frames(&self.state.scope_transfer[..num_samples * 2]);
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        editor::create(self.params.clone())
    }
}

impl ClapPlugin for Bogdan {
    const CLAP_ID: &'static str = "com.oiwn.ppc_bogdan";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Detail-preserving clipper and wavefolder");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Bogdan {
    const VST3_CLASS_ID: [u8; 16] = *b"ppc_bogdan_00001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nice_export_clap!(Bogdan);
nice_export_vst3!(Bogdan);

#[cfg(test)]
mod tests {
    use super::*;
    use audio_harness::Harness;

    #[test]
    fn process_taps_driven_and_processed_channel_zero_frames() {
        let mut plugin = Bogdan::default();
        plugin.configure(48_000.0, 4);

        let left_input = [0.25_f32, 1.25, -1.5, 0.5];
        let mut harness = Harness::new(2, 4);
        harness.set_input(0, &left_input);
        harness.set_input(1, &[-0.25, -1.25, 1.5, -0.5]);

        assert_eq!(harness.process(&mut plugin), ProcessStatus::Normal);

        let tapped = plugin.params.scope_tap.drain();
        assert_eq!(tapped.len(), left_input.len() * 2);
        for (index, pair) in tapped.as_chunks::<2>().0.iter().enumerate() {
            assert_eq!(pair[0], left_input[index], "driven frame {index}");
            assert_eq!(pair[1], harness.output(0)[index], "processed frame {index}");
        }
    }

    #[cfg(feature = "rt-audit")]
    #[test]
    fn process_scope_feed_is_allocation_free_even_when_tap_is_full() {
        const BLOCK_SIZE: usize = 256;
        const BLOCKS_TO_OVERFLOW_TAP: usize = 140;

        let mut plugin = Bogdan::default();
        plugin.configure(48_000.0, BLOCK_SIZE);

        let mut harness = Harness::new(1, BLOCK_SIZE);
        harness.set_input(0, &[1.5; BLOCK_SIZE]);
        harness.process(&mut plugin);

        let (_, allocations) = musictools_core::rt_audit::audit(|| {
            for _ in 0..BLOCKS_TO_OVERFLOW_TAP {
                let _ = harness.process(&mut plugin);
            }
        });

        assert_eq!(allocations, 0);
    }
}
