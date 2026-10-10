//! Typed access to the HID++ 2.0 features the driver uses.
//!
//! Byte layouts follow the HID++ 2.0 feature documentation as implemented by
//! libratbag (MIT) and Solaar; offsets below are into the reply parameters.

use std::collections::HashMap;

use periph_core::model::{Battery, Rgb};
use periph_core::{Error, Result};

use crate::hidpp::{Channel, HidppError, err10};

pub mod id {
    pub const FIRMWARE: u16 = 0x0003;
    pub const DEVICE_NAME: u16 = 0x0005;
    pub const BATTERY_STATUS: u16 = 0x1000;
    pub const BATTERY_VOLTAGE: u16 = 0x1001;
    pub const UNIFIED_BATTERY: u16 = 0x1004;
    pub const REPROG_CONTROLS_V4: u16 = 0x1b04;
    pub const ADJUSTABLE_DPI: u16 = 0x2201;
    pub const GKEY: u16 = 0x8010;
    pub const REPORT_RATE: u16 = 0x8060;
    pub const EXTENDED_REPORT_RATE: u16 = 0x8061;
    pub const COLOR_LED_EFFECTS: u16 = 0x8070;
    pub const RGB_EFFECTS: u16 = 0x8071;
    pub const PER_KEY_LIGHTING_V2: u16 = 0x8081;
    pub const ONBOARD_PROFILES: u16 = 0x8100;

    /// Features probed at open time.
    pub const PROBED: [u16; 14] = [
        FIRMWARE,
        DEVICE_NAME,
        BATTERY_STATUS,
        BATTERY_VOLTAGE,
        UNIFIED_BATTERY,
        REPROG_CONTROLS_V4,
        ADJUSTABLE_DPI,
        GKEY,
        REPORT_RATE,
        EXTENDED_REPORT_RATE,
        COLOR_LED_EFFECTS,
        RGB_EFFECTS,
        PER_KEY_LIGHTING_V2,
        ONBOARD_PROFILES,
    ];
}

/// One HID++ 2.0 device (a device index on a channel) and its feature map.
pub struct Hidpp {
    pub ch: Channel,
    pub index: u8,
    features: HashMap<u16, u8>,
}

impl Hidpp {
    /// Ping the device: returns the protocol version, or an error if nothing
    /// answers (asleep / unpaired) or it only speaks HID++ 1.0.
    pub fn connect(ch: Channel, index: u8) -> Result<Self> {
        // Root.getProtocolVersion(0, 0, ping byte)
        match ch.request(index, 0x00, 0x1, &[0, 0, 0x5a])? {
            Ok(p) if p.len() >= 3 && p[2] == 0x5a && p[0] >= 2 => {}
            Ok(p) => {
                return Err(Error::protocol(format!(
                    "unexpected protocol reply {p:02x?}"
                )));
            }
            Err(HidppError::V1(err10::INVALID_SUBID)) => {
                return Err(Error::Unsupported("HID++ 1.0 device"));
            }
            Err(e) => return Err(e.into_error()),
        }
        let mut dev = Self {
            ch,
            index,
            features: HashMap::new(),
        };
        for f in id::PROBED {
            if let Some(i) = dev.lookup(f)? {
                dev.features.insert(f, i);
            }
        }
        Ok(dev)
    }

    /// Root.getFeature: the feature's index, or `None` if absent.
    fn lookup(&self, feature: u16) -> Result<Option<u8>> {
        match self
            .ch
            .request(self.index, 0x00, 0x0, &feature.to_be_bytes())?
        {
            Ok(p) if p.first().copied().unwrap_or(0) != 0 => Ok(Some(p[0])),
            Ok(_) => Ok(None),
            Err(HidppError::V2(_)) => Ok(None),
            Err(e) => Err(e.into_error()),
        }
    }

    pub fn has(&self, feature: u16) -> bool {
        self.features.contains_key(&feature)
    }

    pub fn call(&self, feature: u16, function: u8, params: &[u8]) -> Result<Vec<u8>> {
        let idx = *self
            .features
            .get(&feature)
            .ok_or(Error::Unsupported("feature not present"))?;
        let mut p = self.ch.call(self.index, idx, function, params)?;
        p.resize(16, 0);
        Ok(p)
    }

    /// Feature id for a feature index seen in a notification.
    pub fn feature_at(&self, index: u8) -> Option<u16> {
        self.features
            .iter()
            .find(|(_, i)| **i == index)
            .map(|(f, _)| *f)
    }

