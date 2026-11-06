//! Toy modules for the engine's tests and the `box_bench` example. Real
//! modules live in `box-modules`; these exist so box-dsp can test itself
//! without depending on them.

use box_core::{
    BlockIo, Meters, Module, ModuleDescriptor, ParamSpec, ParamValues, PortSpec, Role, Side, Taper,
};

/// Kind ids of the toys.
pub const CONST: &str = "toy.const";
pub const GAIN: &str = "toy.gain";
pub const COUNTER: &str = "toy.counter";
pub const NAN: &str = "toy.nan";
pub const SINK: &str = "toy.sink";

static IN: [PortSpec; 1] = [PortSpec::audio("in", "In", Side::Left, 0)];
static OUT: [PortSpec; 1] = [PortSpec::audio("out", "Out", Side::Right, 0)];
static SINK_IN: [PortSpec; 2] = [
    PortSpec::audio("l", "L", Side::Left, 0),
    PortSpec::audio("r", "R", Side::Left, 1),
];
static SINK_OUT: [PortSpec; 2] = [
    PortSpec::audio("l", "L", Side::Right, 0),
    PortSpec::audio("r", "R", Side::Right, 1),
];
static VALUE: [ParamSpec; 1] = [ParamSpec::float(
    "value",
    "Value",
    -10.0,
    10.0,
    0.5,
    Taper::Linear,
)];

const fn toy(
    kind: &'static str,
    inputs: &'static [PortSpec],
    outputs: &'static [PortSpec],
    params: &'static [ParamSpec],
    role: Role,
    create: fn(f32, usize) -> Box<dyn Module>,
) -> ModuleDescriptor {
    ModuleDescriptor {
        kind,
        name: kind,
        category: "Toy",
        size: (1, 1),
        inputs,
        outputs,
        params,
        meters: &["runs"],
        role,
        create,
    }
}

/// Outputs its `value` param.
pub static CONST_DESCRIPTOR: ModuleDescriptor =
    toy(CONST, &[], &OUT, &VALUE, Role::Normal, |_, _| {
        Box::new(Const)
    });
/// Multiplies its input by `value`.
pub static GAIN_DESCRIPTOR: ModuleDescriptor =
    toy(GAIN, &IN, &OUT, &VALUE, Role::Normal, |_, _| Box::new(Gain));
/// Outputs a running sample count: state that must survive a re-patch.
pub static COUNTER_DESCRIPTOR: ModuleDescriptor =
    toy(COUNTER, &[], &OUT, &[], Role::Normal, |_, _| {
        Box::new(Counter(0.0))
    });
/// Outputs NaN.
pub static NAN_DESCRIPTOR: ModuleDescriptor =
    toy(NAN, &[], &OUT, &[], Role::Normal, |_, _| Box::new(Nan));
/// Stereo sink; R falls back to L when unconnected.
pub static SINK_DESCRIPTOR: ModuleDescriptor =
    toy(SINK, &SINK_IN, &SINK_OUT, &[], Role::Sink, |_, _| {
        Box::new(Sink)
    });

/// The toy registry.
#[must_use]
pub fn registry(kind: &str) -> Option<&'static ModuleDescriptor> {
    [
        &CONST_DESCRIPTOR,
        &GAIN_DESCRIPTOR,
        &COUNTER_DESCRIPTOR,
        &NAN_DESCRIPTOR,
        &SINK_DESCRIPTOR,
    ]
    .into_iter()
    .find(|descriptor| descriptor.kind == kind)
}

fn count_run(meters: &Meters) {
    meters.set(0, meters.get(0) + 1.0);
}

struct Const;
impl Module for Const {
    fn reset(&mut self, _: f32, _: usize) {}
    fn clear(&mut self) {}
    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        io.outputs[0].fill(params.get(0));
        count_run(meters);
    }
}

struct Gain;
impl Module for Gain {
    fn reset(&mut self, _: f32, _: usize) {}
    fn clear(&mut self) {}
    fn process(&mut self, io: &mut BlockIo<'_, '_>, params: &ParamValues<'_>, meters: &Meters) {
        let gain = params.get(0);
        for (out, &sample) in io.outputs[0].iter_mut().zip(io.inputs[0]) {
            *out = sample * gain;
        }
        count_run(meters);
    }
}

struct Counter(f32);
impl Module for Counter {
    fn reset(&mut self, _: f32, _: usize) {}
    fn clear(&mut self) {
        self.0 = 0.0;
    }
    fn process(&mut self, io: &mut BlockIo<'_, '_>, _: &ParamValues<'_>, meters: &Meters) {
        for out in io.outputs[0].iter_mut() {
            *out = self.0;
            self.0 += 1.0;
        }
        count_run(meters);
    }
}

struct Nan;
impl Module for Nan {
    fn reset(&mut self, _: f32, _: usize) {}
    fn clear(&mut self) {}
    fn process(&mut self, io: &mut BlockIo<'_, '_>, _: &ParamValues<'_>, _: &Meters) {
        io.outputs[0].fill(f32::NAN);
    }
}

struct Sink;
impl Module for Sink {
    fn reset(&mut self, _: f32, _: usize) {}
    fn clear(&mut self) {}
    fn process(&mut self, io: &mut BlockIo<'_, '_>, _: &ParamValues<'_>, meters: &Meters) {
        let right = if io.is_connected(1) { 1 } else { 0 };
        let (left_in, right_in) = (io.inputs[0], io.inputs[right]);
        io.outputs[0].copy_from_slice(left_in);
        io.outputs[1].copy_from_slice(right_in);
        count_run(meters);
    }
}
