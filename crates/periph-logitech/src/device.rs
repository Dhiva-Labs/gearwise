//! A HID++ 2.0 device mapped onto the core [`Device`] contract.

use std::collections::{BTreeMap, HashSet};
use std::time::{Duration, Instant};

use periph_core::device::*;
use periph_core::keys::{self, KeyboardLayout};
use periph_core::model::*;
use periph_core::{Error, Result};

use crate::features::{self, Hidpp, id};
use crate::hidpp::{Channel, DIRECT_INDEX};
use crate::onboard::{self, Profile};

/// Pending onboard edits are flushed to flash this long after the last
/// change, so dragging a slider doesn't rewrite flash on every step.
const COMMIT_DELAY: Duration = Duration::from_millis(1200);
const BATTERY_INTERVAL: Duration = Duration::from_secs(60);

/// Base id for G-keys (feature 0x8010) in the button id space; 0x1b04
/// control ids and onboard slots stay below it.
pub const GKEY_BASE: u16 = 0x1000;

struct Onboard {
    info: onboard::Info,
    index: u8,
    profile: Profile,
    rom: Profile,
    dirty_since: Option<Instant>,
}

enum Leds {
    None,
    /// 0x8070 zones; `effects[i]` = (kind, effect index in that zone).
    Color {
        zones: Vec<Vec<(EffectKind, u8)>>,
        sw_control: bool,
    },
    /// 0x8071 clusters.
    Rgb {
        zones: Vec<Vec<(EffectKind, u8)>>,
        sw_control: bool,
    },
}

/// Where a connected device lives, for building its descriptor.
pub struct Location {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
    pub serial: Option<String>,
    pub bluetooth: bool,
}

pub struct LogiDevice {
    dev: Hidpp,
    desc: DeviceDescriptor,
    caps: Capabilities,
    state: DeviceState,
    onboard: Option<Onboard>,
    leds: Leds,
    /// 0x1b04 controls that can be diverted to software.
    divertable: HashSet<u16>,
    gkey_count: u8,
    gkeys_diverted: bool,
    cids_down: Vec<u16>,
    gkey_mask: u32,
    last_battery: Instant,
}

impl LogiDevice {
    pub fn open(ch: Channel, index: u8, loc: &Location) -> Result<Self> {
        let dev = Hidpp::connect(ch, index)?;
        let name = dev.name().unwrap_or_else(|_| "Logitech device".into());
        let kind = match dev.device_type().unwrap_or(0xff) {
            0 | 2 => DeviceKind::Keyboard,
            3..=5 => DeviceKind::Mouse,
            8 => DeviceKind::Headset,
            _ => DeviceKind::Other,
        };
        let firmware = dev.firmware().ok().flatten();
        let (model_pid, unit_id) = device_info(&dev).unwrap_or((loc.product_id, None));

        let connection = if index != DIRECT_INDEX {
            Connection::Wireless
        } else if loc.bluetooth {
            Connection::Bluetooth
        } else {
            Connection::Wired
        };
        let serial = unit_id.or_else(|| loc.serial.clone());
        let desc = DeviceDescriptor {
            id: format!(
                "logitech:{:04x}:{}:{index:02x}",
                loc.product_id,
                serial.clone().unwrap_or_else(|| loc.path.clone())
            ),
            model_key: DeviceDescriptor::model_key_for(Vendor::Logitech, model_pid),
            vendor: Vendor::Logitech,
            kind,
            name,
            vendor_id: loc.vendor_id,
            product_id: model_pid,
            connection,
            firmware,
            serial,
            simulated: false,
        };

        let mut d = LogiDevice {
            dev,
            desc,
            caps: Capabilities::default(),
            state: DeviceState::default(),
            onboard: None,
            leds: Leds::None,
            divertable: HashSet::new(),
            gkey_count: 0,
            gkeys_diverted: false,
            cids_down: Vec::new(),
            gkey_mask: 0,
            last_battery: Instant::now(),
        };
        d.discover()?;
        Ok(d)
    }

