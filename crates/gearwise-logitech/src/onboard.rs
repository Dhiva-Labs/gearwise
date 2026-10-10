//! Onboard profiles (feature 0x8100): profiles stored in the device's flash,
//! which is where G-series mice keep DPI stages, report rate, button
//! bindings and lighting.
//!
//! Profiles are edited read-modify-write: the sector is read, only the known
//! fields are changed, and bytes this driver doesn't understand are written
//! back untouched. Sector layout (memory model 1, profile formats 1–5):
//!
//! | offset | size | field                               |
//! |--------|------|-------------------------------------|
//! | 0      | 1    | report interval, ms                 |
//! | 1      | 1    | default DPI index                   |
//! | 2      | 1    | DPI-shift ("sniper") index          |
//! | 3      | 10   | 5 × DPI, little-endian, 0 = unused  |
//! | 32     | 64   | 16 × button binding (4 bytes)       |
//! | 96     | 64   | 16 × G-shift binding                |
//! | 160    | 48   | name                                |
//! | 208    | 22   | 2 × LED effect (11 bytes)           |
//! | 230    | 22   | 2 × alternate LED effect            |
//! | n-2    | 2    | CRC-CCITT (big-endian) of the rest  |

use gearwise_core::model::{ButtonAction, DpiFunction, DpiStage, Modifiers, MouseButton};
use gearwise_core::{Error, Result};

use crate::features::{Hidpp, id};

pub const DPI_SLOTS: usize = 5;
pub const BUTTON_SLOTS: usize = 16;
pub const LED_SLOTS: usize = 2;

const OFF_RATE: usize = 0;
const OFF_DEFAULT_DPI: usize = 1;
const OFF_SHIFT_DPI: usize = 2;
const OFF_DPI: usize = 3;
const OFF_BUTTONS: usize = 32;
const OFF_LEDS: usize = 208;
const OFF_ALT_LEDS: usize = 230;

const ROM_BASE: u16 = 0x0100;
const DIRECTORY: u16 = 0x0000;

const MODE_ONBOARD: u8 = 0x01;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub profile_count: u8,
    pub rom_profile_count: u8,
    pub button_count: u8,
    pub sector_size: u16,
}

impl Info {
    /// getProfilesDescr; `None` if the memory/profile/macro format is one
    /// this driver can't edit safely.
    pub fn read(dev: &Hidpp) -> Result<Option<Self>> {
        let p = dev.call(id::ONBOARD_PROFILES, 0x0, &[])?;
        let (memory_model, profile_format, macro_format) = (p[0], p[1], p[2]);
        let sector_size = u16::from_be_bytes([p[7], p[8]]);
        let ok = memory_model == 1
            && (1..=5).contains(&profile_format)
            && macro_format == 1
            && sector_size >= 256;
        if !ok {
            log::info!(
                "onboard profiles: unsupported layout (memory {memory_model}, profile {profile_format}, macro {macro_format}, sector {sector_size})"
            );
            return Ok(None);
        }
        Ok(Some(Self {
            profile_count: p[3],
            rom_profile_count: p[4],
            button_count: p[5].min(BUTTON_SLOTS as u8),
            sector_size,
        }))
    }
}

/// CRC-CCITT (poly 0x1021, seed 0xFFFF) as used by HID++ onboard memory.
pub fn crc_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xffff;
    for &b in data {
        let temp = (crc >> 8) ^ b as u16;
        crc <<= 8;
        let mut quick = temp ^ (temp >> 4);
        crc ^= quick;
        quick <<= 5;
        crc ^= quick;
        quick <<= 7;
        crc ^= quick;
    }
    crc
}

pub fn crc_valid(sector: &[u8]) -> bool {
    let n = sector.len();
    n >= 2 && crc_ccitt(&sector[..n - 2]) == u16::from_be_bytes([sector[n - 2], sector[n - 1]])
}

pub fn read_sector(dev: &Hidpp, sector: u16, size: u16) -> Result<Vec<u8>> {
    let size = size as usize;
    let mut data = vec![0u8; size];
    let mut offset = 0usize;
    while offset < size {
        // Reads past size-16 are rejected, so the last chunk overlaps.
        let at = offset.min(size - 16);
        let mut params = [0u8; 4];
        params[..2].copy_from_slice(&sector.to_be_bytes());
        params[2..].copy_from_slice(&(at as u16).to_be_bytes());
        let chunk = dev.call(id::ONBOARD_PROFILES, 0x5, &params)?;
        data[at..at + 16].copy_from_slice(&chunk[..16]);
        offset = at + 16;
    }
    Ok(data)
}

