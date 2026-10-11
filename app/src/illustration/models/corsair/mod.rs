//! Corsair mice with their own drawing. One file per model (plus a `*_shapes.rs` for the measured
//! geometry), registered in [`models`]; see `../README.md`. [`common`] holds the painting helpers
//! the SteelSeries drawings reuse.

pub(crate) mod common;
mod dark_core;
mod dark_core_shapes;
mod harpoon;
mod harpoon_shapes;
mod katar;
mod katar_shapes;
mod m65;
mod m65_shapes;
mod scimitar;
mod scimitar_shapes;

use super::ModelArt;

/// The Corsair drawings, most specific first within a family.
pub fn models() -> &'static [&'static dyn ModelArt] {
    &[
        &m65::M65,
        &dark_core::DarkCore,
        &katar::Katar,
        &harpoon::Harpoon,
        &scimitar::Scimitar,
    ]
}

#[cfg(test)]
mod tests;
