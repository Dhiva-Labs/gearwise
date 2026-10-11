//! Logitech and Logitech G mice with their own drawing. One file per model (plus a
//! `*_shapes.rs` for the measured geometry), registered in [`models`]; see `../README.md`.

mod g502;
mod g502_shapes;
mod mx_master;
mod mx_master_shapes;

use super::ModelArt;

/// The Logitech drawings, most specific first within a family.
pub fn models() -> &'static [&'static dyn ModelArt] {
    &[&g502::G502, &mx_master::MxMaster]
}

#[cfg(test)]
mod tests;