/// Write a whole sector, recomputing its trailing CRC.
pub fn write_sector(dev: &Hidpp, sector: u16, data: &mut [u8]) -> Result<()> {
    let n = data.len();
    let crc = crc_ccitt(&data[..n - 2]);
    data[n - 2..].copy_from_slice(&crc.to_be_bytes());

    let mut start = [0u8; 6];
    start[..2].copy_from_slice(&sector.to_be_bytes());
    start[4..].copy_from_slice(&(n as u16).to_be_bytes());
    dev.call(id::ONBOARD_PROFILES, 0x6, &start)?;
    for chunk in data.chunks(16) {
        let mut buf = [0u8; 16];
        buf[..chunk.len()].copy_from_slice(chunk);
        dev.call(id::ONBOARD_PROFILES, 0x7, &buf)?;
    }
    dev.call(id::ONBOARD_PROFILES, 0x8, &[])?;
    Ok(())
}

pub fn ensure_onboard_mode(dev: &Hidpp) -> Result<()> {
    if dev.call(id::ONBOARD_PROFILES, 0x2, &[])?[0] != MODE_ONBOARD {
        dev.call(id::ONBOARD_PROFILES, 0x1, &[MODE_ONBOARD])?;
    }
    Ok(())
}

/// Active profile, 0-based.
pub fn current_profile(dev: &Hidpp) -> Result<u8> {
    let p = dev.call(id::ONBOARD_PROFILES, 0x4, &[])?;
    // Reply is the profile's sector address; ROM profiles are 0x01xx.
    Ok((u16::from_be_bytes([p[0], p[1]]) & 0xff).saturating_sub(1) as u8)
}

pub fn set_current_profile(dev: &Hidpp, index: u8) -> Result<()> {
    dev.call(id::ONBOARD_PROFILES, 0x3, &(index as u16 + 1).to_be_bytes())?;
    Ok(())
}

pub fn current_dpi_index(dev: &Hidpp) -> Result<u8> {
    Ok(dev.call(id::ONBOARD_PROFILES, 0xb, &[])?[0])
}

pub fn set_current_dpi_index(dev: &Hidpp, index: u8) -> Result<()> {
    dev.call(id::ONBOARD_PROFILES, 0xc, &[index])?;
    Ok(())
}

fn rom_sector(info: &Info, index: u8) -> u16 {
    if index < info.rom_profile_count {
        ROM_BASE + index as u16 + 1
    } else {
        ROM_BASE + 1
    }
}

/// Read profile `index` (0-based). Returns the sector bytes and whether they
/// came from ROM because the user copy was never written or is corrupt.
pub fn load_profile(dev: &Hidpp, info: &Info, index: u8) -> Result<(Vec<u8>, bool)> {
    if directory(dev, info)?.is_some() {
        if let Ok(s) = read_sector(dev, index as u16 + 1, info.sector_size) {
            if crc_valid(&s) {
                return Ok((s, false));
            }
        }
    }
    Ok((
        read_sector(dev, rom_sector(info, index), info.sector_size)?,
        true,
    ))
}

pub fn load_rom_profile(dev: &Hidpp, info: &Info, index: u8) -> Result<Vec<u8>> {
    read_sector(dev, rom_sector(info, index), info.sector_size)
}

/// Per-profile enabled flags from the directory sector, or `None` if the
/// directory has never been written (device runs ROM profiles).
fn directory(dev: &Hidpp, info: &Info) -> Result<Option<Vec<bool>>> {
    let d = match read_sector(dev, DIRECTORY, info.sector_size) {
        Ok(d) => d,
        // Some mice reject reads of a never-written sector.
        Err(Error::InvalidValue(_)) => return Ok(None),
        Err(e) => return Err(e),
    };
    if !crc_valid(&d) {
        return Ok(None);
    }
    let mut enabled = vec![false; info.profile_count as usize];
    for (i, e) in enabled.iter_mut().enumerate() {
        let entry = &d[i * 4..i * 4 + 4];
        if entry[..2] == [0xff, 0xff] {
            break;
        }
        *e = entry[2] != 0;
    }
    Ok(Some(enabled))
}

