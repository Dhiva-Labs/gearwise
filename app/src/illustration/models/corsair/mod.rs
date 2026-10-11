//! Corsair mice with their own drawing. One file per model (plus a `*_shapes.rs` for the measured
//! geometry when it is large), registered in [`models`]; see `../README.md`.

use super::ModelArt;

/// The Corsair drawings, most specific first within a family. Empty until the first model lands.
pub fn models() -> &'static [&'static dyn ModelArt] {
    &[]
}
