//! Razer mice with their own drawing. One file per model (plus a `*_shapes.rs` for the measured
//! geometry), registered in [`models`]; see `../README.md`. `shared` holds the pieces the
//! drawings have in common (smoothed silhouette, knurled wheel, rubber grips, logo mark).

mod basilisk_v3;
mod basilisk_v3_shapes;
mod cobra_pro;
mod cobra_pro_shapes;
mod deathadder_v3;
mod deathadder_v3_shapes;
mod naga_v2_pro;
mod naga_v2_pro_shapes;
mod shared;
mod viper_ultimate;
mod viper_ultimate_shapes;
mod viper_v3_pro;
mod viper_v3_pro_shapes;

use super::ModelArt;

/// The Razer drawings, most specific first within a family.
pub fn models() -> &'static [&'static dyn ModelArt] {
    &[
        &basilisk_v3::BasiliskV3,
        &naga_v2_pro::NagaV2Pro,
        &cobra_pro::CobraPro,
        &viper_v3_pro::ViperV3Pro,
        &viper_ultimate::ViperUltimate,
        &deathadder_v3::DeathAdderV3,
    ]
}

#[cfg(test)]
mod tests;
