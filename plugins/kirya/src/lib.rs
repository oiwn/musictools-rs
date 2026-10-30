//! nice-plug wrapper for the Kirya plate reverb.

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use kirya_dsp::analysis::IrRender;
use kirya_dsp::{KiryaReverb, KiryaSettings};
use musictools_core::LogTaper;
use nice_plug::prelude::*;

mod editor;
mod task;

pub use task::RenderIr;

/// Taper shared by the four filter cutoffs.
const CUTOFF_TAPER: LogTaper = LogTaper::new(20.0, 20_000.0);
/// Taper of the modulation rate.
const MOD_RATE_TAPER: LogTaper = LogTaper::new(0.01, 20.0);

/// Ids are a permanent contract: a host stores automation against them, so an
/// id must never be renamed or reused.
///
/// The five log-tapered params (the cutoffs and Mod Rate) store their knob
/// position `0..=1`, not hertz: nice-plug has no log range, and a power-curve
/// range over 1000:1 squashes the low end. [`LogTaper`] maps position to
/// hertz exactly as truce's `log(min, max)` did, and smoothing the position
/// exponentially is a constant-ratio glide in hertz.
#[derive(Params)]
pub struct KiryaParams {
    #[id = "dry"]
    pub dry: FloatParam,

    #[id = "wet"]
    pub wet: FloatParam,

    /// Pre-delay and Size retune delay lengths, so both take the slow 50 ms
    /// glide: it avoids zipper noise and turns a knob sweep into a musical
    /// tape-style pitch slide.
    #[id = "pre_delay"]
    pub pre_delay: FloatParam,

    #[id = "size"]
    pub size: FloatParam,

    #[id = "diffusion"]
    pub diffusion: FloatParam,

    #[id = "decay"]
    pub decay: FloatParam,

    #[id = "input_low_cut"]
    pub input_low_cut: FloatParam,

    #[id = "input_high_cut"]
    pub input_high_cut: FloatParam,

    #[id = "reverb_low_cut"]
    pub reverb_low_cut: FloatParam,

    #[id = "reverb_high_cut"]
    pub reverb_high_cut: FloatParam,

    #[id = "mod_rate"]
    pub mod_rate: FloatParam,

    #[id = "mod_depth"]
    pub mod_depth: FloatParam,

    /// Modulation waveform: 0% is a triangle, 100% a sine.
    #[id = "mod_shape"]
    pub mod_shape: FloatParam,

    /// Hold the tank indefinitely. The input stays live, so new material can
    /// be layered into a frozen tail.
    #[id = "freeze"]
    pub freeze: BoolParam,

    /// Latest finished impulse-response render, handed from the background
    /// task to the editor. `process` never touches it, so a plain `Mutex` is
    /// safe here — no audio-thread lock is ever taken.
    pub ir_slot: Arc<Mutex<Option<Box<IrRender>>>>,

    /// Bumped every time `ir_slot` is replaced, so the editor can tell a new
    /// render from the one it already uploaded.
    pub ir_generation: Arc<AtomicU64>,

    /// Id of the newest render request. nice-plug queues every task it is
    /// given, so a render whose id is no longer the newest skips itself (see
    /// [`RenderIr::run`]).
    pub ir_requested: Arc<AtomicU64>,

    /// Host sample rate as `f64` bits, published on activation.
    ///
    /// The editor context does not carry the rate, and the IR probe has to
    /// render at whatever the host is playing or the displayed reverb time
    /// would be wrong at anything but 48 kHz. Zero until the first activation.
    pub sample_rate_bits: Arc<AtomicU64>,
}

/// A `0..1` amount shown and typed as a whole percentage.
fn percent(name: &str, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_smoother(SmoothingStyle::Exponential(5.0))
        .with_unit(" %")
        .with_value_to_string(Arc::new(musictools_core::format_percent))
        .with_string_to_value(Arc::new(musictools_core::parse_percent))
        .with_step_size(0.01)
}

