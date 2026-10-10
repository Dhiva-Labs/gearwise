//! Logitech driver: HID++ 2.0 over USB (wired), Lightspeed / Unifying /
//! Bolt receivers, and Bluetooth.
//!
//! Devices are discovered generically from their HID++ feature set, so any
//! HID++ 2.0 G-series or MX device works without a per-model table: DPI
//! (0x2201), report rate (0x8060 / 0x8061), lighting (0x8070 / 0x8071),
//! per-key RGB (0x8081), onboard profiles with button bindings (0x8100),
//! diverted buttons and G-keys for software actions (0x1b04 / 0x8010), and
//! battery (0x1000 / 0x1001 / 0x1004).

mod device;
mod features;
mod hidpp;
mod onboard;

use gearwise_core::model::{VID_LOGITECH, Vendor};
use gearwise_core::transport::{BusKind, HidBackend, HidInterfaceInfo, share};
use gearwise_core::{Device, Driver, Error, HidTransport, Result};

pub use device::{GKEY_BASE, LogiDevice};

use crate::device::Location;
use crate::hidpp::{Channel, DIRECT_INDEX};

/// HID++ vendor usage page (USB) and its short/long collections.
const UP_HIDPP: u16 = 0xff00;
const USAGE_SHORT: u16 = 0x0001;
const USAGE_LONG: u16 = 0x0002;
/// HID++ over Bluetooth LE.
const UP_HIDPP_BLE: u16 = 0xff43;
const USAGE_BLE_LONG: u16 = 0x0202;

/// Receiver product ids (Unifying, Nano, Lightspeed, Bolt).
const RECEIVERS: &[u16] = &[
    0xc52b, 0xc532, 0xc52f, 0xc518, 0xc51a, 0xc51b, 0xc521, 0xc525, 0xc526, 0xc52e, 0xc531, 0xc534,
    0xc535, 0xc537, 0xc539, 0xc53a, 0xc53d, 0xc53f, 0xc541, 0xc545, 0xc547, 0xc54d, 0xc548,
];

pub fn is_receiver(pid: u16) -> bool {
    RECEIVERS.contains(&pid)
}

#[derive(Default)]
pub struct LogitechDriver;

impl LogitechDriver {
    pub fn new() -> Self {
        Self
    }
}

/// Interface selection, separated from the platform for testing.
///
/// * Linux lists one entry per hidraw node with its first collection's usage
///   (short, or long on long-only devices); either is the whole interface.
/// * Windows lists each collection separately; claim the long one and pair
///   it with the short sibling in [`open`](Driver::open).
/// * On USB only wired devices (0xc0xx–0xc3xx) and receivers are claimed:
///   the Linux `hid-logitech-dj` driver also exposes a virtual node per
///   paired wireless device, which would duplicate what the receiver serves.
fn wants(iface: &HidInterfaceInfo, windows: bool) -> bool {
    if iface.vendor_id != VID_LOGITECH {
        return false;
    }
    if iface.usage_page == UP_HIDPP_BLE {
        return iface.usage == USAGE_BLE_LONG || !windows;
    }
    if iface.usage_page != UP_HIDPP {
        return false;
    }
    let usage_ok = match iface.usage {
        USAGE_LONG => true,
        USAGE_SHORT => !windows,
        _ => false,
    };
    let pid_ok = iface.bus == BusKind::Bluetooth || (0xc000..=0xc5ff).contains(&iface.product_id);
    usage_ok && pid_ok
}

impl Driver for LogitechDriver {
    fn name(&self) -> &'static str {
        "logitech"
    }

    fn vendor(&self) -> Vendor {
        Vendor::Logitech
    }

    fn probe(&self, iface: &HidInterfaceInfo) -> bool {
        wants(iface, cfg!(windows))
    }

    fn open(
        &self,
        iface: &HidInterfaceInfo,
        backend: &dyn HidBackend,
    ) -> Result<Vec<Box<dyn Device>>> {
        let long = share(backend.open(&iface.path)?);
        let short = if cfg!(windows) {
            short_sibling(iface, backend)?.map(share)
        } else {
            None
        };
        let ch = Channel::new(long, short);
        let loc = Location {
            vendor_id: iface.vendor_id,
            product_id: iface.product_id,
            path: iface.path.clone(),
            serial: iface.serial.clone().filter(|s| !s.is_empty()),
            bluetooth: iface.bus == BusKind::Bluetooth,
        };
        open_channel(ch, &loc, is_receiver(iface.product_id))
    }
}

fn open_channel(ch: Channel, loc: &Location, receiver: bool) -> Result<Vec<Box<dyn Device>>> {
    if !receiver {
        return Ok(vec![Box::new(LogiDevice::open(ch, DIRECT_INDEX, loc)?)]);
    }
    // Paired devices sit at indices 1–6; asleep or empty slots answer with
    // an error (or not at all) and are skipped until the next scan.
    let mut devices: Vec<Box<dyn Device>> = Vec::new();
    for index in 1..=6u8 {
        match LogiDevice::open(ch.clone(), index, loc) {
            Ok(d) => devices.push(Box::new(d)),
            Err(Error::Disconnected | Error::Timeout | Error::Unsupported(_)) => {}
            Err(e) => log::debug!("receiver {:04x} slot {index}: {e}", loc.product_id),
        }
    }
    Ok(devices)
}

/// On Windows, the short-report collection of the same HID++ interface.
fn short_sibling(
    iface: &HidInterfaceInfo,
    backend: &dyn HidBackend,
) -> Result<Option<Box<dyn HidTransport>>> {
    let sibling = backend.enumerate()?.into_iter().find(|i| {
        i.vendor_id == iface.vendor_id
            && i.product_id == iface.product_id
            && i.interface_number == iface.interface_number
            && i.usage_page == UP_HIDPP
            && i.usage == USAGE_SHORT
            && i.serial == iface.serial
            && i.path != iface.path
    });
    match sibling {
        Some(s) => Ok(Some(backend.open(&s.path)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests;