    /// Build capabilities and the initial state from the feature set.
    fn discover(&mut self) -> Result<()> {
        let dev = &self.dev;

        if dev.has(id::ONBOARD_PROFILES) {
            match self.load_onboard() {
                Ok(o) => self.onboard = o,
                Err(e) => log::warn!("{}: onboard profiles unavailable: {e}", self.desc.name),
            }
        }

        // Sensor
        if dev.has(id::ADJUSTABLE_DPI) {
            let (min, max, step, list) = dev.dpi_range()?;
            let step = if step > 0 {
                step
            } else {
                smallest_gap(&list).unwrap_or(50)
            };
            self.caps.dpi = Some(DpiCaps {
                min,
                max,
                step,
                max_stages: onboard::DPI_SLOTS as u8,
                separate_xy: false,
            });
            self.state.dpi = Some(match &self.onboard {
                Some(o) => {
                    let (stages, default) = o.profile.dpi_stages();
                    let active = onboard::current_dpi_index(dev)
                        .unwrap_or(default)
                        .min(stages.len().max(1) as u8 - 1);
                    DpiState { stages, active }
                }
                None => DpiState {
                    stages: vec![DpiStage::uniform(dev.dpi()?)],
                    active: 0,
                },
            });
        }

        // Polling
        if dev.has(id::REPORT_RATE) || dev.has(id::EXTENDED_REPORT_RATE) {
            self.caps.report_rates = dev.report_rates()?;
            self.state.report_rate = match &self.onboard {
                Some(o) if !dev.has(id::EXTENDED_REPORT_RATE) => Some(o.profile.report_rate()),
                _ => dev.report_rate().ok(),
            };
        }

        self.discover_lighting()?;
        self.discover_buttons()?;

        if let Some(o) = &self.onboard {
            self.caps.onboard_profiles = o.info.profile_count;
            self.state.onboard_profile = Some(o.index);
        }
        if self.dev.has_battery() {
            self.caps.battery = true;
            self.state.battery = self.dev.battery().ok().flatten();
        }
        Ok(())
    }

    fn load_onboard(&self) -> Result<Option<Onboard>> {
        let Some(info) = onboard::Info::read(&self.dev)? else {
            return Ok(None);
        };
        onboard::ensure_onboard_mode(&self.dev)?;
        let index = onboard::current_profile(&self.dev)?.min(info.profile_count.saturating_sub(1));
        let (sector, _) = onboard::load_profile(&self.dev, &info, index)?;
        let rom = onboard::load_rom_profile(&self.dev, &info, index)?;
        Ok(Some(Onboard {
            info,
            index,
            profile: Profile(sector),
            rom: Profile(rom),
            dirty_since: None,
        }))
    }

