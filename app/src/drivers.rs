//! The hardware seam: which HID backend and vendor drivers the app runs with.

use periph_core::transport::MockBackend;
use periph_core::{Driver, HidBackend};

/// HID backend used by the device worker. Falls back to an empty backend
/// (no devices, demo mode still works) if hidapi can't start.
pub fn backend() -> Box<dyn HidBackend> {
    match periph_hid::HidapiBackend::new() {
        Ok(b) => Box::new(b),
        Err(e) => {
            log::error!("HID backend unavailable: {e}");
            offline()
        }
    }
}

/// A backend with no devices.
pub fn offline() -> Box<dyn HidBackend> {
    Box::new(MockBackend::new(Vec::new(), |_| {
        Err(periph_core::Error::Disconnected)
    }))
}

/// Vendor drivers the device manager probes interfaces with.
pub fn drivers() -> Vec<Box<dyn Driver>> {
    vec![
        Box::new(periph_logitech::LogitechDriver::new()),
        Box::new(periph_razer::RazerDriver::new()),
        Box::new(periph_corsair::CorsairDriver),
        Box::new(periph_steelseries::SteelSeriesDriver::new()),
    ]
}
