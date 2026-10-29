//! neon-ui — the Dark Neon UI kit shared by every editor in the suite.
//!
//! The style guide is `specs/design.md`; where a rendered editor and the spec
//! disagree, the sketch (`specs/design-sketch.png`) is canonical. The kit is
//! framework-free: painters and interactive widgets over plain values. Param
//! and meter bindings land in a later task, after the plugin-framework
//! migration.
//!
//! The living version of this kit is `cargo run -p neon-ui --example showcase`.

pub mod button;
pub mod glow;
pub mod knob;
pub mod palette;
pub mod slider;
pub mod surface;
pub mod value_entry;
pub mod visuals;
