//! Per-brand demo devices and screenshot-tour steps for the model-accurate mouse art: one file
//! per brand, so the people adding models for different brands never edit the same file.
//!
//! * `devices()` are simulated mice that mirror real products (buttons named as the driver
//!   names them) and appear in the sidebar of the tour, after the G502 HERO.
//! * `steps()` are the tour steps showing them. Step names start with the brand's number
//!   range (Logitech 3x, Razer 4x, Corsair 5x, SteelSeries 6x) so the files never collide.
//!
//! Model keys (`"<vendor>:<pid>"`) must be unique across all demo devices.

mod corsair;
mod logitech;
mod razer;
mod steelseries;

use crate::screenshot::Step;
use crate::worker::DeviceSnapshot;

pub(crate) fn devices() -> Vec<DeviceSnapshot> {
    [
        logitech::devices(),
        razer::devices(),
        corsair::devices(),
        steelseries::devices(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub(crate) fn steps() -> Vec<Step> {
    [
        logitech::steps(),
        razer::steps(),
        corsair::steps(),
        steelseries::steps(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
