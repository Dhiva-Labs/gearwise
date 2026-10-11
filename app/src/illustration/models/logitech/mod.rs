//! Logitech and Logitech G mice with their own drawing. One file per model (plus a
//! `*_shapes.rs` for the measured geometry), registered in [`models`]; see `../README.md`.

mod common;
mod g305;
mod g305_shapes;
mod g502;
mod g502_shapes;
mod g502x;
mod g502x_shapes;
mod g600;
mod g600_shapes;
mod g703;
mod g703_shapes;
mod g903;
mod g903_shapes;
mod mx_anywhere;
mod mx_anywhere_shapes;
mod mx_master;
mod mx_master_shapes;
mod superlight2;
mod superlight2_shapes;

use super::ModelArt;

/// The Logitech drawings, most specific first within a family.
pub fn models() -> &'static [&'static dyn ModelArt] {
    &[
        &superlight2::Superlight2,
        &g305::G305,
        &g703::G703,
        &g903::G903,
        &g600::G600,
        &g502x::G502X,
        &g502::G502,
        &mx_master::MxMaster,
        &mx_anywhere::MxAnywhere,
    ]
}

#[cfg(test)]
mod tests;
