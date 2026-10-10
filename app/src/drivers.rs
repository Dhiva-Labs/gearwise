//! The hardware seam: which HID backend and vendor drivers the app runs with.

use periph_core::transport::MockBackend;
use periph_core::{Driver, HidBackend};

/// HID backend used by the device worker.
pub fn backend() -> Box<dyn HidBackend> {
    Box::new(MockBackend::new(Vec::new(), |_| Err(periph_core::Error::Disconnected)))
}

/// Vendor drivers the device manager probes interfaces with.
pub fn drivers() -> Vec<Box<dyn Driver>> {
    Vec::new()
}
