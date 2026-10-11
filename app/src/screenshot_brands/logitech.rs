//! Logitech demo devices and tour steps for the model-accurate art (see `super`).

use gearwise_core::model::{
    Battery, ButtonAction, ButtonCaps, Connection, DeviceKind, DpiCaps, DpiFunction, DpiStage,
    DpiState, EffectKind, Modifiers, MouseButton, Rgb, Vendor, media,
};

use crate::screenshot::Step;
use crate::screenshot_demo::{Spec, btn, lit, mouse_btn, mouse_like, zone};
use crate::state::Tab;
use crate::worker::DeviceSnapshot;

const MX_MASTER: &str = "logitech:b034";
const SUPERLIGHT2: &str = "logitech:c09b";
const G305: &str = "logitech:4074";
const G703: &str = "logitech:4070";
const G903: &str = "logitech:4067";
const G502X: &str = "logitech:c099";
const G600: &str = "logitech:c24a";
const MX_ANYWHERE: &str = "logitech:b037";

pub(super) fn devices() -> Vec<DeviceSnapshot> {
    vec![
        mx_master_3s(),
        superlight_2(),
        g305(),
        g703(),
        g903(),
        g502_x_plus(),
        g600(),
        mx_anywhere_3s(),
    ]
}

pub(super) fn steps() -> Vec<Step> {
    vec![
        Step::device(
            "30-mx-master-assignments",
            MX_MASTER,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(0xd7),
        ),
        Step::device(
            "31-light-mx-master-assignments",
            MX_MASTER,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(0xc3),
        )
        .light(),
        Step::device("32-superlight-2", SUPERLIGHT2, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(3)
        }),
        Step::device("33-g305", G305, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(5)
        }),
        Step::device("34-g703", G703, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(5)
        }),
        Step::device("35-g903", G903, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(7)
        }),
        Step::device("36-g502-x-plus", G502X, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(6)
        }),
        Step::device("37-g600", G600, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(11)
        }),
        Step::device("38-mx-anywhere-3s", MX_ANYWHERE, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(0xc4)
        }),
    ]
}

/// The driver lists a mouse in onboard mode slot by slot: the first five are named after their
/// factory action (left, right, middle, back, forward) and carry the driver's hotspots, the
/// rest are "G6 · ...". `list` is (name, factory action); ids are the slot numbers.
fn slots(list: &[(&str, ButtonAction)]) -> Vec<ButtonCaps> {
    let hotspots = [
        (0.30, 0.20),
        (0.70, 0.20),
        (0.50, 0.17),
        (0.10, 0.50),
        (0.11, 0.40),
    ];
    list.iter()
        .enumerate()
        .map(|(i, (name, action))| btn(i as u16, name, action.clone(), hotspots.get(i).copied()))
        .collect()
}

/// The five buttons every onboard-mode mouse starts with.
fn primary() -> Vec<(&'static str, ButtonAction)> {
    vec![
        ("Left click", mouse_btn(MouseButton::Left)),
        ("Right click", mouse_btn(MouseButton::Right)),
        ("Middle click", mouse_btn(MouseButton::Middle)),
        ("Back", mouse_btn(MouseButton::Back)),
        ("Forward", mouse_btn(MouseButton::Forward)),
    ]
}

const EFFECTS: [EffectKind; 4] = [
    EffectKind::Off,
    EffectKind::Static,
    EffectKind::Breathing,
    EffectKind::ColorCycle,
];

/// Finish a snapshot the way a Lightspeed / wired G mouse looks to the app: DPI range, battery
/// when wireless, one onboard profile.
fn gaming(
    mut snap: DeviceSnapshot,
    firmware: &str,
    dpi_max: u16,
    battery: Option<u8>,
) -> DeviceSnapshot {
    snap.descriptor.connection = if battery.is_some() {
        Connection::Wireless
    } else {
        Connection::Wired
    };
    snap.descriptor.firmware = Some(firmware.into());
    snap.capabilities.dpi = Some(DpiCaps {
        min: 100,
        max: dpi_max,
        step: 50,
        max_stages: 5,
        separate_xy: false,
    });
    snap.state.dpi = Some(DpiState {
        stages: [400, 800, 1600, 3200, 6400].map(DpiStage::uniform).into(),
        active: 2,
    });
    snap.capabilities.onboard_profiles = 1;
    snap.state.onboard_profile = Some(0);
    snap.capabilities.battery = battery.is_some();
    snap.state.battery = battery.map(|percent| Battery {
        percent: Some(percent),
        charging: false,
    });
    snap
}

