//! The cpal output stream that runs the engine.
//!
//! The callback is the audio thread: it only calls `Engine::process`,
//! copies into the device buffer, and records its own load in atomics.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use box_dsp::Engine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Frames rendered per engine call inside one callback.
const SCRATCH: usize = 2048;

/// What the status line shows about the device.
#[derive(Clone, Debug)]
pub struct AudioInfo {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Load and block size measured inside the callback.
#[derive(Debug, Default)]
pub struct Stats {
    /// Smoothed fraction of the block's duration spent rendering.
    load: AtomicU32,
    /// Frames in the latest callback.
    frames: AtomicU32,
}

impl Stats {
    /// Smoothed engine load, 0..=1 (can exceed 1 when overloaded).
    #[must_use]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.load.load(Ordering::Relaxed))
    }

    /// Frames in the latest callback.
    #[must_use]
    pub fn frames(&self) -> u32 {
        self.frames.load(Ordering::Relaxed)
    }

    fn record(&self, frames: usize, seconds: f32, sample_rate: f32) {
        let budget = frames as f32 / sample_rate;
        if budget > 0.0 {
            let now = seconds / budget;
            let smoothed = self.load() * 0.9 + now * 0.1;
            self.load.store(smoothed.to_bits(), Ordering::Relaxed);
        }
        self.frames.store(frames as u32, Ordering::Relaxed);
    }
}

/// A running output stream. Dropping it stops audio.
pub struct Audio {
    _stream: cpal::Stream,
    pub info: AudioInfo,
    pub stats: Arc<Stats>,
}

/// The default output device and its f32 config.
pub struct Device {
    device: cpal::Device,
    config: cpal::StreamConfig,
    pub info: AudioInfo,
}

/// Finds the default output device.
pub fn open() -> Result<Device, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no output device".to_owned())?;
    let supported = device
        .default_output_config()
        .map_err(|error| format!("no output config: {error}"))?;
    if supported.sample_format() != cpal::SampleFormat::F32 {
        return Err(format!(
            "unsupported sample format {}",
            supported.sample_format()
        ));
    }
    let config: cpal::StreamConfig = supported.into();
    let info = AudioInfo {
        device: device.to_string(),
        sample_rate: config.sample_rate,
        channels: config.channels,
    };
    Ok(Device {
        device,
        config,
        info,
    })
}

/// Starts the stream with `engine` running in its callback.
pub fn start(device: Device, mut engine: Engine) -> Result<Audio, String> {
    let stats = Arc::new(Stats::default());
    let callback_stats = Arc::clone(&stats);
    let channels = usize::from(device.info.channels.max(1));
    let sample_rate = device.info.sample_rate as f32;
    let mut left = vec![0.0_f32; SCRATCH];
    let mut right = vec![0.0_f32; SCRATCH];
    let stream = device
        .device
        .build_output_stream(
            device.config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let started = Instant::now();
                for chunk in data.chunks_mut(SCRATCH * channels) {
                    let frames = chunk.len() / channels;
                    engine.process(&mut left[..frames], &mut right[..frames]);
                    write_frames(chunk, channels, &left[..frames], &right[..frames]);
                }
                callback_stats.record(
                    data.len() / channels,
                    started.elapsed().as_secs_f32(),
                    sample_rate,
                );
            },
            |error| eprintln!("box: audio stream error: {error}"),
            None,
        )
        .map_err(|error| format!("cannot open stream: {error}"))?;
    stream
        .play()
        .map_err(|error| format!("cannot start stream: {error}"))?;
    Ok(Audio {
        _stream: stream,
        info: device.info,
        stats,
    })
}

/// Interleaves stereo into a device buffer of `channels`; a mono device
/// gets the average, extra channels stay silent.
fn write_frames(out: &mut [f32], channels: usize, left: &[f32], right: &[f32]) {
    for ((frame, &l), &r) in out.chunks_mut(channels).zip(left).zip(right) {
        match frame {
            [mono] => *mono = 0.5 * (l + r),
            [first, second, rest @ ..] => {
                *first = l;
                *second = r;
                rest.fill(0.0);
            }
            [] => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_interleave_per_channel_count() {
        let mut stereo = [9.0; 6];
        write_frames(&mut stereo, 2, &[1.0, 2.0, 3.0], &[-1.0, -2.0, -3.0]);
        assert_eq!(stereo, [1.0, -1.0, 2.0, -2.0, 3.0, -3.0]);
        let mut mono = [9.0; 2];
        write_frames(&mut mono, 1, &[1.0, 0.0], &[0.0, 1.0]);
        assert_eq!(mono, [0.5, 0.5]);
        let mut quad = [9.0; 4];
        write_frames(&mut quad, 4, &[1.0], &[2.0]);
        assert_eq!(quad, [1.0, 2.0, 0.0, 0.0]);
    }

    #[test]
    fn load_is_smoothed_against_the_block_budget() {
        let stats = Stats::default();
        stats.record(480, 0.005, 48_000.0);
        assert!((stats.load() - 0.05).abs() < 1e-6);
        assert_eq!(stats.frames(), 480);
    }
}