fn write_directory(dev: &Hidpp, info: &Info, enabled: &[bool]) -> Result<()> {
    let mut d = vec![0xffu8; info.sector_size as usize];
    let mut at = 0;
    for (i, e) in enabled.iter().enumerate() {
        d[at..at + 4].copy_from_slice(&[0x00, i as u8 + 1, *e as u8, 0x00]);
        at += 4;
    }
    d[at..at + 4].copy_from_slice(&[0xff, 0xff, 0x00, 0x00]);
    write_sector(dev, DIRECTORY, &mut d)
}

/// Persist `sector` as profile `index` and make the device reload it.
///
/// If the device has only ever run ROM profiles, every slot is first seeded
/// with its ROM profile so the directory never points at empty sectors.
pub fn commit(dev: &Hidpp, info: &Info, index: u8, sector: &[u8]) -> Result<()> {
    let mut enabled = match directory(dev, info)? {
        Some(e) => e,
        None => {
            for i in 0..info.profile_count {
                if i != index {
                    let mut rom = load_rom_profile(dev, info, i)?;
                    write_sector(dev, i as u16 + 1, &mut rom)?;
                }
            }
            vec![true; info.profile_count as usize]
        }
    };
    let mut data = sector.to_vec();
    write_sector(dev, index as u16 + 1, &mut data)?;
    if let Some(e) = enabled.get_mut(index as usize) {
        *e = true;
    }
    write_directory(dev, info, &enabled)?;
    set_current_profile(dev, index)
}

// ---------------------------------------------------------------------------
// Field access on a profile sector
// ---------------------------------------------------------------------------

/// A profile sector being edited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile(pub Vec<u8>);

impl Profile {
    pub fn report_rate(&self) -> u16 {
        1000 / self.0[OFF_RATE].max(1) as u16
    }

    pub fn set_report_rate(&mut self, hz: u16) -> Result<()> {
        self.0[OFF_RATE] = crate::features::rate_to_ms(hz)?;
        Ok(())
    }

    /// Enabled DPI stages and the default stage index among them.
    pub fn dpi_stages(&self) -> (Vec<DpiStage>, u8) {
        let mut stages = Vec::new();
        let mut default = 0u8;
        for slot in 0..DPI_SLOTS {
            let v =
                u16::from_le_bytes([self.0[OFF_DPI + 2 * slot], self.0[OFF_DPI + 2 * slot + 1]]);
            if v == 0 || v == 0xffff {
                continue;
            }
            if slot == self.0[OFF_DEFAULT_DPI] as usize {
                default = stages.len() as u8;
            }
            stages.push(DpiStage::uniform(v));
        }
        (stages, default)
    }

    /// Replace the DPI stages. The profile's default stage is kept (clamped
    /// to the new stage count); the active stage is a RAM setting.
    pub fn set_dpi_stages(&mut self, stages: &[DpiStage]) -> Result<()> {
        if stages.is_empty() || stages.len() > DPI_SLOTS {
            return Err(Error::invalid(format!("1–{DPI_SLOTS} DPI stages")));
        }
        for slot in 0..DPI_SLOTS {
            let v = stages.get(slot).map_or(0, |s| s.x);
            self.0[OFF_DPI + 2 * slot..OFF_DPI + 2 * slot + 2].copy_from_slice(&v.to_le_bytes());
        }
        if self.0[OFF_DEFAULT_DPI] as usize >= stages.len() {
            self.0[OFF_DEFAULT_DPI] = 0;
        }
        if self.0[OFF_SHIFT_DPI] as usize >= stages.len() {
            self.0[OFF_SHIFT_DPI] = 0;
        }
        Ok(())
    }

    fn shift_dpi(&self) -> u16 {
        let slot = self.0[OFF_SHIFT_DPI] as usize % DPI_SLOTS;
        u16::from_le_bytes([self.0[OFF_DPI + 2 * slot], self.0[OFF_DPI + 2 * slot + 1]])
    }

    pub fn binding(&self, slot: usize) -> [u8; 4] {
        let o = OFF_BUTTONS + 4 * slot;
        [self.0[o], self.0[o + 1], self.0[o + 2], self.0[o + 3]]
    }