/// The G Pro X Superlight 2 over Lightspeed: five buttons and the DPI key under the shell,
/// one lit zone (the palm logo).
fn superlight_2() -> DeviceSnapshot {
    let mut list = primary();
    list.push(("G6 · DPI cycle", ButtonAction::Dpi(DpiFunction::Cycle)));
    let mut snap = mouse_like(Spec {
        name: "G Pro X Superlight 2",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xc09b,
        buttons: slots(&list),
        zones: vec![zone(0, "Logo", &EFFECTS)],
        lighting: vec![(0, lit(Rgb::new(255, 77, 94)))],
        custom: vec![
            (
                3,
                ButtonAction::Key {
                    usage: 0x06,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
            (4, ButtonAction::Media(media::PLAY_PAUSE)),
        ],
    });
    snap.capabilities.report_rates = vec![125, 250, 500, 1000];
    gaming(snap, "RRP 1.0.12", 32000, Some(72))
}

/// The MX Master 3S as the Logitech driver lists it over a Bolt receiver: only the controls
/// that can be diverted to software (no left / right click), no lighting, one DPI value and no
/// report-rate choice, but a battery.
fn mx_master_3s() -> DeviceSnapshot {
    let ctrl = |usage| ButtonAction::Key {
        usage,
        mods: Modifiers(Modifiers::LCTRL),
    };
    let mouse = ButtonAction::Mouse;
    let buttons = [
        (
            0x52,
            "Middle click",
            mouse(MouseButton::Middle),
            Some((0.50, 0.17)),
        ),
        (0x53, "Back", mouse(MouseButton::Back), Some((0.10, 0.50))),
        (
            0x56,
            "Forward",
            mouse(MouseButton::Forward),
            Some((0.11, 0.40)),
        ),
        (0xc3, "Gesture button", ButtonAction::Default, None),
        (0xc4, "Smart shift", ButtonAction::Default, None),
        (0xd7, "Thumb wheel", ButtonAction::Default, None),
    ]
    .into_iter()
    .map(|(id, name, action, hotspot)| btn(id, name, action, hotspot))
    .collect();
    let mut snap = mouse_like(Spec {
        name: "MX Master 3S",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xb034,
        buttons,
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![
            (0x53, ctrl(0x1d)),
            (0x56, ctrl(0x1c)),
            (0xc3, ButtonAction::Media(media::PLAY_PAUSE)),
        ],
    });
    snap.descriptor.connection = Connection::Wireless;
    snap.descriptor.firmware = Some("RBM21 25.00.0013".into());
    snap.capabilities.dpi = Some(DpiCaps {
        min: 200,
        max: 8000,
        step: 50,
        max_stages: 1,
        separate_xy: false,
    });
    snap.capabilities.report_rates.clear();
    snap.capabilities.onboard_profiles = 0;
    snap.capabilities.battery = true;
    snap.state.dpi = Some(DpiState {
        stages: vec![DpiStage::uniform(1000)],
        active: 0,
    });
    snap.state.report_rate = None;
    snap.state.onboard_profile = None;
    snap.state.battery = Some(Battery {
        percent: Some(84),
        charging: false,
    });
    snap
}

/// The G305 over Lightspeed: five buttons plus the DPI key behind the wheel, no lighting.
fn g305() -> DeviceSnapshot {
    let mut list = primary();
    list.push(("G6 · DPI cycle", ButtonAction::Dpi(DpiFunction::Cycle)));
    let mut snap = mouse_like(Spec {
        name: "G305 LIGHTSPEED",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0x4074,
        buttons: slots(&list),
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![
            (
                3,
                ButtonAction::Key {
                    usage: 0x1d,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
            (5, ButtonAction::Media(media::MUTE)),
        ],
    });
    snap.capabilities.report_rates = vec![125, 250, 500, 1000];
    gaming(snap, "RQR 8.1.5", 12000, Some(91))
}

/// The G703 over Lightspeed: five buttons plus the DPI key behind the wheel; the wheel's light
/// strip ("Primary") and the palm logo are lit.
fn g703() -> DeviceSnapshot {
    let mut list = primary();
    list.push(("G6 · DPI cycle", ButtonAction::Dpi(DpiFunction::Cycle)));
    let mut snap = mouse_like(Spec {
        name: "G703 LIGHTSPEED",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0x4070,
        buttons: slots(&list),
        zones: vec![zone(0, "Primary", &EFFECTS), zone(1, "Logo", &EFFECTS)],
        lighting: vec![
            (0, lit(Rgb::new(0, 212, 170))),
            (1, lit(Rgb::new(0, 184, 252))),
        ],
        custom: vec![
            (
                3,
                ButtonAction::Key {
                    usage: 0x05,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
            (4, ButtonAction::Media(media::NEXT)),
        ],
    });
    snap.capabilities.report_rates = vec![125, 250, 500, 1000];
    gaming(snap, "RQR 12.3.4", 25600, Some(64))
}

/// The G903 over Lightspeed: the five standard buttons, the DPI keys on the spine, a second
/// Back / Forward pair on the right flank and wheel tilt; the DPI lights ("Primary") and the
/// palm logo are lit.
fn g903() -> DeviceSnapshot {
    let mut list = primary();
    list.extend([
        ("G6 · DPI cycle", ButtonAction::Dpi(DpiFunction::Cycle)),
        ("G7 · DPI down", ButtonAction::Dpi(DpiFunction::Down)),
        ("G8 · DPI up", ButtonAction::Dpi(DpiFunction::Up)),
        ("G9 · Back (right)", mouse_btn(MouseButton::Back)),
        ("G10 · Forward (right)", mouse_btn(MouseButton::Forward)),
        ("G11 · Scroll right", mouse_btn(MouseButton::WheelRight)),
        ("G12 · Scroll left", mouse_btn(MouseButton::WheelLeft)),
    ]);
    let mut snap = mouse_like(Spec {
        name: "G903 LIGHTSPEED",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0x4067,
        buttons: slots(&list),
        zones: vec![zone(0, "Primary", &EFFECTS), zone(1, "Logo", &EFFECTS)],
        lighting: vec![
            (0, lit(Rgb::new(0, 184, 252))),
            (1, lit(Rgb::new(0, 184, 252))),
        ],
        custom: vec![
            (7, ButtonAction::Media(media::VOLUME_UP)),
            (6, ButtonAction::Media(media::VOLUME_DOWN)),
            (
                8,
                ButtonAction::Key {
                    usage: 0x1d,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
        ],
    });
    snap.capabilities.report_rates = vec![125, 250, 500, 1000];
    gaming(snap, "RQR 14.2.1", 25600, Some(88))
}

/// The G502 X PLUS over Lightspeed: the HERO-style eleven slots (G6 DPI shift under the thumb,
/// G7 / G8 on the flank, G9 behind the wheel, tilt) and one lit zone for the LIGHTSYNC strip.
fn g502_x_plus() -> DeviceSnapshot {
    let mut list = primary();
    list.extend([
        (
            "G6 · DPI shift (1200)",
            ButtonAction::Dpi(DpiFunction::Shift(1200)),
        ),
        ("G7 · DPI down", ButtonAction::Dpi(DpiFunction::Down)),
        ("G8 · DPI up", ButtonAction::Dpi(DpiFunction::Up)),
        ("G9 · Profile cycle", ButtonAction::ProfileCycle),
        ("G10 · Scroll right", mouse_btn(MouseButton::WheelRight)),
        ("G11 · Scroll left", mouse_btn(MouseButton::WheelLeft)),
    ]);
    let mut snap = mouse_like(Spec {
        name: "G502 X PLUS LIGHTSPEED",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xc099,
        buttons: slots(&list),
        zones: vec![zone(0, "Primary", &EFFECTS)],
        lighting: vec![(0, lit(Rgb::new(168, 85, 247)))],
        custom: vec![
            (
                3,
                ButtonAction::Key {
                    usage: 0x05,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
            (4, ButtonAction::Media(media::PLAY_PAUSE)),
            (6, ButtonAction::Media(media::VOLUME_DOWN)),
            (7, ButtonAction::Media(media::VOLUME_UP)),
        ],
    });
    snap.capabilities.report_rates = vec![125, 250, 500, 1000];
    gaming(snap, "RQR 49.1.6", 25600, Some(78))
}

/// The G600 MMO: three clicks, wheel tilt, G-shift under the ring finger, G7 / G8 behind the
/// wheel and the twelve thumb buttons ("Side button 1" to "12", G9 to G20) with the factory
/// number-row keys. Wired, no lighting.
fn g600() -> DeviceSnapshot {
    let key = |usage| ButtonAction::Key {
        usage,
        mods: Modifiers(0),
    };
    let mut named: Vec<(String, ButtonAction)> = vec![
        ("Left click".into(), mouse_btn(MouseButton::Left)),
        ("Right click".into(), mouse_btn(MouseButton::Right)),
        ("Middle click".into(), mouse_btn(MouseButton::Middle)),
        ("G4 · Scroll left".into(), mouse_btn(MouseButton::WheelLeft)),
        (
            "G5 · Scroll right".into(),
            mouse_btn(MouseButton::WheelRight),
        ),
        (
            "G6 · DPI shift (800)".into(),
            ButtonAction::Dpi(DpiFunction::Shift(800)),
        ),
        ("G7 · DPI down".into(), ButtonAction::Dpi(DpiFunction::Down)),
        ("G8 · DPI up".into(), ButtonAction::Dpi(DpiFunction::Up)),
    ];
    // 1 to 9, 0, minus, equals: the number row each thumb button types out of the box.
    let number_row = [
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2d, 0x2e,
    ];
    named.extend(
        number_row
            .into_iter()
            .enumerate()
            .map(|(i, usage)| (format!("Side button {}", i + 1), key(usage))),
    );
    let buttons = named
        .iter()
        .enumerate()
        .map(|(i, (name, action))| {
            let hotspot = [(0.30, 0.20), (0.70, 0.20), (0.50, 0.17)].get(i).copied();
            btn(i as u16, name, action.clone(), hotspot)
        })
        .collect();
    let mut snap = mouse_like(Spec {
        name: "G600 MMO",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xc24a,
        buttons,
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![
            (
                11,
                ButtonAction::Key {
                    usage: 0x09,
                    mods: Modifiers(Modifiers::LSHIFT),
                },
            ),
            (14, ButtonAction::Media(media::MUTE)),
        ],
    });
    snap.capabilities.report_rates = vec![125, 250, 500, 1000];
    gaming(snap, "RAF 1.04", 8200, None)
}

/// The MX Anywhere 3S over Bolt: only the controls that can be diverted to software (the wheel
/// click, the two thumb buttons and the mode-shift button), no lighting, one DPI value, battery.
fn mx_anywhere_3s() -> DeviceSnapshot {
    let buttons = [
        (
            0x52,
            "Middle click",
            mouse_btn(MouseButton::Middle),
            Some((0.50, 0.17)),
        ),
        (
            0x53,
            "Back",
            mouse_btn(MouseButton::Back),
            Some((0.10, 0.50)),
        ),
        (
            0x56,
            "Forward",
            mouse_btn(MouseButton::Forward),
            Some((0.11, 0.40)),
        ),
        (0xc4, "Smart shift", ButtonAction::Default, None),
    ]
    .into_iter()
    .map(|(id, name, action, hotspot)| btn(id, name, action, hotspot))
    .collect();
    let mut snap = mouse_like(Spec {
        name: "MX Anywhere 3S",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xb037,
        buttons,
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![
            (
                0x53,
                ButtonAction::Key {
                    usage: 0x50,
                    mods: Modifiers(Modifiers::LALT),
                },
            ),
            (
                0x56,
                ButtonAction::Key {
                    usage: 0x4f,
                    mods: Modifiers(Modifiers::LALT),
                },
            ),
            (0xc4, ButtonAction::Media(media::PLAY_PAUSE)),
        ],
    });
    snap.descriptor.connection = Connection::Wireless;
    snap.descriptor.firmware = Some("RBM18 22.00.0010".into());
    snap.capabilities.dpi = Some(DpiCaps {
        min: 200,
        max: 8000,
        step: 50,
        max_stages: 1,
        separate_xy: false,
    });
    snap.capabilities.report_rates.clear();
    snap.capabilities.onboard_profiles = 0;
    snap.capabilities.battery = true;
    snap.state.dpi = Some(DpiState {
        stages: vec![DpiStage::uniform(1000)],
        active: 0,
    });
    snap.state.report_rate = None;
    snap.state.onboard_profile = None;
    snap.state.battery = Some(Battery {
        percent: Some(92),
        charging: false,
    });
    snap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::illustration::{Design, MouseArt};

    #[test]
    fn mx_master_3s_lists_only_the_divertable_controls() {
        let mx = mx_master_3s();
        assert_eq!(mx.descriptor.model_key, MX_MASTER);
        let ids: Vec<u16> = mx.capabilities.buttons.iter().map(|b| b.id).collect();
        assert_eq!(ids, [0x52, 0x53, 0x56, 0xc3, 0xc4, 0xd7]);
        assert!(mx.capabilities.lighting.is_empty());
        assert!(mx.capabilities.battery && mx.state.battery.is_some());
        assert!(mx.capabilities.report_rates.is_empty());
        // Every control has a part on the drawing, without falling back to a hotspot.
        let design = Design::for_name(&mx.descriptor.name);
        let rect =
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 300.0 / design.aspect()));
        let placed = MouseArt::new(rect, design).place(&mx.capabilities.buttons);
        assert_eq!(placed.len(), 6);
        assert!(placed.iter().all(|p| p.role.is_some()));
    }

    /// How many of a demo mouse's buttons the drawing places on a part by role (the rest have no
    /// part, so they would fall back to a hotspot or be left out) and the drawing's id.
    fn placed_on_parts(snap: &DeviceSnapshot) -> (usize, Option<&'static str>) {
        let design = Design::for_name(&snap.descriptor.name);
        let rect =
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 300.0 / design.aspect()));
        let placed = MouseArt::new(rect, design).place(&snap.capabilities.buttons);
        let id = match design {
            Design::Model(m) => Some(m.id()),
            Design::Generic(_) => None,
        };
        (placed.iter().filter(|p| p.role.is_some()).count(), id)
    }

    #[test]
    fn every_demo_mouse_has_its_own_drawing_and_unique_key() {
        let mice = devices();
        let mut keys = std::collections::HashSet::new();
        for snap in crate::screenshot_demo::extra_devices().iter().chain(
            &gearwise_core::demo::devices()
                .into_iter()
                .map(|mut d| DeviceSnapshot {
                    descriptor: d.descriptor().clone(),
                    capabilities: d.capabilities().clone(),
                    state: d.read_state().unwrap_or_default(),
                })
                .collect::<Vec<_>>(),
        ) {
            assert!(
                keys.insert(snap.descriptor.model_key.clone()),
                "{} is used by two demo devices",
                snap.descriptor.model_key
            );
        }
        let ids = [
            ("G Pro X Superlight 2", "superlight-2"),
            ("G305 LIGHTSPEED", "g305"),
            ("G703 LIGHTSPEED", "g703"),
            ("G903 LIGHTSPEED", "g903"),
            ("G502 X PLUS LIGHTSPEED", "g502-x"),
            ("G600 MMO", "g600"),
            ("MX Anywhere 3S", "mx-anywhere"),
        ];
        for (name, id) in ids {
            let snap = mice
                .iter()
                .find(|d| d.descriptor.name.starts_with(name))
                .unwrap_or_else(|| panic!("no demo device {name}"));
            assert_eq!(placed_on_parts(snap).1, Some(id), "{name}");
        }
    }

    #[test]
    fn demo_buttons_land_on_the_parts_the_real_controls_are() {
        let mice = devices();
        let get = |name: &str| {
            mice.iter()
                .find(|d| d.descriptor.name.starts_with(name))
                .unwrap()
        };
        // (device, buttons it lists, buttons that sit on a part). The Superlight's DPI key is on
        // its underside, so the top view has no part for it.
        for (name, listed, on_parts) in [
            ("G Pro X Superlight 2", 6, 5),
            ("G305 LIGHTSPEED", 6, 6),
            ("G703 LIGHTSPEED", 6, 6),
            ("G903 LIGHTSPEED", 12, 12),
            ("G502 X PLUS LIGHTSPEED", 11, 11),
            ("G600 MMO", 20, 20),
            ("MX Anywhere 3S", 4, 4),
        ] {
            let snap = get(name);
            assert_eq!(snap.capabilities.buttons.len(), listed, "{name}");
            assert_eq!(placed_on_parts(snap).0, on_parts, "{name}");
        }
    }

    #[test]
    fn lighting_zones_match_what_each_mouse_lights() {
        let mice = devices();
        let zones = |name: &str| -> Vec<String> {
            mice.iter()
                .find(|d| d.descriptor.name.starts_with(name))
                .unwrap()
                .capabilities
                .lighting
                .iter()
                .map(|z| z.name.clone())
                .collect()
        };
        assert_eq!(zones("G Pro X Superlight 2"), ["Logo"]);
        assert_eq!(zones("G703 LIGHTSPEED"), ["Primary", "Logo"]);
        assert_eq!(zones("G903 LIGHTSPEED"), ["Primary", "Logo"]);
        assert_eq!(zones("G502 X PLUS LIGHTSPEED"), ["Primary"]);
        for unlit in ["G305 LIGHTSPEED", "G600 MMO", "MX Anywhere 3S"] {
            assert!(zones(unlit).is_empty(), "{unlit}");
        }
    }

    #[test]
    fn steps_only_name_devices_this_file_creates() {
        for step in steps() {
            assert!(step.targets(&devices()), "{}", step.name());
        }
    }
}