    // -- 0x0005 device name / type --------------------------------------

    pub fn name(&self) -> Result<String> {
        let len = self.call(id::DEVICE_NAME, 0x0, &[])?[0] as usize;
        let mut name = Vec::with_capacity(len);
        while name.len() < len {
            let chunk = self.call(id::DEVICE_NAME, 0x1, &[name.len() as u8])?;
            let take = (len - name.len()).min(chunk.len());
            if take == 0 {
                break;
            }
            name.extend_from_slice(&chunk[..take]);
        }
        let s = String::from_utf8_lossy(&name);
        Ok(s.trim_end_matches('\0').trim().to_string())
    }

    /// HID++ device type: 0 keyboard, 3 mouse, 4 touchpad, 5 trackball,
    /// 8 headset, ...
    pub fn device_type(&self) -> Result<u8> {
        Ok(self.call(id::DEVICE_NAME, 0x2, &[])?[0])
    }

    // -- 0x0003 firmware -------------------------------------------------

    pub fn firmware(&self) -> Result<Option<String>> {
        let count = self.call(id::FIRMWARE, 0x0, &[])?[0];
        for entity in 0..count {
            let p = self.call(id::FIRMWARE, 0x1, &[entity])?;
            // type 0 = main application firmware
            if p[0] == 0 {
                let prefix = String::from_utf8_lossy(&p[1..4])
                    .trim_matches(|c: char| c == '\0' || c == ' ')
                    .to_string();
                let build = u16::from_be_bytes([p[6], p[7]]);
                return Ok(Some(format!(
                    "{prefix} {:02x}.{:02x}.B{build:04x}",
                    p[4], p[5]
                )));
            }
        }
        Ok(None)
    }

    // -- battery ---------------------------------------------------------

    pub fn has_battery(&self) -> bool {
        self.has(id::UNIFIED_BATTERY)
            || self.has(id::BATTERY_STATUS)
            || self.has(id::BATTERY_VOLTAGE)
    }

    pub fn battery(&self) -> Result<Option<Battery>> {
        if self.has(id::UNIFIED_BATTERY) {
            // getStatus: [state of charge %, level flags, charging status, ext power]
            let p = self.call(id::UNIFIED_BATTERY, 0x1, &[])?;
            let percent = (p[0] > 0)
                .then_some(p[0].min(100))
                .or_else(|| level_flags_percent(p[1]));
            return Ok(Some(Battery {
                percent,
                charging: matches!(p[2], 1 | 2),
            }));
        }
        if self.has(id::BATTERY_STATUS) {
            // [level %, next level %, status]; status 1..=4 are charging states
            let p = self.call(id::BATTERY_STATUS, 0x0, &[])?;
            return Ok(Some(Battery {
                percent: Some(p[0].min(100)),
                charging: matches!(p[2], 1..=4),
            }));
        }
        if self.has(id::BATTERY_VOLTAGE) {
            // [voltage mV (BE u16), flags]; flag bit 7 = charging
            let p = self.call(id::BATTERY_VOLTAGE, 0x0, &[])?;
            let mv = u16::from_be_bytes([p[0], p[1]]);
            return Ok(Some(Battery {
                percent: Some(voltage_to_percent(mv)),
                charging: p[2] & 0x80 != 0,
            }));
        }
        Ok(None)
    }

    // -- 0x2201 adjustable DPI -------------------------------------------

    /// (min, max, step) for sensor 0. A discrete list reports step 0 and the
    /// list via `dpi_list`.
    pub fn dpi_range(&self) -> Result<(u16, u16, u16, Vec<u16>)> {
        let p = self.call(id::ADJUSTABLE_DPI, 0x1, &[0])?;
        Ok(parse_dpi_list(&p))
    }

    pub fn dpi(&self) -> Result<u16> {
        let p = self.call(id::ADJUSTABLE_DPI, 0x2, &[0])?;
        Ok(u16::from_be_bytes([p[1], p[2]]))
    }

    pub fn set_dpi(&self, dpi: u16) -> Result<()> {
        let [hi, lo] = dpi.to_be_bytes();
        self.call(id::ADJUSTABLE_DPI, 0x3, &[0, hi, lo])?;
        Ok(())
    }

    // -- report rate -------------------------------------------------------

