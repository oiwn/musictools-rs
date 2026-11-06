//! Engine cost per block for chains of toy gain modules.
//!
//! `cargo run -p box-dsp --release --example box_bench`

use std::time::Instant;

use box_dsp::toys::{self, registry};
use box_dsp::{Cable, Patch, PatchNode, new_engine};

const SAMPLE_RATE: f32 = 48_000.0;
/// Rendered audio per measurement.
const SECONDS: f32 = 2.0;

fn chain(modules: usize) -> Patch {
    let mut patch = Patch::default();
    patch.nodes.push(PatchNode::new(1, toys::CONST, (0, 0)));
    for index in 0..modules {
        let uid = index as u64 + 2;
        patch.nodes.push(PatchNode::new(uid, toys::GAIN, (0, 0)));
        patch.cables.push(Cable::new(uid - 1, "out", uid, "in"));
    }
    let sink = modules as u64 + 2;
    patch.nodes.push(PatchNode::new(sink, toys::SINK, (0, 0)));
    patch.cables.push(Cable::new(sink - 1, "out", sink, "l"));
    patch
}

fn main() {
    println!(
        "{:>8} {:>7} {:>11} {:>11} {:>9}",
        "modules", "block", "µs/block", "ns/module", "budget %"
    );
    for &modules in &[1_usize, 8, 32] {
        for &block in &[64_usize, 256, 1024] {
            let (mut controller, mut engine) = new_engine(registry, SAMPLE_RATE, block);
            controller.apply(&chain(modules)).expect("chain compiles");
            let mut left = vec![0.0; block];
            let mut right = vec![0.0; block];
            let blocks = ((SAMPLE_RATE * SECONDS) as usize / block).max(1);
            for _ in 0..16 {
                engine.process(&mut left, &mut right);
            }
            let start = Instant::now();
            for _ in 0..blocks {
                engine.process(&mut left, &mut right);
            }
            let per_block = start.elapsed().as_secs_f64() * 1e6 / blocks as f64;
            let per_module = per_block * 1e3 / (modules + 2) as f64;
            let budget = per_block / (block as f64 / f64::from(SAMPLE_RATE) * 1e6) * 100.0;
            println!("{modules:>8} {block:>7} {per_block:>11.2} {per_module:>11.1} {budget:>9.3}");
            std::hint::black_box(&left);
        }
    }
}
