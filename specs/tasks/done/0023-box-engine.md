---
id: 0023-box-engine
status: done
scope: [crates/box-dsp/]
created: 2026-10-30
source: user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
depends: [0014-grid-foundation]
---
# Task: box engine

## Plan

- [x] Patch model (serde): nodes `{uid, kind, cell (col, row), params by id, powered}` plus cables `{from: (uid, port), to: (uid, port)}`; Output modules are the terminal nodes. Unknown kinds and param ids are skipped on load, missing ones take defaults.
- [x] Compiler (off the audio thread): validate ports, Kahn topological sort, refuse a cable that would close a cycle, cull nodes with no path to an Output, assign buffers by liveness (reuse), fan-in as a sum step → flat `Plan { steps, buffer_count }`.
- [x] Executor: one `dyn Module::process` per module per block, buffers preallocated at `max_block`, powered-off modules copy through, `finite_or` on graph input and output, host blocks above `max_block` chunked.
- [x] Swap: new plans and freshly created modules arrive over an `rtrb` SPSC queue and install at a block boundary; surviving modules are relinked by uid so tails survive a re-patch; removed modules and old plans go back over a return queue and drop off the audio thread.
- [x] Tests: a chain matches direct chaining sample for sample; fan-in sums; a cycle is refused; culled nodes don't run; a re-patch keeps a surviving module's state; non-finite input stays finite; rt-audit zero allocations across process and swaps.
- [x] `box_bench` example (`--release`): µs per block for 1, 8 and 32 module chains at 64/256/1024 samples, printed as a table.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `cargo run -p box-dsp --release --example box_bench`

## Context

- Second task of the box chain (see 0014's Context for the user's goals and decisions). Performance-first: no locks, no allocation, no frees on the audio thread; one virtual call per module per block.
- `rtrb` is a new workspace dependency (lock-free SPSC ring), added here.
- Reference for audio-safe handoffs: Igorek's swap protocol (`specs/igorek.md`) and Kirya's request-id guard (`plugins/kirya/src/task.rs`).
- v1 modules report zero latency; graph-internal delay compensation is out of scope until a module needs it.

## Review

- Matches the plan inside scope: `crates/box-dsp/` plus workspace deps rtrb, serde and serde_json.
- UI side: `Controller` compiles the patch, creates new modules and owns each node's `Arc<Shared>`.
- Audio side: `Engine::process` installs at the block boundary, relinks survivors by uid, sends old graphs back (holding one if the return queue is full), runs the plan and chunks above `max_block`.
- No `unsafe`: each step's output buffers are `mem::take`n out of the pool and its inputs borrowed from it. The compiler keeps the two disjoint.
- A 32-module chain uses 5 buffers.
- Deviations:
  - There's no graph input yet. `finite_or` covers the graph output; the host Input module (and its input guard) arrive with 0025.
  - Unknown kinds and broken cables are skipped at compile, with a `skipped_cables` count. Params with unknown ids are skipped when `Shared` is created.
- Fix found while building 0027: the controller now tracks each instance's kind, so a uid reused for a different kind gets a fresh module and param store instead of the old one. `forget_instances()` gives a loaded patch fresh state.
- 19 tests pass, including the rt-audit run across process and four swaps. The gate was re-run after the fix.
- Bench on this machine: 32 modules cost 2.3 µs per 64-sample block (0.18 % of the budget) and 5.9 µs per 1024-sample block.

## Summary

box engine: patch model, acyclic graph compiler with buffer reuse, and an allocation-free executor with lock-free plan swaps.

## Log

- 2026-10-30 created: box engine
- 2026-10-30 set source user request 2026-10-30: grid foundation as "box" — expandable module interface, performance first, compact views on the grid and the full plugin in an inspector
- 2026-10-30 set depends 0014-grid-foundation
- 2026-10-30 scope add crates/box-dsp/ — patch model, compiler, realtime engine
- 2026-10-31 set source user request 2026-10-31: box standalone app first, one tile UI per module on a fixed cell grid, basic modules (sine, gates, env, output)
- 2026-10-31 draft approved: let's plan to implement basic version of grid to have an idea — Full chain as drafted; Allow moving tiles too; plan approved (user, 2026-10-31)
- 2026-10-31 advance draft → ready
- 2026-10-31 scope approved: crates/box-dsp/
- 2026-10-31 advance ready → in-progress/implement
- 2026-10-31 baseline: crates/box-core/Cargo.toml@da582a43791d, crates/box-core/src/descriptor.rs@f325c22c029d, crates/box-core/src/geometry.rs@6ac7920056bd, crates/box-core/src/lib.rs@cf56491f7121, crates/box-core/src/module.rs@9f4f2c5e3d85, crates/box-core/src/param.rs@ab123c6fa2a3, crates/box-core/src/port.rs@4d0e5d21e9b0, crates/box-core/src/smoother.rs@8b9b30520424, crates/box-core/src/store.rs@05f60ec72cb8
- 2026-10-31 advance in-progress/implement → in-progress/verify
- 2026-10-31 advance in-progress/verify → in-progress/review
- 2026-10-31 advance in-progress/review → approval
- 2026-11-01 approved: Wow it looks good and feels good! — It worked, close (user, 2026-11-01)
- 2026-11-01 advance approval → done
