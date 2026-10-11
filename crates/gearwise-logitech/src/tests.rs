//! End-to-end tests against an emulated HID++ 2.0 firmware.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use gearwise_core::model::*;
use gearwise_core::transport::{
    BusKind, HidInterfaceInfo, MockBackend, MockOp, MockReply, MockTransport,
};
use gearwise_core::{Device, DeviceEvent, Driver, Setting};

use super::*;
use crate::features::id;
use crate::hidpp::{REPORT_LONG, REPORT_SHORT};
use crate::onboard::{crc_ccitt, crc_valid};

const SECTOR: usize = 256;

/// Emulated firmware for one device index.
#[derive(Default)]
struct Fw {
    name: String,
    device_type: u8,
    model_pid: u16,
    features: Vec<u16>,
    dpi: u16,
    rate_ms: u8,
    onboard_mode: u8,
    current_profile: u16,
    dpi_index: u8,
    flash: HashMap<u16, Vec<u8>>,
    write_target: Option<(u16, Vec<u8>)>,
    led_writes: Vec<Vec<u8>>,
    cid_reporting: HashMap<u16, u8>,
    gkey_divert: u8,
    calls: Vec<(u16, u8)>,
}

impl Fw {
    fn feature_index(&self, f: u16) -> u8 {
        self.features
            .iter()
            .position(|x| *x == f)
            .map_or(0, |i| i as u8 + 1)
    }

    /// Handle one request; `None` = error reply with that code.
    fn handle(&mut self, feat_idx: u8, function: u8, p: &[u8]) -> Result<Vec<u8>, u8> {
        if feat_idx == 0 {
            return match function {
                0 => {
                    let f = u16::from_be_bytes([p[0], p[1]]);
                    Ok(vec![self.feature_index(f), 0, 0])
                }
                1 => Ok(vec![4, 2, p[2]]),
                _ => Err(0x07),
            };
        }
        let feature = *self.features.get(feat_idx as usize - 1).ok_or(0x06u8)?;
        self.calls.push((feature, function));
        match (feature, function) {
            (id::DEVICE_NAME, 0) => Ok(vec![self.name.len() as u8]),
            (id::DEVICE_NAME, 1) => Ok(self.name.as_bytes()[p[0] as usize..]
                .iter()
                .take(16)
                .copied()
                .collect()),
            (id::DEVICE_NAME, 2) => Ok(vec![self.device_type]),
            (id::FIRMWARE, 0) => {
                let mut r = vec![1, 0xde, 0xad, 0xbe, 0xef, 0x00, 0x08];
                r.extend_from_slice(&self.model_pid.to_be_bytes());
                Ok(r)
            }
            (id::FIRMWARE, 1) => Ok(vec![0, b'U', b'1', b' ', 0x49, 0x01, 0x00, 0x06]),
            (id::ADJUSTABLE_DPI, 1) => Ok(vec![0, 0x00, 0x64, 0xe0, 0x32, 0x64, 0x00]),
            (id::ADJUSTABLE_DPI, 2) => Ok(vec![0, (self.dpi >> 8) as u8, self.dpi as u8]),
            (id::ADJUSTABLE_DPI, 3) => {
                self.dpi = u16::from_be_bytes([p[1], p[2]]);
                Ok(p[..3].to_vec())
            }
            // 1, 2, 4 and 8 ms
            (id::REPORT_RATE, 0) => Ok(vec![0b1000_1011]),
            (id::REPORT_RATE, 1) => Ok(vec![self.rate_ms]),
            (id::REPORT_RATE, 2) => {
                self.rate_ms = p[0];
                Ok(vec![])
            }
            (id::COLOR_LED_EFFECTS, 0) => Ok(vec![2]),
            // zone, location (logo / primary), 4 effects
            (id::COLOR_LED_EFFECTS, 1) => Ok(vec![p[0], 0, if p[0] == 0 { 2 } else { 1 }, 4]),
            (id::COLOR_LED_EFFECTS, 2) => {
                let ids = [0x00u16, 0x01, 0x03, 0x0a];
                let [hi, lo] = ids[p[1] as usize].to_be_bytes();
                Ok(vec![p[0], p[1], hi, lo])
            }
            (id::COLOR_LED_EFFECTS, 3) | (id::COLOR_LED_EFFECTS, 8) => {
                self.led_writes.push(p.to_vec());
                Ok(vec![])
            }
            (id::REPROG_CONTROLS_V4, 0) => Ok(vec![3]),
            (id::REPROG_CONTROLS_V4, 1) => {
                // left (not divertable), back, gesture (divertable)
                let (cid, flags) = [(0x50u16, 0x10u8), (0x53, 0x30), (0xc3, 0x30)][p[0] as usize];
                let [hi, lo] = cid.to_be_bytes();
                Ok(vec![hi, lo, 0, 0, flags])
            }
            (id::REPROG_CONTROLS_V4, 3) => {
                self.cid_reporting
                    .insert(u16::from_be_bytes([p[0], p[1]]), p[2]);
                Ok(p[..5].to_vec())
            }
            (id::GKEY, 0) => Ok(vec![3]),
            (id::GKEY, 2) => {
                self.gkey_divert = p[0];
                Ok(vec![])
            }
            (id::BATTERY_STATUS, 0) => Ok(vec![64, 50, 0]),
            (id::ONBOARD_PROFILES, 0) => Ok(vec![1, 1, 1, 3, 1, 11, 16, 0x01, 0x00, 0, 0]),
            (id::ONBOARD_PROFILES, 1) => {
                self.onboard_mode = p[0];
                Ok(vec![])
            }
            (id::ONBOARD_PROFILES, 2) => Ok(vec![self.onboard_mode]),
            (id::ONBOARD_PROFILES, 3) => {
                self.current_profile = u16::from_be_bytes([p[0], p[1]]);
                Ok(vec![])
            }
            (id::ONBOARD_PROFILES, 4) => Ok(self.current_profile.to_be_bytes().to_vec()),
            (id::ONBOARD_PROFILES, 5) => {
                let sector = u16::from_be_bytes([p[0], p[1]]);
                let offset = u16::from_be_bytes([p[2], p[3]]) as usize;
                if offset + 16 > SECTOR {
                    return Err(0x02);
                }
                let data = self
                    .flash
                    .get(&sector)
                    .cloned()
                    .unwrap_or_else(|| vec![0xff; SECTOR]);
                Ok(data[offset..offset + 16].to_vec())
            }
            (id::ONBOARD_PROFILES, 6) => {
                self.write_target = Some((u16::from_be_bytes([p[0], p[1]]), Vec::new()));
                Ok(vec![])
            }
            (id::ONBOARD_PROFILES, 7) => {
                self.write_target
                    .as_mut()
                    .ok_or(0x02u8)?
                    .1
                    .extend_from_slice(&p[..16]);
                Ok(vec![])
            }
            (id::ONBOARD_PROFILES, 8) => {
                let (sector, data) = self.write_target.take().ok_or(0x02u8)?;
                self.flash.insert(sector, data);
                Ok(vec![])
            }
            (id::ONBOARD_PROFILES, 0xb) => Ok(vec![self.dpi_index]),
            (id::ONBOARD_PROFILES, 0xc) => {
                self.dpi_index = p[0];
                Ok(vec![])
            }
            _ => Err(0x07),
        }
    }
}