    pub fn set_binding(&mut self, slot: usize, b: [u8; 4]) {
        let o = OFF_BUTTONS + 4 * slot;
        self.0[o..o + 4].copy_from_slice(&b);
    }

    pub fn button_action(&self, slot: usize) -> ButtonAction {
        decode_binding(self.binding(slot), self.shift_dpi())
    }

    /// Set a button from the core model. `rom` supplies the factory binding
    /// for [`ButtonAction::Default`].
    pub fn set_button_action(
        &mut self,
        slot: usize,
        action: &ButtonAction,
        rom: &Profile,
    ) -> Result<()> {
        if slot >= BUTTON_SLOTS {
            return Err(Error::invalid("button slot"));
        }
        let binding = match action {
            ButtonAction::Default => rom.binding(slot),
            ButtonAction::Dpi(DpiFunction::Shift(dpi)) => {
                // Point the shift index at the closest stage.
                let (stages, _) = self.dpi_stages();
                if let Some((i, _)) = stages
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, s)| s.x.abs_diff(*dpi))
                {
                    self.0[OFF_SHIFT_DPI] = i as u8;
                }
                SPECIAL_SHIFT_DPI
            }
            other => encode_binding(other)?,
        };
        self.set_binding(slot, binding);
        Ok(())
    }

    /// Raw `[effect, params…]` LED block for slot `i`.
    pub fn led(&self, i: usize) -> &[u8] {
        &self.0[OFF_LEDS + 11 * i..OFF_LEDS + 11 * i + 11]
    }

    /// Set an LED slot (and its alternate, as the device shows either).
    pub fn set_led(&mut self, i: usize, block: &[u8; 11]) {
        self.0[OFF_LEDS + 11 * i..OFF_LEDS + 11 * i + 11].copy_from_slice(block);
        // Keep the alternate slot in step, unless the device leaves it unused
        // (all 0xff) — then it stays untouched.
        let alt = &mut self.0[OFF_ALT_LEDS + 11 * i..OFF_ALT_LEDS + 11 * i + 11];
        if alt.iter().any(|b| *b != 0xff) {
            alt.copy_from_slice(block);
        }
    }
}

// Binding types.
const TYPE_MACRO: u8 = 0x00;
const TYPE_HID: u8 = 0x80;
const TYPE_SPECIAL: u8 = 0x90;
const TYPE_DISABLED: u8 = 0xff;
// HID subtypes.
const HID_MOUSE: u8 = 0x01;
const HID_KEYBOARD: u8 = 0x02;
const HID_CONSUMER: u8 = 0x03;
// Special functions.
const SP_TILT_LEFT: u8 = 0x01;
const SP_TILT_RIGHT: u8 = 0x02;
const SP_NEXT_DPI: u8 = 0x03;
const SP_PREV_DPI: u8 = 0x04;
const SP_CYCLE_DPI: u8 = 0x05;
const SP_SHIFT_DPI: u8 = 0x07;
const SP_CYCLE_PROFILE: u8 = 0x0a;

const SPECIAL_SHIFT_DPI: [u8; 4] = [TYPE_SPECIAL, SP_SHIFT_DPI, 0, 0];

/// Mouse buttons as bits of the HID button bitmap (big-endian u16).
fn mouse_bit(b: MouseButton) -> Option<u16> {
    Some(match b {
        MouseButton::Left => 1 << 0,
        MouseButton::Right => 1 << 1,
        MouseButton::Middle => 1 << 2,
        MouseButton::Back => 1 << 3,
        MouseButton::Forward => 1 << 4,
        MouseButton::WheelLeft | MouseButton::WheelRight => return None,
    })
}

