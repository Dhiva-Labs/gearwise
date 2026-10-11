//! Demo data for the screenshot tour and tests: sample profiles and macros, and simulated
//! devices that mirror real products (the user's G502 HERO and K55 RGB PRO among them).

use gearwise_core::device::{Capabilities, DeviceDescriptor, DeviceState};
use gearwise_core::keys::{self, KeyCell, KeyboardLayout};
use gearwise_core::model::{
    ButtonAction, ButtonCaps, Connection, DeviceKind, DpiFunction, EffectKind, LightingEffect,
    LightingZoneCaps, Modifiers, MouseButton, Rgb, VID_CORSAIR, Vendor, ZoneState, media,
};
use gearwise_core::profile::{Library, Macro, MacroMode, MacroStep, Profile};

use crate::worker::DeviceSnapshot;

/// Profiles and macros so those pages have something to show.
pub fn sample_library() -> Library {
    let mut lib = Library::starter();
    lib.prefs.demo_devices = true;
    let mut fps = Profile::new("FPS competitive");
    fps.apps = vec!["valorant.exe".into(), "cs2".into()];
    let mut studio = Profile::new("Creative work");
    studio.apps = vec!["blender".into(), "photoshop.exe".into()];
    lib.profiles.extend([fps, studio]);

    let mut reload = Macro::new("Reload and inspect");
    reload.steps = vec![
        MacroStep::KeyTap {
            usage: 0x15,
            mods: Modifiers(0),
        },
        MacroStep::Delay(180),
        MacroStep::KeyTap {
            usage: 0x09,
            mods: Modifiers(0),
        },
        MacroStep::Delay(60),
        MacroStep::MouseDown(MouseButton::Left),
        MacroStep::Delay(40),
        MacroStep::MouseUp(MouseButton::Left),
    ];
    let mut rapid = Macro::new("Rapid click");
    rapid.mode = MacroMode::WhileHeld;
    rapid.steps = vec![
        MacroStep::MouseDown(MouseButton::Left),
        MacroStep::Delay(25),
        MacroStep::MouseUp(MouseButton::Left),
        MacroStep::Delay(25),
    ];
    let mut sig = Macro::new("Email signature");
    sig.steps = vec![
        MacroStep::Text("Best regards,".into()),
        MacroStep::KeyTap {
            usage: 0x28,
            mods: Modifiers(0),
        },
        MacroStep::Text("Alex".into()),
    ];
    lib.macros.extend([reload, rapid, sig]);
    lib
}

// --- Simulated extras ---------------------------------------------------------------

fn btn(id: u16, name: &str, default: ButtonAction, hotspot: Option<(f32, f32)>) -> ButtonCaps {
    ButtonCaps {
        id,
        name: name.into(),
        remappable: true,
        default_action: default,
        hotspot,
    }
}

fn mouse_btn(b: MouseButton) -> ButtonAction {
    ButtonAction::Mouse(b)
}

fn lit(color: Rgb) -> ZoneState {
    ZoneState {
        effect: LightingEffect::Static { color },
        brightness: 90,
    }
}

fn zone(id: u8, name: &str, effects: &[EffectKind]) -> LightingZoneCaps {
    LightingZoneCaps {
        keys: Vec::new(),
        id,
        name: name.into(),
        effects: effects.to_vec(),
        brightness: true,
    }
}

/// The first core demo device as a template for the simulated extras.
fn template() -> (DeviceDescriptor, Capabilities, DeviceState) {
    let mut base = gearwise_core::demo::devices().remove(0);
    let state = base.read_state().unwrap_or_default();
    (
        base.descriptor().clone(),
        base.capabilities().clone(),
        state,
    )
}

struct Spec<'a> {
    name: &'a str,
    vendor: Vendor,
    kind: DeviceKind,
    pid: u16,
    buttons: Vec<ButtonCaps>,
    zones: Vec<LightingZoneCaps>,
    lighting: Vec<(u8, ZoneState)>,
    custom: Vec<(u16, ButtonAction)>,
}