    /// Supported rates in Hz, ascending.
    pub fn report_rates(&self) -> Result<Vec<u16>> {
        if self.has(id::EXTENDED_REPORT_RATE) {
            let p = self.call(id::EXTENDED_REPORT_RATE, 0x1, &[])?;
            let mask = u16::from_be_bytes([p[0], p[1]]);
            let mut v: Vec<u16> = EXT_RATES
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, r)| *r)
                .collect();
            v.sort_unstable();
            return Ok(v);
        }
        let mask = self.call(id::REPORT_RATE, 0x0, &[])?[0];
        // bit n set => (n + 1) ms supported
        let mut v: Vec<u16> = (0..8)
            .filter(|b| mask & (1 << b) != 0)
            .map(|b| 1000 / (b as u16 + 1))
            .collect();
        v.sort_unstable();
        v.dedup();
        Ok(v)
    }

    pub fn report_rate(&self) -> Result<u16> {
        if self.has(id::EXTENDED_REPORT_RATE) {
            let i = self.call(id::EXTENDED_REPORT_RATE, 0x2, &[])?[0] as usize;
            return EXT_RATES
                .get(i)
                .copied()
                .ok_or_else(|| Error::protocol("bad rate index"));
        }
        let ms = self.call(id::REPORT_RATE, 0x1, &[])?[0].max(1);
        Ok(1000 / ms as u16)
    }

    pub fn set_report_rate(&self, hz: u16) -> Result<()> {
        if self.has(id::EXTENDED_REPORT_RATE) {
            let i = EXT_RATES
                .iter()
                .position(|r| *r == hz)
                .ok_or_else(|| Error::invalid(format!("{hz} Hz")))?;
            self.call(id::EXTENDED_REPORT_RATE, 0x3, &[i as u8])?;
            return Ok(());
        }
        self.call(id::REPORT_RATE, 0x2, &[rate_to_ms(hz)?])?;
        Ok(())
    }
}

/// 0x8061 rate indices: 8 ms, 4 ms, 2 ms, 1 ms, 500 µs, 250 µs, 125 µs.
pub const EXT_RATES: [u16; 7] = [125, 250, 500, 1000, 2000, 4000, 8000];

pub fn rate_to_ms(hz: u16) -> Result<u8> {
    if hz == 0 || hz > 1000 || 1000 % hz != 0 {
        return Err(Error::invalid(format!("{hz} Hz")));
    }
    Ok((1000 / hz) as u8)
}

/// Parse a getSensorDpiList reply (`p[0]` = sensor index). Values with the
/// top three bits set (`0xE000 | step`) mark a range step between the
/// neighbouring values; otherwise the list is discrete.
pub fn parse_dpi_list(p: &[u8]) -> (u16, u16, u16, Vec<u16>) {
    let mut values = Vec::new();
    let mut step = 0;
    let mut i = 1;
    while i + 1 < p.len() {
        let v = u16::from_be_bytes([p[i], p[i + 1]]);
        if v == 0 {
            break;
        }
        if v > 0xe000 {
            step = v - 0xe000;
        } else {
            values.push(v);
        }
        i += 2;
    }
    let min = values.iter().copied().min().unwrap_or(0);
    let max = values.iter().copied().max().unwrap_or(0);
    (min, max, step, values)
}

/// Linear interpolation over a Li-ion discharge curve (as used by Solaar).
pub fn voltage_to_percent(mv: u16) -> u8 {
    const CURVE: [(u16, u8); 13] = [
        (4186, 100),
        (4067, 90),
        (3989, 80),
        (3922, 70),
        (3859, 60),
        (3811, 50),
        (3778, 40),
        (3751, 30),
        (3717, 20),
        (3671, 10),
        (3646, 5),
        (3579, 2),
        (3500, 0),
    ];
    if mv >= CURVE[0].0 {
        return 100;
    }
    for w in CURVE.windows(2) {
        let ((vh, ph), (vl, pl)) = (w[0], w[1]);
        if mv >= vl {
            let frac = (mv - vl) as f32 / (vh - vl) as f32;
            return (pl as f32 + frac * (ph - pl) as f32).round() as u8;
        }
    }
    0
}

