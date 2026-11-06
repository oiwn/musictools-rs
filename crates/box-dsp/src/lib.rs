//! box-dsp — the patch model, the graph compiler, and the realtime engine.
//!
//! The UI side owns a [`Controller`]: it compiles a [`Patch`] into a flat
//! [`Plan`], builds the modules, and hands the result to the audio side's
//! [`Engine`] over a lock-free queue. The engine installs it at a block
//! boundary, moves surviving modules across by uid, and sends the old graph
//! back so it is dropped off the audio thread. See `specs/box.md`.

pub mod compile;
pub mod engine;
mod graph;
pub mod patch;
#[doc(hidden)]
pub mod toys;

pub use compile::{CompileError, Plan, PlanNode, Registry, Step, compile};
pub use engine::{ApplyError, Controller, Engine, new_engine};
pub use patch::{Cable, Patch, PatchNode, PortRef};