fn mouse_like(spec: Spec) -> DeviceSnapshot {
    let (d0, c0, s0) = template();
    let mut descriptor = DeviceDescriptor {
        id: format!("demo:{}:{:04x}", spec.vendor.key(), spec.pid),
        model_key: DeviceDescriptor::model_key_for(spec.vendor, spec.pid),
        vendor: spec.vendor,
        kind: spec.kind,
        name: format!("{} (demo)", spec.name),
        ..d0
    };
    descriptor.product_id = spec.pid;
    let mut capabilities = Capabilities {
        buttons: spec.buttons,
        lighting: spec.zones,
        ..c0
    };
    if spec.kind == DeviceKind::Headset {
        capabilities.dpi = None;
        capabilities.report_rates.clear();
        capabilities.onboard_profiles = 0;
    }
    let mut state = DeviceState {
        lighting: spec.lighting.into_iter().collect(),
        ..s0
    };
    state.buttons = spec.custom.into_iter().collect();
    DeviceSnapshot {
        descriptor,
        capabilities,
        state,
    }
}

/// The user's G502 HERO as the Logitech driver reports it: eleven buttons (hotspots only on
/// the first five, like the driver) and the DPI strip ("Primary") and palm logo zones.
fn g502_hero() -> DeviceSnapshot {
    let hotspots = [
        Some((0.30, 0.20)),
        Some((0.70, 0.20)),
        Some((0.50, 0.17)),
        Some((0.10, 0.50)),
        Some((0.11, 0.40)),
    ];
    let actions = [
        ("Left click", mouse_btn(MouseButton::Left)),
        ("Right click", mouse_btn(MouseButton::Right)),
        ("Middle click", mouse_btn(MouseButton::Middle)),
        ("Back", mouse_btn(MouseButton::Back)),
        ("Forward", mouse_btn(MouseButton::Forward)),
        (
            "G6 · DPI shift (1200)",
            ButtonAction::Dpi(DpiFunction::Shift(1200)),
        ),
        ("G7 · DPI down", ButtonAction::Dpi(DpiFunction::Down)),
        ("G8 · DPI up", ButtonAction::Dpi(DpiFunction::Up)),
        ("G9 · Profile cycle", ButtonAction::ProfileCycle),
        ("G10 · Scroll right", mouse_btn(MouseButton::WheelRight)),
        ("G11 · Scroll left", mouse_btn(MouseButton::WheelLeft)),
    ];
    let buttons = actions
        .into_iter()
        .enumerate()
        .map(|(i, (name, action))| btn(i as u16, name, action, hotspots.get(i).copied().flatten()))
        .collect();
    let effects = [
        EffectKind::Off,
        EffectKind::Static,
        EffectKind::Breathing,
        EffectKind::ColorCycle,
    ];
    let mut snap = mouse_like(Spec {
        name: "G502 HERO Gaming Mouse",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xc08b,
        buttons,
        zones: vec![zone(0, "Primary", &effects), zone(1, "Logo", &effects)],
        lighting: vec![
            (0, lit(Rgb::new(0, 184, 252))),
            (1, lit(Rgb::new(0, 212, 170))),
        ],
        custom: vec![
            (
                3,
                ButtonAction::Key {
                    usage: 0x05,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
            (4, ButtonAction::Media(media::PLAY_PAUSE)),
            (7, ButtonAction::Media(media::VOLUME_UP)),
            (9, ButtonAction::Media(media::NEXT)),
            (10, ButtonAction::Media(media::PREVIOUS)),
        ],
    });
    // Wired, like the real one.
    snap.descriptor.connection = Connection::Wired;
    snap.descriptor.firmware = Some("U1 27.03.B0010".into());
    snap.state.battery = None;
    snap.capabilities.battery = false;
    snap.capabilities.onboard_profiles = 1;
    snap.state.onboard_profile = Some(0);
    snap
}

const G_KEYS: u16 = 0x100;
const MEDIA_KEYS: u16 = 0x110;

fn key_cell(id: u16, label: &str, x: f32, y: f32) -> KeyCell {
    KeyCell {
        id,
        label: label.into(),
        x,
        y,
        w: 1.0,
        h: 1.0,
    }
}

/// The K55 RGB PRO board and which keys each of its five zones lights, as the Corsair driver
/// reports them (duplicated here because the driver keeps this table private).
fn k55_physical() -> (KeyboardLayout, Vec<(u8, Vec<u16>)>) {
    const SHIFT: f32 = 1.5;
    let mut layout = keys::ansi_full();
    for k in &mut layout.keys {
        k.x += SHIFT;
        if k.id == 0xe7 {
            k.label = "Fn".into();
        }
    }
    for (i, y) in [0.0, 1.25, 2.25, 3.25, 4.25, 5.25].iter().enumerate() {
        layout
            .keys
            .push(key_cell(G_KEYS + i as u16, &format!("G{}", i + 1), 0.0, *y));
    }
    for (i, label) in ["Stop", "Prev", "Play", "Next"].iter().enumerate() {
        layout.keys.push(key_cell(
            MEDIA_KEYS + i as u16,
            label,
            18.5 + SHIFT + i as f32,
            0.0,
        ));
    }
    layout.name = "K55 RGB PRO".into();
    let zone1: Vec<u16> = (0..6)
        .map(|i| G_KEYS + i)
        .chain([
            0x29, 0x3a, 0x3b, 0x35, 0x1e, 0x1f, 0x20, 0x2b, 0x14, 0x1a, 0x08, 0x39, 0x04, 0x16,
            0xe1, 0x1d, 0x1b, 0xe0, 0xe3, 0xe2,
        ])
        .collect();
    let zone2 = vec![
        0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x15, 0x17,
        0x1c, 0x18, 0x0c, 0x12, 0x07, 0x09, 0x0a, 0x0b, 0x0d, 0x0e, 0x0f, 0x06, 0x19, 0x05, 0x11,
        0x10, 0x2c,
    ];
    let zone3 = vec![
        0x42, 0x43, 0x44, 0x45, 0x2d, 0x2e, 0x2a, 0x13, 0x2f, 0x30, 0x31, 0x33, 0x34, 0x28, 0x36,
        0x37, 0x38, 0xe5, 0xe6, 0xe7, 0x65, 0xe4,
    ];
    let zone4 = (0x46..=0x52).collect();
    let zone5 = (0x53..=0x63)
        .chain((0..4).map(|i| MEDIA_KEYS + i))
        .collect();
    (
        layout,
        vec![(1, zone1), (2, zone2), (3, zone3), (4, zone4), (5, zone5)],
    )
}

/// The user's K55 RGB PRO: a zone-lit board (no per-key RGB) with a macro column and media keys.
fn k55() -> DeviceSnapshot {
    let (d0, _, _) = template();
    let (layout, zone_keys) = k55_physical();
    let effects = [EffectKind::Off, EffectKind::Static];
    let mut zones = vec![LightingZoneCaps {
        brightness: false,
        ..zone(0, "Win-lock indicator", &effects)
    }];
    for (id, keys) in zone_keys {
        zones.push(LightingZoneCaps {
            keys,
            ..zone(id, &format!("Zone {id}"), &effects)
        });
    }
    let colors = [
        (255, 40, 60),
        (255, 160, 0),
        (255, 210, 40),
        (0, 212, 170),
        (0, 140, 255),
        (150, 70, 255),
    ];
    let lighting = colors
        .iter()
        .enumerate()
        .map(|(i, (r, g, b))| (i as u8, lit(Rgb::new(*r, *g, *b))))
        .collect();
    DeviceSnapshot {
        descriptor: DeviceDescriptor {
            id: "demo:corsair:1ba4".into(),
            model_key: DeviceDescriptor::model_key_for(Vendor::Corsair, 0x1ba4),
            vendor: Vendor::Corsair,
            kind: DeviceKind::Keyboard,
            name: "K55 RGB PRO (demo)".into(),
            vendor_id: VID_CORSAIR,
            product_id: 0x1ba4,
            connection: Connection::Wired,
            firmware: Some("1.9.4".into()),
            serial: None,
            ..d0
        },
        capabilities: Capabilities {
            report_rates: vec![125, 250, 500, 1000],
            lighting: zones,
            layout: Some(layout),
            ..Default::default()
        },
        state: DeviceState {
            report_rate: Some(1000),
            lighting,
            ..Default::default()
        },
    }
}

/// Extra simulated devices (never sent to the worker) that show every silhouette.
pub(crate) fn extra_devices() -> Vec<DeviceSnapshot> {
    let slim_buttons = vec![
        btn(0, "Left click", mouse_btn(MouseButton::Left), None),
        btn(1, "Right click", mouse_btn(MouseButton::Right), None),
        btn(2, "Middle click", mouse_btn(MouseButton::Middle), None),
        btn(3, "Back", mouse_btn(MouseButton::Back), None),
        btn(4, "Forward", mouse_btn(MouseButton::Forward), None),
        btn(5, "DPI button", ButtonAction::Dpi(DpiFunction::Cycle), None),
    ];
    let mut naga_buttons = slim_buttons[..3].to_vec();
    naga_buttons.push(btn(3, "DPI up", ButtonAction::Dpi(DpiFunction::Up), None));
    naga_buttons.push(btn(
        4,
        "DPI down",
        ButtonAction::Dpi(DpiFunction::Down),
        None,
    ));
    naga_buttons.extend((1..=12u16).map(|n| {
        btn(
            4 + n,
            &format!("Side button {n}"),
            ButtonAction::Key {
                usage: 0x1d + n as u8,
                mods: Modifiers(0),
            },
            None,
        )
    }));
    let five = &EffectKind::ALL[..5];

    vec![
        g502_hero(),
        mouse_like(Spec {
            name: "G Pro X Superlight",
            vendor: Vendor::Logitech,
            kind: DeviceKind::Mouse,
            pid: 0xc094,
            buttons: slim_buttons.clone(),
            zones: vec![zone(0, "Logo", five)],
            lighting: vec![(0, lit(Rgb::new(255, 77, 94)))],
            custom: vec![(
                3,
                ButtonAction::Key {
                    usage: 0x06,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            )],
        }),
        mouse_like(Spec {
            name: "Naga Pro",
            vendor: Vendor::Razer,
            kind: DeviceKind::Mouse,
            pid: 0x0067,
            buttons: naga_buttons,
            zones: vec![zone(0, "Logo", five), zone(1, "Scroll wheel", five)],
            lighting: vec![
                (0, lit(Rgb::new(68, 214, 44))),
                (1, lit(Rgb::new(68, 214, 44))),
            ],
            custom: vec![
                (8, ButtonAction::Disabled),
                (10, ButtonAction::Media(media::MUTE)),
            ],
        }),
        mouse_like(Spec {
            name: "DeathAdder V3",
            vendor: Vendor::Razer,
            kind: DeviceKind::Mouse,
            pid: 0x00b6,
            buttons: slim_buttons,
            zones: vec![zone(0, "Logo", five), zone(1, "Scroll wheel", five)],
            lighting: vec![
                (0, lit(Rgb::new(139, 92, 246))),
                (
                    1,
                    ZoneState {
                        effect: LightingEffect::Breathing {
                            color: Rgb::new(139, 92, 246),
                            period_ms: 3000,
                        },
                        brightness: 100,
                    },
                ),
            ],
            custom: vec![],
        }),
        k55(),
        mouse_like(Spec {
            name: "HS80 RGB Wireless",
            vendor: Vendor::Corsair,
            kind: DeviceKind::Headset,
            pid: 0x0a64,
            buttons: Vec::new(),
            zones: vec![zone(0, "Logo", five)],
            lighting: vec![(0, lit(Rgb::new(255, 204, 0)))],
            custom: vec![],
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_pid(pid: u16) -> DeviceSnapshot {
        extra_devices()
            .into_iter()
            .find(|d| d.descriptor.product_id == pid)
            .unwrap()
    }

    #[test]
    fn g502_hero_mirrors_the_real_mouse() {
        let hero = by_pid(0xc08b);
        assert_eq!(hero.descriptor.model_key, "logitech:c08b");
        assert!(hero.descriptor.name.starts_with("G502 HERO Gaming Mouse"));
        let names: Vec<&str> = hero
            .capabilities
            .buttons
            .iter()
            .map(|b| b.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "Left click",
                "Right click",
                "Middle click",
                "Back",
                "Forward",
                "G6 · DPI shift (1200)",
                "G7 · DPI down",
                "G8 · DPI up",
                "G9 · Profile cycle",
                "G10 · Scroll right",
                "G11 · Scroll left"
            ]
        );
        let zones: Vec<&str> = hero
            .capabilities
            .lighting
            .iter()
            .map(|z| z.name.as_str())
            .collect();
        assert_eq!(zones, ["Primary", "Logo"]);
    }

    #[test]
    fn k55_zones_partition_the_board_like_the_driver() {
        let k55 = by_pid(0x1ba4);
        let layout = k55.capabilities.layout.as_ref().unwrap();
        assert!(k55.capabilities.per_key.is_none());
        assert_eq!(layout.keys.len(), 104 + 6 + 4);
        let mut seen = std::collections::HashSet::new();
        for z in &k55.capabilities.lighting {
            for k in &z.keys {
                assert!(layout.contains(*k), "0x{k:x} is not on the board");
                assert!(seen.insert(*k), "0x{k:x} lit by two zones");
            }
        }
        assert_eq!(seen.len(), layout.keys.len());
        assert_eq!(
            layout.keys.iter().find(|k| k.id == 0xe7).unwrap().label,
            "Fn"
        );
    }
}
