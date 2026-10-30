//! neon-ui — the Dark Neon UI kit shared by every editor in the suite.
//!
//! The style guide is `specs/design.md`; where a rendered editor and the spec
//! disagree, the sketch (`specs/design-sketch.png`) is canonical. The kit is
//! framework-free: painters and interactive widgets over plain values. Param
//! and meter bindings land in a later task, after the plugin-framework
//! migration.
//!
//! The living version of this kit is `cargo run -p neon-ui --example showcase`.

/// Param bindings over nice-plug, behind the `nice-plug` feature.
#[cfg(feature = "nice-plug")]
pub mod bindings;
pub mod button;
pub mod fonts;
pub mod glow;
pub mod grid;
pub mod header;
pub mod knob;
pub mod meter;
pub mod palette;
pub mod select;
pub mod slider;
pub mod surface;
pub mod tabs;
pub mod text;
pub mod value_entry;
pub mod visuals;