    fn discover_lighting(&mut self) -> Result<()> {
        let dev = &self.dev;
        let mut zone_caps = Vec::new();
        let mut zones = Vec::new();
        if dev.has(id::COLOR_LED_EFFECTS) {
            let count = dev.call(id::COLOR_LED_EFFECTS, 0x0, &[])?[0];
            for z in 0..count {
                let info = dev.call(id::COLOR_LED_EFFECTS, 0x1, &[z])?;
                let location = u16::from_be_bytes([info[1], info[2]]);
                let mut effects = Vec::new();
                for e in 0..info[3] {
                    let p = dev.call(id::COLOR_LED_EFFECTS, 0x2, &[z, e])?;
                    if let Some(kind) = effect_kind(u16::from_be_bytes([p[2], p[3]])) {
                        effects.push((kind, e));
                    }
                }
                zone_caps.push((z, features::zone_name(location), effects.clone()));
                zones.push(effects);
            }
            self.leds = Leds::Color {
                zones,
                sw_control: false,
            };
        } else if dev.has(id::RGB_EFFECTS) {
            let count = dev.call(id::RGB_EFFECTS, 0x0, &[0xff, 0xff, 0x00])?[2];
            for c in 0..count {
                let info = dev.call(id::RGB_EFFECTS, 0x0, &[c, 0xff, 0x00])?;
                let location = u16::from_be_bytes([info[2], info[3]]);
                let mut effects = Vec::new();
                for e in 0..info[4] {
                    let p = dev.call(id::RGB_EFFECTS, 0x0, &[c, e, 0x00])?;
                    if let Some(kind) = effect_kind(u16::from_be_bytes([p[2], p[3]])) {
                        effects.push((kind, e));
                    }
                }
                zone_caps.push((c, features::zone_name(location), effects.clone()));
                zones.push(effects);
            }
            self.leds = Leds::Rgb {
                zones,
                sw_control: false,
            };
        }

        for (z, name, effects) in zone_caps {
            let mut kinds: Vec<EffectKind> = effects.iter().map(|(k, _)| *k).collect();
            kinds.dedup();
            if kinds.is_empty() {
                continue;
            }
            self.caps.lighting.push(LightingZoneCaps {
                id: z,
                name,
                effects: kinds,
                brightness: true,
            });
            // Current effect: from the onboard profile when there is one.
            if let Some(o) = &self.onboard {
                if (z as usize) < onboard::LED_SLOTS {
                    if let Some((effect, brightness)) =
                        features::decode_effect(o.profile.led(z as usize))
                    {
                        self.state
                            .lighting
                            .insert(z, ZoneState { effect, brightness });
                    }
                }
            }
        }

        if dev.has(id::PER_KEY_LIGHTING_V2) {
            self.caps.per_key = Some(per_key_layout(dev)?);
        }
        Ok(())
    }

    fn discover_buttons(&mut self) -> Result<()> {
        let dev = &self.dev;
        if let Some(o) = &self.onboard {
            for slot in 0..o.info.button_count as usize {
                let factory = o.rom.button_action(slot);
                let name = match slot {
                    0..=4 => factory.label(),
                    _ => format!("G{} · {}", slot + 1, factory.label()),
                };
                self.caps.buttons.push(ButtonCaps {
                    id: slot as u16,
                    name,
                    remappable: true,
                    default_action: factory.clone(),
                    hotspot: slot_hotspot(slot),
                });
                let current = o.profile.button_action(slot);
                if current != factory {
                    self.state.buttons.insert(slot as u16, current);
                }
            }
        } else if dev.has(id::REPROG_CONTROLS_V4) {
            let count = dev.call(id::REPROG_CONTROLS_V4, 0x0, &[])?[0];
            for i in 0..count {
                let p = dev.call(id::REPROG_CONTROLS_V4, 0x1, &[i])?;
                let cid = u16::from_be_bytes([p[0], p[1]]);
                // flags bit 5: control can be diverted to software
                if p[4] & 0x20 == 0 {
                    continue;
                }
                self.divertable.insert(cid);
                self.caps.buttons.push(ButtonCaps {
                    id: cid,
                    name: features::control_name(cid),
                    remappable: true,
                    default_action: cid_default(cid),
                    hotspot: cid_hotspot(cid),
                });
            }
        }
        if dev.has(id::GKEY) {
            self.gkey_count = dev.call(id::GKEY, 0x0, &[])?[0].min(32);
            for i in 0..self.gkey_count as u16 {
                self.caps.buttons.push(ButtonCaps {
                    id: GKEY_BASE + i,
                    name: format!("G{}", i + 1),
                    remappable: true,
                    default_action: ButtonAction::Default,
                    hotspot: None,
                });
            }
        }
        Ok(())
    }

