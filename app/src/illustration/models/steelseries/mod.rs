//! SteelSeries mice with their own drawing. One file per model (plus a `*_shapes.rs` for the measured
//! geometry), registered in [`models`]; see `../README.md`. The painting helpers are shared with the
//! Corsair drawings (`corsair::common`).

mod aerox;
mod aerox3_shapes;
mod aerox5_shapes;
mod aerox9_shapes;
mod prime;
mod prime_shapes;
mod rival3;
mod rival3_shapes;
mod rival600;
mod rival600_shapes;
mod sensei;
mod sensei_shapes;

use super::ModelArt;

/// The SteelSeries drawings, most specific first within a family.
pub fn models() -> &'static [&'static dyn ModelArt] {
    &[
        &aerox::Aerox9,
        &aerox::Aerox5,
        &aerox::Aerox3,
        &prime::Prime,
        &rival3::Rival3,
        &rival600::Rival600,
        &sensei::Sensei,
    ]
}

#[cfg(test)]
mod tests;
