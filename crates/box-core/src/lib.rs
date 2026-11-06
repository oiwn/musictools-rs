//! box-core — the standard interface every box module implements.
//!
//! A module is a [`ModuleDescriptor`] (its kind, tile size in cells, ports
//! and params) plus a [`Module`] that processes whole blocks. Param values
//! live in a lock-free [`ParamStore`] the UI writes and the engine snapshots
//! once per block. Nothing here knows about egui or a plugin framework; see
//! `specs/box.md` for the grid model and the add-a-module recipe.

pub mod descriptor;
pub mod geometry;
pub mod module;
pub mod param;
pub mod pitch;
pub mod port;
pub mod smoother;
pub mod store;

pub use descriptor::{DescriptorError, ModuleDescriptor, Role};
pub use module::{BlockIo, Module};
pub use param::{ParamKind, ParamSpec, Taper};
pub use port::{MAX_PORTS, PortKind, PortSpec, Side};
pub use smoother::Smoother;
pub use store::{Meters, ParamStore, ParamValues, Shared};

#[cfg(test)]
mod tests {
    //! The whole interface exercised through a toy Gain module.

    use super::*;

    struct Gain {
        smoother: Smoother,
        rate: f32,
    }

    static GAIN_INPUTS: [PortSpec; 1] = [PortSpec::audio("in", "In", Side::Left, 0)];
    static GAIN_OUTPUTS: [PortSpec; 1] = [PortSpec::audio("out", "Out", Side::Right, 0)];
    static GAIN_PARAMS: [ParamSpec; 1] =
        [ParamSpec::float("gain", "Gain", 0.0, 2.0, 1.0, Taper::Linear).smoothing(5.0)];
    static GAIN: ModuleDescriptor = ModuleDescriptor {
        kind: "test.gain",
        name: "Gain",
        category: "Test",
        size: (1, 1),
        inputs: &GAIN_INPUTS,
        outputs: &GAIN_OUTPUTS,
        params: &GAIN_PARAMS,
        meters: &["peak"],
        role: Role::Normal,
        create: |rate, max_block| {
            let mut gain = Gain {
                smoother: Smoother::new(1.0),
                rate,
            };
            gain.reset(rate, max_block);
            Box::new(gain)
        },
    };

    impl Module for Gain {
        fn reset(&mut self, sample_rate: f32, _max_block: usize) {
            self.rate = sample_rate;
            self.smoother
                .set_time(sample_rate, GAIN_PARAMS[0].smoothing_ms);
        }

        fn clear(&mut self) {
            self.smoother.snap(self.smoother.current());
        }

        fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
            self.smoother.set_target(params.get(0));
            let mut peak = 0.0_f32;
            for (out, &sample) in io.outputs[0][..io.len].iter_mut().zip(io.inputs[0]) {
                *out = sample * self.smoother.next();
                peak = peak.max(out.abs());
            }
            meters.set(0, peak);
        }
    }

    fn run(module: &mut dyn Module, shared: &Shared, input: &[f32], output: &mut [f32]) {
        let mut values = [0.0; 1];
        shared.params.snapshot_into(&mut values);
        let inputs = [input];
        let mut outputs = [output];
        let mut io = BlockIo {
            inputs: &inputs,
            outputs: &mut outputs,
            len: input.len(),
            connected: 1,
        };
        module.process(&mut io, &ParamValues(&values), &shared.meters);
    }

    #[test]
    fn toy_descriptor_validates() {
        assert_eq!(GAIN.validate(), Ok(()));
        assert_eq!(GAIN.input_index("in"), Some(0));
        assert_eq!(GAIN.output_index("out"), Some(0));
    }

    #[test]
    fn malformed_descriptors_are_rejected() {
        static CLASH: [PortSpec; 2] = [
            PortSpec::audio("a", "A", Side::Left, 0),
            PortSpec::audio("b", "B", Side::Left, 0),
        ];
        let clash = ModuleDescriptor {
            inputs: &CLASH,
            ..GAIN
        };
        assert_eq!(clash.validate(), Err(DescriptorError::BadSlot("b")));
        let sink = ModuleDescriptor {
            role: Role::Sink,
            ..GAIN
        };
        assert_eq!(sink.validate(), Err(DescriptorError::SinkOutputs));
        let empty = ModuleDescriptor {
            size: (0, 1),
            ..GAIN
        };
        assert_eq!(empty.validate(), Err(DescriptorError::EmptySize));
    }

    #[test]
    fn port_positions_sit_on_the_tile_edge() {
        let tile = geometry::tile_area(GAIN.size);
        for port in GAIN.visible_ports() {
            let (x, y) = geometry::port_anchor(&GAIN, port);
            assert!(tile.contains(x, y), "{} at ({x}, {y})", port.id);
            let on_edge =
                x == tile.x || x == tile.x + tile.w || y == tile.y || y == tile.y + tile.h;
            assert!(on_edge, "{} at ({x}, {y})", port.id);
        }
        let (_, y) = geometry::port_anchor(&GAIN, &GAIN_INPUTS[0]);
        let body = geometry::body_area(GAIN.size);
        assert_eq!(y, body.y + body.h / 2.0);
    }

    #[test]
    fn gain_processes_and_publishes_its_peak() {
        let shared = Shared::new(&GAIN);
        shared.params.set(0, 0.5);
        let mut module = (GAIN.create)(1000.0, 64);
        let input = [1.0; 32];
        let mut output = [0.0; 32];
        run(module.as_mut(), &shared, &input, &mut output);
        assert!((output[31] - 0.5).abs() < 1e-6, "{}", output[31]);
        assert!(output[0] < 1.0 && output[0] > 0.5);
        assert_eq!(shared.meters.get(0), output[0]);
    }

    #[cfg(feature = "rt-audit")]
    #[test]
    fn process_allocates_nothing() {
        let shared = Shared::new(&GAIN);
        let mut module = (GAIN.create)(48_000.0, 256);
        let input = vec![0.25; 256];
        let mut output = vec![0.0; 256];
        let (_, allocations) = musictools_core::rt_audit::audit(|| {
            for _ in 0..16 {
                shared.params.set(0, 1.5);
                run(module.as_mut(), &shared, &input, &mut output);
            }
        });
        assert_eq!(allocations, 0);
    }
}