    fn validate(&self, setting: &Setting) -> Result<()> {
        match setting {
            Setting::Dpi(d) => {
                let caps = self.caps.dpi.as_ref().ok_or(Error::Unsupported("DPI"))?;
                if d.stages.is_empty() || d.stages.len() > caps.max_stages as usize {
                    return Err(Error::invalid(format!("1–{} DPI stages", caps.max_stages)));
                }
                if d.active as usize >= d.stages.len() {
                    return Err(Error::invalid("active stage out of range"));
                }
                if let Some(bad) = d.stages.iter().find(|s| s.x < caps.min || s.x > caps.max) {
                    return Err(Error::invalid(format!(
                        "{} DPI is outside {}–{}",
                        bad.x, caps.min, caps.max
                    )));
                }
            }
            Setting::ReportRate(hz) if !self.caps.report_rates.contains(hz) => {
                return Err(Error::invalid(format!("{hz} Hz not supported")));
            }
            Setting::Lighting { zone, state } => {
                let z = self
                    .caps
                    .lighting
                    .iter()
                    .find(|z| z.id == *zone)
                    .ok_or(Error::invalid("lighting zone"))?;
                if !z.effects.contains(&state.effect.kind()) {
                    return Err(Error::Unsupported("effect on this zone"));
                }
            }
            Setting::PerKey(_) if self.caps.per_key.is_none() => {
                return Err(Error::Unsupported("per-key lighting"));
            }
            Setting::Button { id, .. } if !self.caps.buttons.iter().any(|b| b.id == *id) => {
                return Err(Error::invalid("button"));
            }
            Setting::OnboardProfile(p) if *p >= self.caps.onboard_profiles => {
                return Err(Error::invalid("profile slot"));
            }
            Setting::LiftOff(_) | Setting::AngleSnapping(_) | Setting::GameMode(_) => {
                return Err(Error::Unsupported("not available on this device"));
            }
            _ => {}
        }
        Ok(())
    }

    fn mark_dirty(&mut self) {
        if let Some(o) = self.onboard.as_mut() {
            o.dirty_since = Some(Instant::now());
        }
    }

    fn commit_onboard(&mut self) -> Result<()> {
        let Some(o) = self.onboard.as_mut() else {
            return Ok(());
        };
        if o.dirty_since.take().is_none() {
            return Ok(());
        }
        onboard::commit(&self.dev, &o.info, o.index, &o.profile.0)?;
        // Reloading the profile resets the DPI stage to the default; put the
        // user's active stage back.
        if let Some(d) = &self.state.dpi {
            onboard::set_current_dpi_index(&self.dev, d.active).ok();
        }
        Ok(())
    }

    fn apply_dpi(&mut self, d: &DpiState) -> Result<()> {
        let caps = self.caps.dpi.clone().ok_or(Error::Unsupported("DPI"))?;
        let stages: Vec<DpiStage> = d
            .stages
            .iter()
            .map(|s| DpiStage::uniform(caps.snap(s.x)))
            .collect();
        let active = stages[d.active as usize];
        if let Some(o) = self.onboard.as_mut() {
            o.profile.set_dpi_stages(&stages)?;
            self.mark_dirty();
        }
        // Immediate effect while the profile write is pending.
        self.dev.set_dpi(active.x)?;
        self.state.dpi = Some(DpiState {
            stages,
            active: d.active,
        });
        Ok(())
    }

    fn apply_active_stage(&mut self, i: u8) -> Result<()> {
        let d = self.state.dpi.as_ref().ok_or(Error::Unsupported("DPI"))?;
        let stage = *d.stages.get(i as usize).ok_or(Error::invalid("stage"))?;
        if self.onboard.is_some() {
            onboard::set_current_dpi_index(&self.dev, i)?;
        } else {
            self.dev.set_dpi(stage.x)?;
        }
        if let Some(d) = self.state.dpi.as_mut() {
            d.active = i;
        }
        Ok(())
    }

    fn apply_report_rate(&mut self, hz: u16) -> Result<()> {
        let use_profile =
            self.onboard.is_some() && hz <= 1000 && !self.dev.has(id::EXTENDED_REPORT_RATE);
        if use_profile {
            if let Some(o) = self.onboard.as_mut() {
                o.profile.set_report_rate(hz)?;
            }
            self.mark_dirty();
        } else {
            self.dev.set_report_rate(hz)?;
        }
        self.state.report_rate = Some(hz);
        Ok(())
    }