pub fn encode_binding(action: &ButtonAction) -> Result<[u8; 4]> {
    Ok(match action {
        ButtonAction::Disabled => [TYPE_DISABLED; 4],
        ButtonAction::Mouse(MouseButton::WheelLeft) => [TYPE_SPECIAL, SP_TILT_LEFT, 0, 0],
        ButtonAction::Mouse(MouseButton::WheelRight) => [TYPE_SPECIAL, SP_TILT_RIGHT, 0, 0],
        ButtonAction::Mouse(b) => {
            let [hi, lo] = mouse_bit(*b).expect("wheel handled above").to_be_bytes();
            [TYPE_HID, HID_MOUSE, hi, lo]
        }
        ButtonAction::Key { usage, mods } => [TYPE_HID, HID_KEYBOARD, mods.0, *usage],
        ButtonAction::Media(u) => {
            let [hi, lo] = u.to_be_bytes();
            [TYPE_HID, HID_CONSUMER, hi, lo]
        }
        ButtonAction::Dpi(DpiFunction::Up) => [TYPE_SPECIAL, SP_NEXT_DPI, 0, 0],
        ButtonAction::Dpi(DpiFunction::Down) => [TYPE_SPECIAL, SP_PREV_DPI, 0, 0],
        ButtonAction::Dpi(DpiFunction::Cycle) => [TYPE_SPECIAL, SP_CYCLE_DPI, 0, 0],
        ButtonAction::Dpi(DpiFunction::Shift(_)) => SPECIAL_SHIFT_DPI,
        ButtonAction::ProfileCycle => [TYPE_SPECIAL, SP_CYCLE_PROFILE, 0, 0],
        ButtonAction::Macro(id) => match parse_onboard_macro(id) {
            Some((page, offset)) => [TYPE_MACRO, page, 0, offset],
            None => {
                return Err(Error::Unsupported(
                    "software macros on a device in onboard-memory mode",
                ));
            }
        },
        ButtonAction::Default => return Err(Error::invalid("default needs the ROM binding")),
    })
}

pub fn decode_binding(b: [u8; 4], shift_dpi: u16) -> ButtonAction {
    match b[0] {
        TYPE_DISABLED => ButtonAction::Disabled,
        TYPE_HID => match b[1] {
            HID_MOUSE => {
                let bits = u16::from_be_bytes([b[2], b[3]]);
                let button = [
                    MouseButton::Left,
                    MouseButton::Right,
                    MouseButton::Middle,
                    MouseButton::Back,
                    MouseButton::Forward,
                ]
                .into_iter()
                .find(|m| mouse_bit(*m) == Some(bits));
                button.map_or(ButtonAction::Default, ButtonAction::Mouse)
            }
            HID_KEYBOARD => ButtonAction::Key {
                usage: b[3],
                mods: Modifiers(b[2]),
            },
            HID_CONSUMER => ButtonAction::Media(u16::from_be_bytes([b[2], b[3]])),
            _ => ButtonAction::Default,
        },
        TYPE_SPECIAL => match b[1] {
            SP_TILT_LEFT => ButtonAction::Mouse(MouseButton::WheelLeft),
            SP_TILT_RIGHT => ButtonAction::Mouse(MouseButton::WheelRight),
            SP_NEXT_DPI => ButtonAction::Dpi(DpiFunction::Up),
            SP_PREV_DPI => ButtonAction::Dpi(DpiFunction::Down),
            SP_CYCLE_DPI => ButtonAction::Dpi(DpiFunction::Cycle),
            SP_SHIFT_DPI => ButtonAction::Dpi(DpiFunction::Shift(shift_dpi)),
            SP_CYCLE_PROFILE => ButtonAction::ProfileCycle,
            _ => ButtonAction::Default,
        },
        TYPE_MACRO => ButtonAction::Macro(format!("onboard:{:02x}:{:02x}", b[1], b[3])),
        _ => ButtonAction::Default,
    }
}