/// A frequency stored as its knob position on `taper`, shown and typed in Hz.
///
/// `format_hz` prints its own unit, so the param carries none.
fn log_hz(name: &str, default_hz: f32, taper: LogTaper) -> FloatParam {
    FloatParam::new(
        name,
        taper.to_position(default_hz),
        FloatRange::Linear { min: 0.0, max: 1.0 },
    )
    .with_smoother(SmoothingStyle::Exponential(20.0))
    .with_value_to_string(Arc::new(move |position| {
        musictools_core::format_hz(f64::from(taper.to_value(position)))
    }))
    .with_string_to_value(Arc::new(move |text| {
        musictools_core::parse_hz(text).map(|hz| taper.to_position(hz))
    }))
}

impl Default for KiryaParams {
    fn default() -> Self {
        Self {
            dry: percent("Dry", 1.0),
            wet: percent("Wet", 0.5),
            // Same `t^factor` taper as truce's `skewed(0, 500, 0.5)`.
            pre_delay: FloatParam::new(
                "Pre-Delay",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 500.0,
                    factor: 0.5,
                },
            )
            .with_smoother(SmoothingStyle::Exponential(50.0))
            .with_unit(" ms")
            .with_value_to_string(Arc::new(|value| format!("{value:.1}")))
            .with_step_size(0.1),
            size: FloatParam::new(
                "Size",
                1.0,
                FloatRange::Skewed {
                    min: 0.05,
                    max: 4.0,
                    factor: 0.4,
                },
            )
            .with_smoother(SmoothingStyle::Exponential(50.0))
            .with_value_to_string(Arc::new(|value| format!("{value:.2}")))
            .with_step_size(0.01),
            diffusion: percent("Diffusion", 1.0),
            decay: percent("Decay", 0.55),
            input_low_cut: log_hz("In Low Cut", 20.0, CUTOFF_TAPER),
            input_high_cut: log_hz("In High Cut", 20_000.0, CUTOFF_TAPER),
            reverb_low_cut: log_hz("Rev Low Cut", 20.0, CUTOFF_TAPER),
            reverb_high_cut: log_hz("Rev High Cut", 10_000.0, CUTOFF_TAPER),
            mod_rate: log_hz("Mod Rate", 1.0, MOD_RATE_TAPER),
            mod_depth: percent("Mod Depth", 0.5),
            mod_shape: percent("Mod Shape", 0.5),
            freeze: BoolParam::new("Freeze", false),
            ir_slot: Arc::new(Mutex::new(None)),
            ir_generation: Arc::new(AtomicU64::new(0)),
            ir_requested: Arc::new(AtomicU64::new(0)),
            sample_rate_bits: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl KiryaParams {
    /// Every float param, for bulk smoother work.
    fn floats(&self) -> [&FloatParam; 13] {
        [
            &self.dry,
            &self.wet,
            &self.pre_delay,
            &self.size,
            &self.diffusion,
            &self.decay,
            &self.input_low_cut,
            &self.input_high_cut,
            &self.reverb_low_cut,
            &self.reverb_high_cut,
            &self.mod_rate,
            &self.mod_depth,
            &self.mod_shape,
        ]
    }

    /// Settle every smoother on its target, as on activation.
    fn snap_smoothers(&self) {
        for param in self.floats() {
            param.smoothed.reset(param.value());
        }
    }

    /// DSP settings with each float read exactly once through `read`.
    fn build_settings(&self, read: impl Fn(&FloatParam) -> f32) -> KiryaSettings {
        KiryaSettings {
            dry: read(&self.dry),
            wet: read(&self.wet),
            pre_delay_ms: read(&self.pre_delay),
            size: read(&self.size),
            diffusion: read(&self.diffusion),
            decay: read(&self.decay),
            input_low_cut_hz: CUTOFF_TAPER.to_value(read(&self.input_low_cut)),
            input_high_cut_hz: CUTOFF_TAPER.to_value(read(&self.input_high_cut)),
            reverb_low_cut_hz: CUTOFF_TAPER.to_value(read(&self.reverb_low_cut)),
            reverb_high_cut_hz: CUTOFF_TAPER.to_value(read(&self.reverb_high_cut)),
            mod_rate_hz: MOD_RATE_TAPER.to_value(read(&self.mod_rate)),
            mod_depth: read(&self.mod_depth),
            mod_shape: read(&self.mod_shape),
            freeze: self.freeze.value(),
        }
    }

    /// Advance every smoother one sample and return the settings. Audio
    /// thread only.
    fn settings(&self) -> KiryaSettings {
        self.build_settings(|param| param.smoothed.next())
    }

    /// Read every parameter's target, advancing nothing.
    ///
    /// The smoothers belong to the audio thread; anything off it — the IR
    /// render task, the editor's change detection — must use this instead of
    /// [`Self::settings`], or it steals samples from `process`.
    pub(crate) fn target_settings(&self) -> KiryaSettings {
        self.build_settings(FloatParam::value)
    }
}

/// Kirya, the Dattorro plate reverb.
pub struct Kirya {
    params: Arc<KiryaParams>,
    reverb: KiryaReverb,
}

impl Default for Kirya {
    fn default() -> Self {
        Self {
            params: Arc::new(KiryaParams::default()),
            reverb: KiryaReverb::default(),
        }
    }
}

impl Kirya {
    /// Size the reverb for `sample_rate`, publish the rate for the editor's
    /// IR probe, and snap smoothers to their targets (the wrapper does the
    /// snap on activation too; tests call this directly).
    fn configure(&mut self, sample_rate: f64) {
        self.reverb.reset(sample_rate);
        self.params
            .sample_rate_bits
            .store(sample_rate.to_bits(), Ordering::Relaxed);
        self.params.snap_smoothers();
    }
}

impl Plugin for Kirya {
    const NAME: &'static str = "ppc_kirya";
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

    type Editor = nice_plug_egui::EguiEditor<editor::KiryaEditor>;
    type SysExMessage = ();
    type BackgroundTask = RenderIr;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        let params = self.params.clone();
        Box::new(move |task: RenderIr| task.run(&params))
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.configure(f64::from(buffer_config.sample_rate));
        true
    }

    /// Clear the tail without reallocating: this can run on the audio thread.
    fn reset(&mut self) {
        self.reverb.clear();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut frozen = false;
        for mut samples in buffer.iter_samples() {
            // One read of every parameter per frame, before the channel work:
            // the tank is mono, so reading per channel would advance each
            // smoother twice on a stereo track.
            let settings = self.params.settings();
            frozen = settings.freeze;

            let Some(&mut left) = samples.get_mut(0) else {
                continue;
            };
            let right = samples.get_mut(1).map_or(left, |right| *right);
            let frame = self.reverb.process(left, right, settings);

            if let Some(left) = samples.get_mut(0) {
                *left = frame.left;
            }
            if let Some(right) = samples.get_mut(1) {
                *right = frame.right;
            }
        }

        // A frozen tank never decays: keep the host from suspending the
        // plugin underneath it.
        if frozen {
            ProcessStatus::KeepAlive
        } else {
            ProcessStatus::Tail(self.reverb.tail_samples())
        }
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        editor::create(self.params.clone(), async_executor)
    }
}

impl ClapPlugin for Kirya {
    const CLAP_ID: &'static str = "com.oiwn.ppc_kirya";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Dattorro plate reverb");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Reverb,
        ClapFeature::Stereo,
    ];
}