    fn apply_lighting(&mut self, zone: u8, zs: ZoneState) -> Result<()> {
        let (effect_id, params) = features::effect_params(&zs.effect, zs.brightness);
        if let Some(o) = self.onboard.as_mut() {
            if (zone as usize) < onboard::LED_SLOTS {
                let mut block = [0u8; 11];
                block[0] = effect_id as u8;
                block[1..].copy_from_slice(&params);
                o.profile.set_led(zone as usize, &block);
                self.mark_dirty();
            }
        }
        let onboard_mode = self.onboard.is_some();
        match &mut self.leds {
            Leds::Color { zones, sw_control } => {
                let index = effect_index(zones, zone, zs.effect.kind())?;
                if !onboard_mode && !*sw_control {
                    // LED control: let software drive the zones.
                    self.dev.call(id::COLOR_LED_EFFECTS, 0x8, &[1])?;
                    *sw_control = true;
                }
                let mut p = vec![zone, index];
                p.extend_from_slice(&params);
                let r = self.dev.call(id::COLOR_LED_EFFECTS, 0x3, &p);
                // In onboard mode the profile write is authoritative; the
                // direct write is only a preview.
                if !onboard_mode {
                    r?;
                }
            }
            Leds::Rgb { zones, sw_control } => {
                let index = effect_index(zones, zone, zs.effect.kind())?;
                if !onboard_mode {
                    if !*sw_control {
                        // SetSWControl(mode 3, NV-config flag)
                        self.dev.call(id::RGB_EFFECTS, 0x5, &[0x01, 0x03, 0x04])?;
                        *sw_control = true;
                    }
                    let mut p = vec![zone, index];
                    p.extend_from_slice(&params);
                    p.push(0x01);
                    self.dev.call(id::RGB_EFFECTS, 0x1, &p)?;
                }
            }
            Leds::None => return Err(Error::Unsupported("lighting")),
        }
        self.state.lighting.insert(zone, zs);
        Ok(())
    }

    fn apply_per_key(&mut self, colors: &BTreeMap<u16, Rgb>) -> Result<()> {
        if let Leds::Rgb { sw_control, .. } = &mut self.leds {
            if !*sw_control {
                self.dev.call(id::RGB_EFFECTS, 0x5, &[0x01, 0x03, 0x04])?;
                *sw_control = true;
            }
        }
        for (function, params) in per_key_packets(colors) {
            self.dev.call(id::PER_KEY_LIGHTING_V2, function, &params)?;
        }
        // FrameEnd: show the new frame.
        self.dev.call(id::PER_KEY_LIGHTING_V2, 0x7, &[0x00])?;
        self.state
            .per_key
            .extend(colors.iter().map(|(k, v)| (*k, *v)));
        Ok(())
    }

    fn apply_button(&mut self, id: u16, action: &ButtonAction) -> Result<()> {
        if id >= GKEY_BASE {
            let mut buttons = self.state.buttons.clone();
            if *action == ButtonAction::Default {
                buttons.remove(&id);
            } else {
                buttons.insert(id, action.clone());
            }
            let want = buttons.keys().any(|k| *k >= GKEY_BASE);
            if want != self.gkeys_diverted {
                // Divert G-keys: they then report to us instead of typing.
                self.dev.call(id::GKEY, 0x2, &[want as u8])?;
                self.gkeys_diverted = want;
            }
        } else if let Some(o) = self.onboard.as_mut() {
            o.profile
                .set_button_action(id as usize, action, &o.rom.clone())?;
            self.mark_dirty();
        } else if self.divertable.contains(&id) {
            // setCidReporting: dvalid (0x02) always, divert (0x01) when the
            // action is handled in software.
            let divert = *action != ButtonAction::Default;
            let [hi, lo] = id.to_be_bytes();
            let flags = 0x02 | divert as u8;
            self.dev
                .call(id::REPROG_CONTROLS_V4, 0x3, &[hi, lo, flags, 0, 0])?;
        } else {
            return Err(Error::Unsupported("button remapping"));
        }
        Setting::Button {
            id,
            action: action.clone(),
        }
        .apply_to(&mut self.state);
        Ok(())
    }