/// Macros already stored in onboard memory are shown as `onboard:PP:OO`
/// (page, offset) so they survive a read/write round trip.
fn parse_onboard_macro(id: &str) -> Option<(u8, u8)> {
    let rest = id.strip_prefix("onboard:")?;
    let (p, o) = rest.split_once(':')?;
    Some((
        u8::from_str_radix(p, 16).ok()?,
        u8::from_str_radix(o, 16).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_ccitt_false() {
        // CRC-16/CCITT-FALSE check value.
        assert_eq!(crc_ccitt(b"123456789"), 0x29b1);
    }

    #[test]
    fn crc_validation() {
        let mut s = vec![0x5au8; 256];
        let c = crc_ccitt(&s[..254]);
        s[254..].copy_from_slice(&c.to_be_bytes());
        assert!(crc_valid(&s));
        s[10] ^= 1;
        assert!(!crc_valid(&s));
    }

    fn blank() -> Profile {
        let mut p = Profile(vec![0xff; 256]);
        p.0[OFF_RATE] = 1;
        p.0[OFF_DEFAULT_DPI] = 1;
        p.0[OFF_SHIFT_DPI] = 0;
        p.set_dpi_stages(&[
            DpiStage::uniform(400),
            DpiStage::uniform(800),
            DpiStage::uniform(1600),
        ])
        .unwrap();
        p
    }

    #[test]
    fn dpi_stages_round_trip_little_endian() {
        let p = blank();
        assert_eq!(&p.0[3..9], &[0x90, 0x01, 0x20, 0x03, 0x40, 0x06]);
        assert_eq!(&p.0[9..13], &[0, 0, 0, 0]);
        let (stages, default) = p.dpi_stages();
        assert_eq!(
            stages.iter().map(|s| s.x).collect::<Vec<_>>(),
            vec![400, 800, 1600]
        );
        assert_eq!(default, 1);
        assert_eq!(p.report_rate(), 1000);
    }

    #[test]
    fn bindings_round_trip() {
        let actions = [
            ButtonAction::Disabled,
            ButtonAction::Mouse(MouseButton::Left),
            ButtonAction::Mouse(MouseButton::Forward),
            ButtonAction::Mouse(MouseButton::WheelLeft),
            ButtonAction::Key {
                usage: 0x04,
                mods: Modifiers(Modifiers::LCTRL | Modifiers::LSHIFT),
            },
            ButtonAction::Media(0x00cd),
            ButtonAction::Dpi(DpiFunction::Up),
            ButtonAction::Dpi(DpiFunction::Cycle),
            ButtonAction::ProfileCycle,
            ButtonAction::Macro("onboard:03:10".into()),
        ];
        for a in actions {
            let b = encode_binding(&a).unwrap();
            assert_eq!(decode_binding(b, 0), a, "{a:?}");
        }
        assert_eq!(
            encode_binding(&ButtonAction::Mouse(MouseButton::Back)).unwrap(),
            [0x80, 0x01, 0x00, 0x08]
        );
        assert_eq!(
            encode_binding(&ButtonAction::Media(0x00e9)).unwrap(),
            [0x80, 0x03, 0x00, 0xe9]
        );
        assert!(matches!(
            encode_binding(&ButtonAction::Macro("m1".into())),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn default_restores_rom_and_shift_picks_closest_stage() {
        let rom = {
            let mut r = blank();
            r.set_binding(3, [0x80, 0x01, 0x00, 0x08]);
            r
        };
        let mut p = blank();
        p.set_button_action(3, &ButtonAction::Disabled, &rom)
            .unwrap();
        assert_eq!(p.button_action(3), ButtonAction::Disabled);
        p.set_button_action(3, &ButtonAction::Default, &rom)
            .unwrap();
        assert_eq!(p.button_action(3), ButtonAction::Mouse(MouseButton::Back));

        p.set_button_action(5, &ButtonAction::Dpi(DpiFunction::Shift(700)), &rom)
            .unwrap();
        assert_eq!(p.0[OFF_SHIFT_DPI], 1);
        assert_eq!(
            p.button_action(5),
            ButtonAction::Dpi(DpiFunction::Shift(800))
        );
    }

    #[test]
    fn led_updates_alternate_slot_only_when_used() {
        let mut p = blank();
        let block = [0x01, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0];
        p.set_led(1, &block);
        assert_eq!(p.led(1), &block);
        assert_eq!(
            &p.0[OFF_ALT_LEDS + 11..OFF_ALT_LEDS + 22],
            &[0xff; 11],
            "unused alt slot untouched"
        );
        p.0[OFF_ALT_LEDS..OFF_ALT_LEDS + 11].fill(0);
        p.set_led(0, &block);
        assert_eq!(&p.0[OFF_ALT_LEDS..OFF_ALT_LEDS + 11], &block);
    }

    #[test]
    fn default_stage_is_preserved_and_clamped() {
        let mut p = blank();
        p.0[OFF_DEFAULT_DPI] = 2;
        p.set_dpi_stages(&[
            DpiStage::uniform(400),
            DpiStage::uniform(800),
            DpiStage::uniform(900),
        ])
        .unwrap();
        assert_eq!(p.0[OFF_DEFAULT_DPI], 2);
        p.set_dpi_stages(&[DpiStage::uniform(400)]).unwrap();
        assert_eq!(p.0[OFF_DEFAULT_DPI], 0);
    }
}