#[cfg(feature = "vst3")]
impl Vst3Plugin for Kirya {
    const VST3_CLASS_ID: [u8; 16] = *b"ppc_kirya_000001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nice_export_clap!(Kirya);
#[cfg(feature = "vst3")]
nice_export_vst3!(Kirya);

#[cfg(test)]
mod tests {
    use super::*;
    use audio_harness::{Harness, automate};

    const SAMPLE_RATE: f64 = 48_000.0;
    #[allow(clippy::cast_possible_truncation)]
    const RATE: f32 = SAMPLE_RATE as f32;

    fn configured() -> Kirya {
        let mut plugin = Kirya::default();
        plugin.configure(SAMPLE_RATE);
        plugin
    }

    /// Run one stereo block of `input` (both channels) through `plugin`.
    fn run(plugin: &mut Kirya, harness: &mut Harness<Kirya>, input: &[f32]) -> ProcessStatus {
        for channel in 0..harness.channels() {
            harness.set_input(channel, input);
        }
        harness.process(plugin)
    }

    /// The display string of a param at a plain value.
    fn shown(param: &FloatParam, plain: f32) -> String {
        param.normalized_value_to_string(param.preview_normalized(plain), true)
    }

    #[test]
    fn a_dry_impulse_passes_through_and_a_wet_tail_follows_it() {
        // The block has to outrun the first output tap on each side. At 48 kHz
        // and Size 1.0 the earliest left tap lands near sample 429 and the
        // earliest right tap near 569 — that offset between the two is the
        // paper's stereo image, so a shorter block would show a silent right
        // channel and prove nothing.
        const BLOCK: usize = 2_048;

        let mut plugin = configured();
        let mut harness = Harness::new(2, BLOCK);
        let mut input = [0.0_f32; BLOCK];
        input[0] = 1.0;
        run(&mut plugin, &mut harness, &input);
        let (left, right) = (harness.output(0), harness.output(1));

        // Dry defaults to 100%, so the impulse itself survives.
        assert!((left[0] - 1.0).abs() < 0.05, "dry impulse was {}", left[0]);
        // ...and the tank answers with something in both channels.
        assert!(left[100..].iter().any(|&s| s.abs() > 1.0e-6));
        assert!(right[100..].iter().any(|&s| s.abs() > 1.0e-6));
        assert!(left.iter().chain(right.iter()).all(|s| s.is_finite()));
        // The two sides are genuinely different, not a duplicated mono tail.
        assert!(
            left.iter()
                .zip(right.iter())
                .any(|(l, r)| (l - r).abs() > 1.0e-3)
        );
    }