    fn switch_profile(&mut self, index: u8) -> Result<()> {
        self.commit_onboard()?;
        let Some(o) = self.onboard.as_mut() else {
            return Err(Error::Unsupported("onboard profiles"));
        };
        onboard::set_current_profile(&self.dev, index)?;
        let (sector, _) = onboard::load_profile(&self.dev, &o.info, index)?;
        o.rom = Profile(onboard::load_rom_profile(&self.dev, &o.info, index)?);
        o.profile = Profile(sector);
        o.index = index;
        // Rebuild everything profile-derived.
        self.caps = Capabilities::default();
        self.state = DeviceState::default();
        self.discover()
    }

    fn handle_message(&mut self, msg: &crate::hidpp::Message, events: &mut Vec<DeviceEvent>) {
        // Receiver link notifications (HID++ 1.0 sub-id 0x41).
        if msg.feature_index == 0x41 {
            let link_up = msg.params.first().is_some_and(|f| f & 0x40 == 0);
            events.push(if link_up {
                DeviceEvent::Reconnected
            } else {
                DeviceEvent::Disconnected
            });
            return;
        }
        let Some(feature) = self.dev.feature_at(msg.feature_index) else {
            return;
        };
        let p = &msg.params;
        match (feature, msg.function()) {
            (id::REPROG_CONTROLS_V4, 0) if p.len() >= 8 => {
                let now: Vec<u16> = p[..8]
                    .chunks(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .filter(|c| *c != 0)
                    .collect();
                for c in now.iter().filter(|c| !self.cids_down.contains(c)) {
                    events.push(DeviceEvent::ButtonDown(*c));
                }
                for c in self.cids_down.iter().filter(|c| !now.contains(c)) {
                    events.push(DeviceEvent::ButtonUp(*c));
                }
                self.cids_down = now;
            }
            (id::GKEY, 0) if p.len() >= 4 => {
                let mask = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
                for i in 0..32u16 {
                    let bit = 1u32 << i;
                    if mask & bit != 0 && self.gkey_mask & bit == 0 {
                        events.push(DeviceEvent::ButtonDown(GKEY_BASE + i));
                    } else if mask & bit == 0 && self.gkey_mask & bit != 0 {
                        events.push(DeviceEvent::ButtonUp(GKEY_BASE + i));
                    }
                }
                self.gkey_mask = mask;
            }
            (id::BATTERY_STATUS | id::UNIFIED_BATTERY | id::BATTERY_VOLTAGE, _) => {
                if let Ok(Some(b)) = self.dev.battery() {
                    self.state.battery = Some(b);
                    events.push(DeviceEvent::BatteryChanged(b));
                }
            }
            (id::ONBOARD_PROFILES, _) => {
                // Profile or DPI stage changed with the device's own buttons.
                if let Ok(i) = onboard::current_dpi_index(&self.dev) {
                    if let Some(d) = self.state.dpi.as_mut() {
                        d.active = i.min(d.stages.len().saturating_sub(1) as u8);
                    }
                }
                if let Ok(p) = onboard::current_profile(&self.dev) {
                    let current = self.onboard.as_ref().map(|o| o.index);
                    if current != Some(p) {
                        if let Err(e) = self.switch_profile(p) {
                            log::warn!("{}: reloading profile {p}: {e}", self.desc.name);
                        }
                    }
                }
                events.push(DeviceEvent::StateChanged);
            }
            _ => {}
        }
    }
}

impl Device for LogiDevice {
    fn descriptor(&self) -> &DeviceDescriptor {
        &self.desc
    }

    fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    fn read_state(&mut self) -> Result<DeviceState> {
        if self.onboard.is_some() {
            if let (Ok(i), Some(d)) = (
                onboard::current_dpi_index(&self.dev),
                self.state.dpi.as_mut(),
            ) {
                d.active = i.min(d.stages.len().saturating_sub(1) as u8);
            }
        }
        Ok(self.state.clone())
    }

    fn apply(&mut self, setting: &Setting) -> Result<()> {
        self.validate(setting)?;
        match setting {
            Setting::Dpi(d) => self.apply_dpi(d),
            Setting::DpiActiveStage(i) => self.apply_active_stage(*i),
            Setting::ReportRate(hz) => self.apply_report_rate(*hz),
            Setting::Lighting { zone, state } => self.apply_lighting(*zone, *state),
            Setting::PerKey(colors) => self.apply_per_key(colors),
            Setting::Button { id, action } => self.apply_button(*id, action),
            Setting::OnboardProfile(p) => self.switch_profile(*p),
            _ => Err(Error::Unsupported("not available on this device")),
        }
    }

    fn poll_events(&mut self) -> Vec<DeviceEvent> {
        let mut events = Vec::new();
        match self.dev.ch.drain(self.dev.index) {
            Ok(msgs) => {
                for m in &msgs {
                    self.handle_message(m, &mut events);
                }
            }
            Err(e) => log::debug!("{}: drain: {e}", self.desc.name),
        }
        if let Some(since) = self.onboard.as_ref().and_then(|o| o.dirty_since) {
            if since.elapsed() >= COMMIT_DELAY {
                match self.commit_onboard() {
                    Ok(()) => events.push(DeviceEvent::StateChanged),
                    Err(e) => log::warn!("{}: saving onboard profile: {e}", self.desc.name),
                }
            }
        }
        if self.caps.battery && self.last_battery.elapsed() >= BATTERY_INTERVAL {
            self.last_battery = Instant::now();
            if let Ok(Some(b)) = self.dev.battery() {
                if self.state.battery != Some(b) {
                    self.state.battery = Some(b);
                    events.push(DeviceEvent::BatteryChanged(b));
                }
            }
        }
        events
    }