/// 0x1004 level flags when no exact percentage is reported.
fn level_flags_percent(flags: u8) -> Option<u8> {
    match flags {
        f if f & 0x08 != 0 => Some(100), // full
        f if f & 0x04 != 0 => Some(50),  // good
        f if f & 0x02 != 0 => Some(20),  // low
        f if f & 0x01 != 0 => Some(5),   // critical
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// LED effects (0x8070 / 0x8071) and onboard-profile LED slots share a layout:
// [effect, p0..p9]. Effect ids below; parameter offsets are into p0..p9.
// ---------------------------------------------------------------------------

pub mod effect {
    pub const DISABLED: u16 = 0x00;
    pub const FIXED: u16 = 0x01;
    pub const CYCLE: u16 = 0x03;
    pub const WAVE: u16 = 0x04;
    pub const BREATHING: u16 = 0x0a;
    pub const RIPPLE: u16 = 0x0b;
}

/// Encode the 10 effect parameter bytes.
pub fn effect_params(fx: &periph_core::model::LightingEffect, brightness: u8) -> (u16, [u8; 10]) {
    use periph_core::model::{LightingEffect as L, WaveDirection};
    let mut p = [0u8; 10];
    let put_color = |p: &mut [u8; 10], c: Rgb| {
        p[0] = c.r;
        p[1] = c.g;
        p[2] = c.b;
    };
    // Intensity: 1–100 %, where 0 means 100 %.
    let intensity = if brightness >= 100 {
        0
    } else {
        brightness.max(1)
    };
    match *fx {
        L::Off => (effect::DISABLED, p),
        L::Static { color } => {
            // No intensity field: scale the colour instead.
            put_color(&mut p, color.scaled(brightness));
            (effect::FIXED, p)
        }
        L::Breathing { color, period_ms } => {
            put_color(&mut p, color);
            p[3..5].copy_from_slice(&period_ms.to_be_bytes());
            p[6] = intensity;
            (effect::BREATHING, p)
        }
        L::ColorCycle { period_ms } => {
            p[5..7].copy_from_slice(&period_ms.to_be_bytes());
            p[7] = intensity;
            (effect::CYCLE, p)
        }
        L::Wave {
            period_ms,
            direction,
        } => {
            p[6..8].copy_from_slice(&period_ms.to_be_bytes());
            // 1 = towards the right, 6 = towards the left
            p[9] = match direction {
                WaveDirection::LeftToRight => 1,
                WaveDirection::RightToLeft => 6,
            };
            (effect::WAVE, p)
        }
        L::Reactive { color, duration_ms } => {
            put_color(&mut p, color.scaled(brightness));
            p[4..6].copy_from_slice(&duration_ms.to_be_bytes());
            (effect::RIPPLE, p)
        }
    }
}

/// Decode an `[effect, p0..p9]` block back into the core model.
pub fn decode_effect(bytes: &[u8]) -> Option<(periph_core::model::LightingEffect, u8)> {
    use periph_core::model::{LightingEffect as L, WaveDirection};
    if bytes.len() < 11 {
        return None;
    }
    let p = &bytes[1..11];
    let color = Rgb::new(p[0], p[1], p[2]);
    let pct = |v: u8| if v == 0 { 100 } else { v.min(100) };
    Some(match bytes[0] as u16 {
        effect::DISABLED => (L::Off, 100),
        effect::FIXED => (L::Static { color }, 100),
        effect::BREATHING => (
            L::Breathing {
                color,
                period_ms: u16::from_be_bytes([p[3], p[4]]),
            },
            pct(p[6]),
        ),
        effect::CYCLE => (
            L::ColorCycle {
                period_ms: u16::from_be_bytes([p[5], p[6]]),
            },
            pct(p[7]),
        ),
        effect::WAVE => {
            let direction = if p[9] == 6 {
                WaveDirection::RightToLeft
            } else {
                WaveDirection::LeftToRight
            };
            (
                L::Wave {
                    period_ms: u16::from_be_bytes([p[6], p[7]]),
                    direction,
                },
                100,
            )
        }
        effect::RIPPLE => (
            L::Reactive {
                color,
                duration_ms: u16::from_be_bytes([p[4], p[5]]),
            },
            100,
        ),
        _ => return None,
    })
}

/// Name for an LED zone location code.
pub fn zone_name(location: u16) -> String {
    match location {
        0x01 => "Primary".into(),
        0x02 => "Logo".into(),
        0x03 => "Left side".into(),
        0x04 => "Right side".into(),
        0x05 => "Combined".into(),
        0x06..=0x0b => format!("Primary {}", location - 0x05),
        _ => format!("Zone {location}"),
    }
}

/// Per-key (0x8081) zone id for a HID keyboard usage.
pub fn usage_to_zone(usage: u16) -> Option<u8> {
    match usage {
        0x04..=0x63 => Some((usage - 3) as u8),
        0x65 => Some(98),
        0xe0..=0xe7 => Some((usage - 0xe0 + 104) as u8),
        _ => None,
    }
}

/// Friendly names for common 0x1b04 control ids.
pub fn control_name(cid: u16) -> String {
    match cid {
        0x0050 => "Left click".into(),
        0x0051 => "Right click".into(),
        0x0052 => "Middle click".into(),
        0x0053 => "Back".into(),
        0x0056 => "Forward".into(),
        0x005b => "Scroll left".into(),
        0x005d => "Scroll right".into(),
        0x00c3 => "Gesture button".into(),
        0x00c4 => "Smart shift".into(),
        0x00d7 => "Thumb wheel".into(),
        0x00fd => "DPI switch".into(),
        0x00ed => "DPI change".into(),
        _ => format!("Control 0x{cid:04x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use periph_core::model::{LightingEffect, WaveDirection};

    #[test]
    fn dpi_range_with_step() {
        // sensor 0: 100, step 50, 25600
        let p = [0x00, 0x00, 0x64, 0xe0, 0x32, 0x64, 0x00, 0x00, 0x00];
        assert_eq!(parse_dpi_list(&p), (100, 25600, 50, vec![100, 25600]));
    }

    #[test]
    fn dpi_discrete_list() {
        let p = [0x00, 0x01, 0x90, 0x03, 0x20, 0x06, 0x40, 0x00, 0x00];
        let (min, max, step, list) = parse_dpi_list(&p);
        assert_eq!((min, max, step), (400, 1600, 0));
        assert_eq!(list, vec![400, 800, 1600]);
    }

    #[test]
    fn voltage_curve() {
        assert_eq!(voltage_to_percent(4200), 100);
        assert_eq!(voltage_to_percent(3811), 50);
        assert_eq!(voltage_to_percent(3400), 0);
        let mid = voltage_to_percent(3835);
        assert!(mid > 50 && mid < 60);
    }

    #[test]
    fn rate_ms() {
        assert_eq!(rate_to_ms(1000).unwrap(), 1);
        assert_eq!(rate_to_ms(125).unwrap(), 8);
        assert!(rate_to_ms(2000).is_err());
        assert!(rate_to_ms(300).is_err());
    }

    #[test]
    fn effect_round_trip() {
        let cases = [
            (
                LightingEffect::Static {
                    color: Rgb::new(10, 20, 30),
                },
                100,
            ),
            (
                LightingEffect::Breathing {
                    color: Rgb::new(1, 2, 3),
                    period_ms: 4000,
                },
                60,
            ),
            (LightingEffect::ColorCycle { period_ms: 10_000 }, 100),
            (
                LightingEffect::Wave {
                    period_ms: 3000,
                    direction: WaveDirection::RightToLeft,
                },
                100,
            ),
            (LightingEffect::Off, 100),
        ];
        for (fx, b) in cases {
            let (id, p) = effect_params(&fx, b);
            let mut block = vec![id as u8];
            block.extend_from_slice(&p);
            assert_eq!(decode_effect(&block), Some((fx, b)), "{fx:?}");
        }
    }

    #[test]
    fn breathing_layout_matches_spec() {
        let (id, p) = effect_params(
            &LightingEffect::Breathing {
                color: Rgb::new(0xaa, 0xbb, 0xcc),
                period_ms: 0x1388,
            },
            50,
        );
        assert_eq!(id, effect::BREATHING);
        assert_eq!(p, [0xaa, 0xbb, 0xcc, 0x13, 0x88, 0, 50, 0, 0, 0]);
    }

    #[test]
    fn per_key_zones() {
        assert_eq!(usage_to_zone(0x04), Some(1)); // A
        assert_eq!(usage_to_zone(0x29), Some(38)); // Esc
        assert_eq!(usage_to_zone(0x63), Some(96)); // KP .
        assert_eq!(usage_to_zone(0xe0), Some(104)); // LCtrl
        assert_eq!(usage_to_zone(0xe7), Some(111)); // RGui
        assert_eq!(usage_to_zone(0x01), None);
    }
}