    #[test]
    fn readouts_match_the_truce_build() {
        // The strings the truce Kirya showed, so hosts and users see the same
        // values after the port. Percent params now print the unit once
        // with a space ("55 %"), like Bogdan.
        let params = KiryaParams::default();

        assert_eq!(
            shown(&params.mod_rate, MOD_RATE_TAPER.to_position(0.01)),
            "0.01 Hz"
        );
        assert_eq!(
            shown(&params.mod_rate, MOD_RATE_TAPER.to_position(1.0)),
            "1.00 Hz"
        );
        assert_eq!(
            shown(&params.mod_rate, MOD_RATE_TAPER.to_position(20.0)),
            "20.0 Hz"
        );
        for param in [
            &params.input_low_cut,
            &params.input_high_cut,
            &params.reverb_low_cut,
            &params.reverb_high_cut,
        ] {
            assert_eq!(shown(param, CUTOFF_TAPER.to_position(20.0)), "20.0 Hz");
            assert_eq!(shown(param, CUTOFF_TAPER.to_position(440.0)), "440 Hz");
            assert_eq!(
                shown(param, CUTOFF_TAPER.to_position(10_000.0)),
                "10.00 kHz"
            );
        }

        assert_eq!(shown(&params.size, 1.0), "1.00");
        assert_eq!(shown(&params.decay, 0.55), "55 %");
        assert_eq!(shown(&params.pre_delay, 0.0), "0.0 ms");
    }

    #[test]
    fn log_knobs_keep_the_exact_truce_taper() {
        // truce's `log(20, 20000)`: half travel is the geometric mean.
        let params = KiryaParams::default();
        let hz = |position: f32| CUTOFF_TAPER.to_value(position);
        assert!((hz(0.5) - 632.46).abs() < 0.05, "{}", hz(0.5));
        assert_eq!(
            params.input_low_cut.normalized_value_to_string(0.5, true),
            "632 Hz"
        );
        // Defaults land where truce put them.
        let settings = params.target_settings();
        assert!((settings.input_low_cut_hz - 20.0).abs() < 1.0e-3);
        assert!((settings.input_high_cut_hz - 20_000.0).abs() < 0.5);
        assert!((settings.reverb_high_cut_hz - 10_000.0).abs() < 0.5);
        assert!((settings.mod_rate_hz - 1.0).abs() < 1.0e-4);
    }

    #[test]
    fn typed_values_parse_back() {
        let params = KiryaParams::default();
        for (text, hz) in [("440", 440.0), ("440 Hz", 440.0), ("2.5 kHz", 2_500.0)] {
            let position = params
                .reverb_high_cut
                .string_to_normalized_value(text)
                .unwrap();
            let back = CUTOFF_TAPER.to_value(position);
            assert!((back - hz).abs() / hz < 1.0e-3, "{text} came back {back}");
        }
        let normalized = params.wet.string_to_normalized_value("25 %").unwrap();
        assert!((params.wet.preview_plain(normalized) - 0.25).abs() < 1.0e-6);
        let normalized = params.pre_delay.string_to_normalized_value("120").unwrap();
        assert!((params.pre_delay.preview_plain(normalized) - 120.0).abs() < 0.05);
    }