    fn save_onboard(&mut self) -> Result<()> {
        self.commit_onboard()
    }
}

/// 0x0003 getDeviceInfo: (model PID, unit id as hex).
fn device_info(dev: &Hidpp) -> Result<(u16, Option<String>)> {
    let p = dev.call(id::FIRMWARE, 0x0, &[])?;
    // [entity count, unit id ×4, transport ×2, model id ×6, …]
    let unit = &p[1..5];
    let unit_id =
        (unit != [0, 0, 0, 0]).then(|| unit.iter().map(|b| format!("{b:02x}")).collect::<String>());
    let pid = p[7..13]
        .chunks(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .find(|v| *v != 0);
    Ok((pid.ok_or(Error::protocol("no model id"))?, unit_id))
}

fn effect_kind(effect_id: u16) -> Option<EffectKind> {
    Some(match effect_id {
        features::effect::DISABLED => EffectKind::Off,
        features::effect::FIXED => EffectKind::Static,
        features::effect::CYCLE => EffectKind::ColorCycle,
        features::effect::WAVE => EffectKind::Wave,
        features::effect::BREATHING => EffectKind::Breathing,
        features::effect::RIPPLE => EffectKind::Reactive,
        _ => return None,
    })
}

fn effect_index(zones: &[Vec<(EffectKind, u8)>], zone: u8, kind: EffectKind) -> Result<u8> {
    zones
        .get(zone as usize)
        .and_then(|z| z.iter().find(|(k, _)| *k == kind))
        .map(|(_, i)| *i)
        .ok_or(Error::Unsupported("effect on this zone"))
}

fn smallest_gap(list: &[u16]) -> Option<u16> {
    list.windows(2)
        .map(|w| w[1].saturating_sub(w[0]))
        .filter(|g| *g > 0)
        .min()
}

/// Full-size layout trimmed to the zones the keyboard reports (0x8081
/// getInfo returns a bitmap of supported zone ids, 14 bytes per page).
fn per_key_layout(dev: &Hidpp) -> Result<KeyboardLayout> {
    let mut bitmap = Vec::with_capacity(42);
    for page in 0..3u8 {
        let p = dev.call(id::PER_KEY_LIGHTING_V2, 0x0, &[0x00, 0x00, page])?;
        bitmap.extend_from_slice(&p[2..16]);
    }
    let supported = |zone: u8| {
        bitmap
            .get(zone as usize / 8)
            .is_some_and(|b| b >> (zone % 8) & 1 == 1)
    };
    let mut layout = keys::ansi_full();
    layout
        .keys
        .retain(|k| features::usage_to_zone(k.id).is_some_and(supported));
    let has_numpad = layout.keys.iter().any(|k| k.x >= 18.0);
    layout.name = if has_numpad {
        "Full size".into()
    } else {
        "Tenkeyless".into()
    };
    Ok(layout)
}

/// Encode per-key colors as 0x8081 packets: one-colour batches of up to 13
/// keys (fn 0x6) for colours shared by 4+ keys, the rest four keys per
/// packet (fn 0x1).
pub fn per_key_packets(colors: &BTreeMap<u16, Rgb>) -> Vec<(u8, Vec<u8>)> {
    let mut by_color: BTreeMap<(u8, u8, u8), Vec<u8>> = BTreeMap::new();
    for (usage, c) in colors {
        if let Some(z) = features::usage_to_zone(*usage) {
            by_color.entry((c.r, c.g, c.b)).or_default().push(z);
        }
    }
    let mut packets = Vec::new();
    let mut singles: Vec<u8> = Vec::new();
    for ((r, g, b), mut zones) in by_color {
        while zones.len() > 3 {
            let take: Vec<u8> = zones.drain(..zones.len().min(13)).collect();
            let mut p = vec![r, g, b];
            p.extend_from_slice(&take);
            packets.push((0x6, p));
        }
        for z in zones {
            singles.extend_from_slice(&[z, r, g, b]);
            if singles.len() == 16 {
                packets.push((0x1, std::mem::take(&mut singles)));
            }
        }
    }
    if !singles.is_empty() {
        packets.push((0x1, singles));
    }
    packets
}

fn slot_hotspot(slot: usize) -> Option<(f32, f32)> {
    Some(match slot {
        0 => (0.30, 0.20),
        1 => (0.70, 0.20),
        2 => (0.50, 0.17),
        3 => (0.10, 0.50),
        4 => (0.11, 0.40),
        _ => return None,
    })
}

fn cid_hotspot(cid: u16) -> Option<(f32, f32)> {
    match cid {
        0x50 => slot_hotspot(0),
        0x51 => slot_hotspot(1),
        0x52 => slot_hotspot(2),
        0x53 => slot_hotspot(3),
        0x56 => slot_hotspot(4),
        0x5b => Some((0.43, 0.12)),
        0x5d => Some((0.57, 0.12)),
        _ => None,
    }
}

fn cid_default(cid: u16) -> ButtonAction {
    match cid {
        0x50 => ButtonAction::Mouse(MouseButton::Left),
        0x51 => ButtonAction::Mouse(MouseButton::Right),
        0x52 => ButtonAction::Mouse(MouseButton::Middle),
        0x53 => ButtonAction::Mouse(MouseButton::Back),
        0x56 => ButtonAction::Mouse(MouseButton::Forward),
        0x5b => ButtonAction::Mouse(MouseButton::WheelLeft),
        0x5d => ButtonAction::Mouse(MouseButton::WheelRight),
        _ => ButtonAction::Default,
    }
}