/// A ROM profile: 400/800/1600/3200 DPI (default 800), 1000 Hz, buttons
/// left/right/middle/back/forward, DPI up/down, cycle; logo static red.
fn rom_profile() -> Vec<u8> {
    let mut s = vec![0xffu8; SECTOR];
    s[0] = 1;
    s[1] = 1;
    s[2] = 0;
    for (i, d) in [400u16, 800, 1600, 3200, 0].iter().enumerate() {
        s[3 + 2 * i..5 + 2 * i].copy_from_slice(&d.to_le_bytes());
    }
    let buttons: [[u8; 4]; 8] = [
        [0x80, 1, 0, 0x01],
        [0x80, 1, 0, 0x02],
        [0x80, 1, 0, 0x04],
        [0x80, 1, 0, 0x08],
        [0x80, 1, 0, 0x10],
        [0x90, 0x03, 0, 0],
        [0x90, 0x04, 0, 0],
        [0x90, 0x05, 0, 0],
    ];
    for (i, b) in buttons.iter().enumerate() {
        s[32 + 4 * i..36 + 4 * i].copy_from_slice(b);
    }
    let led = [0x01, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    s[208..219].copy_from_slice(&led);
    let crc = crc_ccitt(&s[..SECTOR - 2]);
    s[SECTOR - 2..].copy_from_slice(&crc.to_be_bytes());
    s
}

fn g502() -> Fw {
    let mut fw = Fw {
        name: "G502 HERO Gaming Mouse".into(),
        device_type: 3,
        model_pid: 0xc08b,
        features: vec![
            id::FIRMWARE,
            id::DEVICE_NAME,
            id::ADJUSTABLE_DPI,
            id::REPORT_RATE,
            id::COLOR_LED_EFFECTS,
            id::ONBOARD_PROFILES,
        ],
        dpi: 800,
        rate_ms: 1,
        onboard_mode: 2,
        current_profile: 1,
        ..Default::default()
    };
    fw.flash.insert(0x0101, rom_profile());
    fw
}

fn mx_mouse() -> Fw {
    Fw {
        name: "MX Master 3S".into(),
        device_type: 3,
        model_pid: 0xb034,
        features: vec![
            id::DEVICE_NAME,
            id::REPROG_CONTROLS_V4,
            id::BATTERY_STATUS,
            id::ADJUSTABLE_DPI,
        ],
        dpi: 1000,
        ..Default::default()
    }
}

type Shared = Arc<Mutex<HashMap<u8, Fw>>>;

fn long_reply(idx: u8, feat: u8, addr: u8, params: &[u8]) -> Vec<u8> {
    let mut v = vec![REPORT_LONG, idx, feat, addr];
    v.extend_from_slice(&params[..params.len().min(16)]);
    v.resize(20, 0);
    v
}

/// Transport whose responder dispatches to the emulated device at the
/// requested index; missing indices answer like a receiver with an empty slot.
fn transport(devs: Shared) -> MockTransport {
    MockTransport::new(move |op| {
        let MockOp::Write(d) = op else {
            return MockReply::none();
        };
        let (idx, feat, addr) = (d[1], d[2], d[3]);
        let mut devs = devs.lock().unwrap();
        let Some(fw) = devs.get_mut(&idx) else {
            return MockReply::input(vec![REPORT_SHORT, idx, 0x8f, feat, addr, 0x08, 0]);
        };
        match fw.handle(feat, addr >> 4, &d[4..]) {
            Ok(p) => MockReply::input(long_reply(idx, feat, addr, &p)),
            Err(code) => MockReply::input(long_reply(idx, 0xff, feat, &[addr, code])),
        }
    })
}

fn iface(pid: u16, usage: u16) -> HidInterfaceInfo {
    HidInterfaceInfo {
        path: format!("/dev/hidraw-{pid:04x}"),
        vendor_id: VID_LOGITECH,
        product_id: pid,
        interface_number: 1,
        usage_page: 0xff00,
        usage,
        bus: BusKind::Usb,
        ..Default::default()
    }
}

fn open(pid: u16, devs: Shared) -> Vec<Box<dyn Device>> {
    // Linux lists the hidraw node with its first (short) collection's usage;
    // Windows lists the long collection the driver claims there.
    let i = iface(pid, if cfg!(windows) { 0x0002 } else { 0x0001 });
    let d2 = devs.clone();
    let backend = MockBackend::new(vec![i.clone()], move |_| {
        Ok(Box::new(transport(d2.clone())))
    });
    let drv = LogitechDriver::new();
    assert!(drv.probe(&i));
    drv.open(&i, &backend).unwrap()
}

fn shared(entries: Vec<(u8, Fw)>) -> Shared {
    Arc::new(Mutex::new(entries.into_iter().collect()))
}

#[test]
fn probe_selects_the_hidpp_interface() {
    let mut i = iface(0xc08b, 0x0001);
    assert!(wants(&i, false));
    assert!(!wants(&i, true), "windows claims the long collection");
    i.usage = 0x0002;
    assert!(wants(&i, true));
    // Keyboard/mouse interfaces are not HID++.
    i.usage_page = 0x0001;
    assert!(!wants(&i, false));
    // dj virtual node for a paired wireless device.
    let mut v = iface(0x407f, 0x0001);
    assert!(!wants(&v, false));
    v.bus = BusKind::Bluetooth;
    assert!(wants(&v, false));
    // Other vendors.
    let mut o = iface(0xc08b, 0x0001);
    o.vendor_id = VID_RAZER;
    assert!(!wants(&o, false));
}

#[test]
fn wired_g502_discovery() {
    let devs = shared(vec![(0xff, g502())]);
    let mut list = open(0xc08b, devs.clone());
    assert_eq!(list.len(), 1);
    let d = &mut list[0];
    let desc = d.descriptor().clone();
    assert_eq!(desc.name, "G502 HERO Gaming Mouse");
    assert_eq!(desc.kind, DeviceKind::Mouse);
    assert_eq!(desc.connection, Connection::Wired);
    assert_eq!(desc.model_key, "logitech:c08b");
    assert_eq!(desc.firmware.as_deref(), Some("U1 49.01.B0006"));
    assert_eq!(desc.serial.as_deref(), Some("deadbeef"));

    let caps = d.capabilities().clone();
    assert_eq!(
        caps.dpi,
        Some(DpiCaps {
            min: 100,
            max: 25600,
            step: 50,
            max_stages: 5,
            separate_xy: false
        })
    );
    assert_eq!(caps.report_rates, vec![125, 250, 500, 1000]);
    assert_eq!(caps.onboard_profiles, 3);
    assert_eq!(caps.lighting.len(), 2);
    assert_eq!(caps.lighting[0].name, "Logo");
    assert_eq!(
        caps.lighting[0].effects,
        vec![
            EffectKind::Off,
            EffectKind::Static,
            EffectKind::ColorCycle,
            EffectKind::Breathing
        ]
    );
    assert_eq!(caps.buttons.len(), 11);
    assert_eq!(
        caps.buttons[3].default_action,
        ButtonAction::Mouse(MouseButton::Back)
    );
    assert_eq!(caps.buttons[5].name, "G6 · DPI up");

    let st = d.read_state().unwrap();
    let dpi = st.dpi.unwrap();
    assert_eq!(
        dpi.stages.iter().map(|s| s.x).collect::<Vec<_>>(),
        vec![400, 800, 1600, 3200]
    );
    assert_eq!(dpi.active, 0);
    assert_eq!(st.report_rate, Some(1000));
    assert_eq!(
        st.lighting.get(&0).map(|z| z.effect),
        Some(LightingEffect::Static {
            color: Rgb::new(0xff, 0, 0)
        })
    );
    assert!(st.buttons.is_empty(), "everything is factory");
    // Opening switched the device to onboard mode.
    assert_eq!(devs.lock().unwrap()[&0xff].onboard_mode, 1);
}

#[test]
fn onboard_edits_commit_to_flash() {
    let devs = shared(vec![(0xff, g502())]);
    let mut d = open(0xc08b, devs.clone()).remove(0);

    let stages = vec![
        DpiStage::uniform(800),
        DpiStage::uniform(1620),
        DpiStage::uniform(3200),
    ];
    d.apply(&Setting::Dpi(DpiState { stages, active: 1 }))
        .unwrap();
    d.apply(&Setting::ReportRate(500)).unwrap();
    d.apply(&Setting::Button {
        id: 3,
        action: ButtonAction::Disabled,
    })
    .unwrap();
    d.apply(&Setting::Button {
        id: 4,
        action: ButtonAction::Key {
            usage: 0x06,
            mods: Modifiers(Modifiers::LCTRL),
        },
    })
    .unwrap();
    d.apply(&Setting::Lighting {
        zone: 0,
        state: ZoneState {
            effect: LightingEffect::Breathing {
                color: Rgb::new(0, 0xd4, 0xaa),
                period_ms: 3000,
            },
            brightness: 80,
        },
    })
    .unwrap();

    {
        let fw = &devs.lock().unwrap()[&0xff];
        // Immediate DPI change, snapped to the 50-DPI step.
        assert_eq!(fw.dpi, 1600);
        // Nothing written to flash yet.
        assert!(!fw.flash.contains_key(&0x0001));
    }

    d.save_onboard().unwrap();

    let guard = devs.lock().unwrap();
    let fw = &guard[&0xff];
    let p1 = &fw.flash[&0x0001];
    assert!(crc_valid(p1));
    assert_eq!(p1[0], 2, "500 Hz = 2 ms");
    assert_eq!(p1[1], 1, "default stage");
    assert_eq!(&p1[3..11], &[0x20, 0x03, 0x40, 0x06, 0x80, 0x0c, 0, 0]);
    assert_eq!(&p1[32 + 12..32 + 16], &[0xff; 4]);
    assert_eq!(&p1[32 + 16..32 + 20], &[0x80, 0x02, 0x01, 0x06]);
    assert_eq!(&p1[208..215], &[0x0a, 0, 0xd4, 0xaa, 0x0b, 0xb8, 0]);
    // Untouched bytes survive (name region was 0xff in ROM).
    assert_eq!(p1[160], 0xff);
    // Other slots were seeded from ROM so the directory is consistent.
    assert!(crc_valid(&fw.flash[&0x0002]));
    assert!(crc_valid(&fw.flash[&0x0003]));
    let dir = &fw.flash[&0x0000];
    assert!(crc_valid(dir));
    assert_eq!(
        &dir[..16],
        &[0, 1, 1, 0, 0, 2, 1, 0, 0, 3, 1, 0, 0xff, 0xff, 0, 0]
    );
    // Profile reselected, active stage restored.
    assert_eq!(fw.current_profile, 1);
    assert_eq!(fw.dpi_index, 1);
    // Lighting was also previewed directly.
    assert_eq!(fw.led_writes.last().unwrap()[..2], [0, 3]);
    drop(guard);

    // Default restores the ROM binding.
    d.apply(&Setting::Button {
        id: 3,
        action: ButtonAction::Default,
    })
    .unwrap();
    d.save_onboard().unwrap();
    let fw = &devs.lock().unwrap()[&0xff];
    assert_eq!(
        &fw.flash[&0x0001][32 + 12..32 + 16],
        &[0x80, 0x01, 0x00, 0x08]
    );
}

#[test]
fn invalid_settings_are_rejected_before_io() {
    let devs = shared(vec![(0xff, g502())]);
    let mut d = open(0xc08b, devs.clone()).remove(0);
    let before = devs.lock().unwrap()[&0xff].calls.len();
    assert!(d.apply(&Setting::ReportRate(8000)).is_err());
    assert!(
        d.apply(&Setting::Dpi(DpiState {
            stages: vec![DpiStage::uniform(50)],
            active: 0
        }))
        .is_err()
    );
    assert!(d.apply(&Setting::GameMode(true)).is_err());
    assert!(
        d.apply(&Setting::Lighting {
            zone: 0,
            state: ZoneState {
                effect: LightingEffect::Wave {
                    period_ms: 1,
                    direction: WaveDirection::LeftToRight
                },
                brightness: 1
            }
        })
        .is_err()
    );
    assert!(
        d.apply(&Setting::Button {
            id: 2,
            action: ButtonAction::Macro("m1".into())
        })
        .is_err()
    );
    assert_eq!(devs.lock().unwrap()[&0xff].calls.len(), before);
}

#[test]
fn receiver_lists_paired_devices() {
    let devs = shared(vec![(2, mx_mouse())]);
    let list = open(0xc539, devs);
    assert_eq!(list.len(), 1);
    let desc = list[0].descriptor();
    assert_eq!(desc.connection, Connection::Wireless);
    assert_eq!(desc.name, "MX Master 3S");
    assert!(desc.id.ends_with(":02"));
}

#[test]
fn empty_receiver_returns_no_devices() {
    assert!(open(0xc539, shared(vec![])).is_empty());
}

#[test]
fn host_mode_buttons_divert_and_report_presses() {
    let devs = shared(vec![(1, mx_mouse())]);
    let mut d = open(0xc548, devs.clone()).remove(0);
    let caps = d.capabilities().clone();
    let ids: Vec<u16> = caps.buttons.iter().map(|b| b.id).collect();
    assert_eq!(ids, vec![0x53, 0xc3], "only divertable controls");
    assert!(caps.battery);
    assert_eq!(
        d.read_state().unwrap().battery,
        Some(Battery {
            percent: Some(64),
            charging: false
        })
    );

    d.apply(&Setting::Button {
        id: 0xc3,
        action: ButtonAction::Macro("m1".into()),
    })
    .unwrap();
    assert_eq!(devs.lock().unwrap()[&1].cid_reporting[&0xc3], 0x03);
    d.apply(&Setting::Button {
        id: 0xc3,
        action: ButtonAction::Default,
    })
    .unwrap();
    assert_eq!(devs.lock().unwrap()[&1].cid_reporting[&0xc3], 0x02);
    d.apply(&Setting::Button {
        id: 0x53,
        action: ButtonAction::Key {
            usage: 0x2c,
            mods: Modifiers(0),
        },
    })
    .unwrap();

    // Host-emulated DPI stages.
    d.apply(&Setting::Dpi(DpiState {
        stages: vec![DpiStage::uniform(800), DpiStage::uniform(1600)],
        active: 0,
    }))
    .unwrap();
    d.apply(&Setting::DpiActiveStage(1)).unwrap();
    assert_eq!(devs.lock().unwrap()[&1].dpi, 1600);
}

#[test]
fn diverted_press_notifications_become_events() {
    let devs = shared(vec![(1, mx_mouse())]);
    let i = iface(0xc548, 0x0001);
    let d2 = devs.clone();
    // Keep a handle to inject notifications into the transport's queue.
    let inject: Arc<Mutex<Vec<Vec<u8>>>> = Arc::default();
    let inj = inject.clone();
    let backend = MockBackend::new(vec![i.clone()], move |_| {
        let inner = Arc::new(Mutex::new(transport(d2.clone())));
        let inj = inj.clone();
        Ok(Box::new(InjectTransport { inner, inject: inj }))
    });
    let mut d = LogitechDriver::new().open(&i, &backend).unwrap().remove(0);
    let reprog = devs.lock().unwrap()[&1].feature_index(id::REPROG_CONTROLS_V4);

    inject
        .lock()
        .unwrap()
        .push(long_reply(1, reprog, 0x00, &[0x00, 0xc3, 0, 0, 0, 0, 0, 0]));
    assert_eq!(d.poll_events(), vec![DeviceEvent::ButtonDown(0xc3)]);
    inject
        .lock()
        .unwrap()
        .push(long_reply(1, reprog, 0x00, &[0; 8]));
    assert_eq!(d.poll_events(), vec![DeviceEvent::ButtonUp(0xc3)]);
    // Receiver link-loss notification.
    inject
        .lock()
        .unwrap()
        .push(vec![REPORT_SHORT, 1, 0x41, 0x04, 0x61, 0x7f, 0x40]);
    assert_eq!(d.poll_events(), vec![DeviceEvent::Disconnected]);
}

/// Wraps the emulator transport and lets a test queue unsolicited reports.
struct InjectTransport {
    inner: Arc<Mutex<MockTransport>>,
    inject: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl gearwise_core::HidTransport for InjectTransport {
    fn write(&mut self, data: &[u8]) -> gearwise_core::Result<usize> {
        self.inner.lock().unwrap().write(data)
    }
    fn read_timeout(&mut self, buf: &mut [u8], t: i32) -> gearwise_core::Result<usize> {
        let mut q = self.inject.lock().unwrap();
        if !q.is_empty() {
            let r = q.remove(0);
            buf[..r.len()].copy_from_slice(&r);
            return Ok(r.len());
        }
        drop(q);
        self.inner.lock().unwrap().read_timeout(buf, t)
    }
    fn send_feature_report(&mut self, data: &[u8]) -> gearwise_core::Result<()> {
        self.inner.lock().unwrap().send_feature_report(data)
    }
    fn get_feature_report(&mut self, buf: &mut [u8]) -> gearwise_core::Result<usize> {
        self.inner.lock().unwrap().get_feature_report(buf)
    }
}

#[test]
fn gkeys_divert_when_assigned() {
    let mut kb = Fw {
        name: "G915 TKL".into(),
        device_type: 0,
        model_pid: 0xc343,
        features: vec![id::DEVICE_NAME, id::GKEY],
        ..Default::default()
    };
    kb.features.push(id::FIRMWARE);
    let devs = shared(vec![(0xff, kb)]);
    let mut d = open(0xc343, devs.clone()).remove(0);
    assert_eq!(d.descriptor().kind, DeviceKind::Keyboard);
    assert_eq!(d.capabilities().buttons.len(), 3);
    d.apply(&Setting::Button {
        id: GKEY_BASE + 1,
        action: ButtonAction::Macro("m".into()),
    })
    .unwrap();
    assert_eq!(devs.lock().unwrap()[&0xff].gkey_divert, 1);
    d.apply(&Setting::Button {
        id: GKEY_BASE + 1,
        action: ButtonAction::Default,
    })
    .unwrap();
    assert_eq!(devs.lock().unwrap()[&0xff].gkey_divert, 0);
}

#[test]
fn per_key_packets_batch_by_colour() {
    let red = Rgb::new(255, 0, 0);
    let blue = Rgb::new(0, 0, 255);
    let mut m = BTreeMap::new();
    for u in 0x04..=0x13u16 {
        m.insert(u, red); // 16 keys
    }
    m.insert(0x29, blue);
    let packets = device::per_key_packets(&m);
    // 13 + 3 red: one batch of 13, then 3 singles; blue single packed with them.
    assert_eq!(packets[0].0, 0x6);
    assert_eq!(&packets[0].1[..4], &[255, 0, 0, 1]);
    assert_eq!(packets[0].1.len(), 3 + 13);
    let singles: Vec<&(u8, Vec<u8>)> = packets.iter().filter(|p| p.0 == 0x1).collect();
    let total: usize = singles.iter().map(|p| p.1.len() / 4).sum();
    assert_eq!(total, 4);
    assert!(singles.iter().all(|p| p.1.len() <= 16));
}