    #[test]
    fn a_mono_layout_is_processed_without_reading_a_second_channel() {
        let mut plugin = configured();
        let mut harness = Harness::new(1, 256);
        let mut input = [0.0_f32; 256];
        input[0] = 1.0;

        assert!(matches!(
            run(&mut plugin, &mut harness, &input),
            ProcessStatus::Tail(_)
        ));
        assert!(harness.output(0).iter().all(|s| s.is_finite()));
        assert!((harness.output(0)[0] - 1.0).abs() < 0.05);
    }

    #[test]
    fn non_finite_host_input_never_reaches_the_output() {
        let mut plugin = configured();
        let mut harness = Harness::new(2, 256);
        let input: Vec<f32> = (0..256)
            .map(|n| match n % 4 {
                0 => f32::NAN,
                1 => f32::INFINITY,
                2 => f32::NEG_INFINITY,
                _ => 0.5,
            })
            .collect();

        for _ in 0..8 {
            run(&mut plugin, &mut harness, &input);
            assert!(
                harness
                    .output(0)
                    .iter()
                    .chain(harness.output(1))
                    .all(|s| s.is_finite())
            );
        }
    }

    #[test]
    fn the_reported_tail_is_finite_while_running_and_kept_alive_while_frozen() {
        let mut plugin = configured();
        let mut harness = Harness::new(2, 256);
        let silence = [0.0_f32; 256];

        match run(&mut plugin, &mut harness, &silence) {
            ProcessStatus::Tail(samples) => {
                assert!(samples > 0, "a reverb with no tail is not a reverb");
            }
            other => panic!("expected a tail, got {other:?}"),
        }

        automate(&plugin.params.freeze, true, RATE);
        assert_eq!(
            run(&mut plugin, &mut harness, &silence),
            ProcessStatus::KeepAlive
        );

        automate(&plugin.params.freeze, false, RATE);
        assert!(matches!(
            run(&mut plugin, &mut harness, &silence),
            ProcessStatus::Tail(_)
        ));
    }

    #[test]
    fn reset_clears_the_tail_between_activations() {
        let mut used = configured();
        let mut fresh = configured();
        let mut used_harness = Harness::new(2, 256);
        let mut fresh_harness = Harness::new(2, 256);

        let mut loud = [0.0_f32; 256];
        loud[0] = 1.0;
        for _ in 0..8 {
            run(&mut used, &mut used_harness, &loud);
        }
        used.reset();

        let silence = [0.0_f32; 256];
        run(&mut used, &mut used_harness, &silence);
        run(&mut fresh, &mut fresh_harness, &silence);
        assert_eq!(used_harness.output(0), fresh_harness.output(0));
        assert_eq!(used_harness.output(1), fresh_harness.output(1));
    }

    #[test]
    fn activation_publishes_the_rate_for_the_probe() {
        let mut plugin = Kirya::default();
        plugin.configure(96_000.0);
        let bits = plugin.params.sample_rate_bits.load(Ordering::Relaxed);
        assert_eq!(f64::from_bits(bits), 96_000.0);
    }

    #[cfg(feature = "rt-audit")]
    #[test]
    fn process_allocates_nothing_even_while_size_and_freeze_move() {
        // Eight delay lines plus a Size control is the likeliest place for an
        // accidental reallocation on the audio thread, so sweep Size across
        // its whole range and toggle Freeze while auditing.
        const BLOCK_SIZE: usize = 256;
        const BLOCKS: usize = 400;

        let mut plugin = configured();
        let mut harness = Harness::new(2, BLOCK_SIZE);
        let input = [0.25_f32; BLOCK_SIZE];
        run(&mut plugin, &mut harness, &input);

        let (_, allocations) = musictools_core::rt_audit::audit(|| {
            for block in 0..BLOCKS {
                #[allow(clippy::cast_precision_loss)]
                let sweep = block as f32 / BLOCKS as f32;
                automate(&plugin.params.size, 0.05 + sweep * (4.0 - 0.05), RATE);
                automate(&plugin.params.pre_delay, sweep * 500.0, RATE);
                automate(&plugin.params.freeze, block % 32 >= 16, RATE);
                let _ = run(&mut plugin, &mut harness, &input);
            }
        });

        assert_eq!(allocations, 0);
    }
}
