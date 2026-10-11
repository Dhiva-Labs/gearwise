//! Razer demo devices and tour steps for the model-accurate art (see `super`).

use gearwise_core::model::{
    Battery, ButtonAction, ButtonCaps, Connection, DeviceKind, DpiCaps, DpiFunction, DpiStage,
    DpiState, EffectKind, LightingZoneCaps, Modifiers, MouseButton, Rgb, Vendor, ZoneState, media,
};

use crate::screenshot::Step;
use crate::screenshot_demo::{Spec, btn, lit, mouse_btn, mouse_like, zone};
use crate::state::Tab;
use crate::worker::DeviceSnapshot;

const DEATHADDER_V3_PRO: &str = "razer:00b7";
const VIPER_V3_PRO: &str = "razer:00c1";
const VIPER_ULTIMATE: &str = "razer:007b";
const BASILISK_V3_PRO: &str = "razer:00ab";
const NAGA_V2_PRO: &str = "razer:00a8";
const COBRA_PRO: &str = "razer:00b0";

pub(super) fn devices() -> Vec<DeviceSnapshot> {
    vec![
        deathadder_v3_pro(),
        viper_v3_pro(),
        viper_ultimate(),
        basilisk_v3_pro(),
        naga_v2_pro(),
        cobra_pro(),
    ]
}

pub(super) fn steps() -> Vec<Step> {
    vec![
        Step::device(
            "40-deathadder-v3-pro",
            DEATHADDER_V3_PRO,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(3),
        ),
        Step::device("41-viper-v3-pro", VIPER_V3_PRO, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(4)
        }),
        Step::device("42-viper-ultimate", VIPER_ULTIMATE, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(6)
        }),
        Step::device(
            "43-basilisk-v3-pro",
            BASILISK_V3_PRO,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(7),
        ),
        Step::device("44-naga-v2-pro", NAGA_V2_PRO, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(12)
        }),
        Step::device("45-cobra-pro", COBRA_PRO, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(3)
        }),
        Step::device(
            "46-basilisk-v3-pro-lighting",
            BASILISK_V3_PRO,
            Tab::Lighting,
            |a| a.pages.light.zone = 2,
        ),
        Step::device("47-cobra-pro-lighting", COBRA_PRO, Tab::Lighting, |a| {
            a.pages.light.zone = 2
        }),
    ]
}

/// What differs between the Razer mice; the rest (DPI stages, polling rates, firmware) is
/// common ground.
struct Mouse<'a> {
    name: &'a str,
    pid: u16,
    wireless: bool,
    dpi_max: u16,
    buttons: Vec<ButtonCaps>,
    zones: Vec<LightingZoneCaps>,
    lighting: Vec<(u8, ZoneState)>,
    custom: Vec<(u16, ButtonAction)>,
}

fn razer_mouse(m: Mouse) -> DeviceSnapshot {
    let mut snap = mouse_like(Spec {
        name: m.name,
        vendor: Vendor::Razer,
        kind: DeviceKind::Mouse,
        pid: m.pid,
        buttons: m.buttons,
        zones: m.zones,
        lighting: m.lighting,
        custom: m.custom,
    });
    snap.descriptor.connection = if m.wireless {
        Connection::Wireless
    } else {
        Connection::Wired
    };
    snap.descriptor.firmware = Some("1.02.00".into());
    snap.capabilities.dpi = Some(DpiCaps {
        min: 100,
        max: m.dpi_max,
        step: 50,
        max_stages: 5,
        separate_xy: false,
    });
    snap.capabilities.report_rates = vec![125, 500, 1000];
    snap.capabilities.onboard_profiles = 1;
    snap.capabilities.battery = m.wireless;
    snap.state.dpi = Some(DpiState {
        stages: [800, 1600, 3200].map(DpiStage::uniform).to_vec(),
        active: 1,
    });
    snap.state.report_rate = Some(1000);
    snap.state.onboard_profile = Some(0);
    snap.state.battery = m.wireless.then_some(Battery {
        percent: Some(82),
        charging: false,
    });
    snap
}

/// The three primary controls every one of these mice has.
fn primary() -> Vec<ButtonCaps> {
    vec![
        btn(0, "Left click", mouse_btn(MouseButton::Left), None),
        btn(1, "Right click", mouse_btn(MouseButton::Right), None),
        btn(2, "Middle click", mouse_btn(MouseButton::Middle), None),
    ]
}

fn ctrl(usage: u8) -> ButtonAction {
    ButtonAction::Key {
        usage,
        mods: Modifiers(Modifiers::LCTRL),
    }
}

/// The DeathAdder V3 Pro (wireless): five controls and no lighting. Its DPI button is on the
/// underside, so it is not part of the top-down drawing.
fn deathadder_v3_pro() -> DeviceSnapshot {
    let mut buttons = primary();
    buttons.push(btn(3, "Back", mouse_btn(MouseButton::Back), None));
    buttons.push(btn(4, "Forward", mouse_btn(MouseButton::Forward), None));
    razer_mouse(Mouse {
        name: "DeathAdder V3 Pro",
        pid: 0x00b7,
        wireless: true,
        dpi_max: 30_000,
        buttons,
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![(3, ctrl(0x1d)), (4, ButtonAction::Media(media::PLAY_PAUSE))],
    })
}

/// The Viper V3 Pro (wireless): the same five controls on the left flank only, no lighting.
fn viper_v3_pro() -> DeviceSnapshot {
    let mut buttons = primary();
    buttons.push(btn(3, "Back", mouse_btn(MouseButton::Back), None));
    buttons.push(btn(4, "Forward", mouse_btn(MouseButton::Forward), None));
    razer_mouse(Mouse {
        name: "Viper V3 Pro",
        pid: 0x00c1,
        wireless: true,
        dpi_max: 35_000,
        buttons,
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![(4, ctrl(0x1c))],
    })
}

/// The Viper Ultimate (wireless): thumb buttons on both flanks, named the way the driver names
/// the right-hand pair, and a Chroma logo.
fn viper_ultimate() -> DeviceSnapshot {
    let mut buttons = primary();
    buttons.extend([
        btn(3, "Back", mouse_btn(MouseButton::Back), None),
        btn(4, "Forward", mouse_btn(MouseButton::Forward), None),
        btn(5, "Forward (right)", mouse_btn(MouseButton::Forward), None),
        btn(6, "Back (right)", mouse_btn(MouseButton::Back), None),
    ]);
    razer_mouse(Mouse {
        name: "Viper Ultimate",
        pid: 0x007b,
        wireless: true,
        dpi_max: 20_000,
        buttons,
        zones: vec![zone(0, "Logo", &EffectKind::ALL[..5])],
        lighting: vec![(0, lit(Rgb::new(68, 214, 44)))],
        custom: vec![(6, ButtonAction::Media(media::NEXT))],
    })
}

/// The Basilisk V3 Pro (wireless): HyperScroll tilt wheel and its mode key, a DPI key, the DPI
/// clutch and two thumb buttons; Chroma on the logo, the wheel and the underglow strip.
fn basilisk_v3_pro() -> DeviceSnapshot {
    let mut buttons = primary();
    buttons.extend([
        btn(3, "Scroll left", mouse_btn(MouseButton::WheelLeft), None),
        btn(4, "Scroll right", mouse_btn(MouseButton::WheelRight), None),
        btn(5, "Wheel mode", ButtonAction::Default, None),
        btn(6, "DPI button", ButtonAction::Dpi(DpiFunction::Cycle), None),
        btn(
            7,
            "DPI clutch",
            ButtonAction::Dpi(DpiFunction::Shift(800)),
            None,
        ),
        btn(8, "Back", mouse_btn(MouseButton::Back), None),
        btn(9, "Forward", mouse_btn(MouseButton::Forward), None),
    ]);
    let fx = [
        EffectKind::Off,
        EffectKind::Static,
        EffectKind::ColorCycle,
        EffectKind::Wave,
    ];
    razer_mouse(Mouse {
        name: "Basilisk V3 Pro",
        pid: 0x00ab,
        wireless: true,
        dpi_max: 30_000,
        buttons,
        zones: vec![
            zone(0, "Logo", &fx),
            zone(1, "Scroll wheel", &fx),
            zone(2, "Underglow", &fx),
        ],
        lighting: vec![
            (0, lit(Rgb::new(68, 214, 44))),
            (1, lit(Rgb::new(68, 214, 44))),
            (2, lit(Rgb::new(0, 184, 252))),
        ],
        custom: vec![
            (7, ButtonAction::Dpi(DpiFunction::Shift(400))),
            (9, ctrl(0x1c)),
        ],
    })
}

/// The Naga V2 Pro (wireless) with the 12-button thumb plate: the driver-style names for the
/// wheel tilt and DPI keys, then "Side button 1" to "Side button 12". Chroma on the logo, the
/// wheel and the thumb plate.
fn naga_v2_pro() -> DeviceSnapshot {
    let mut buttons = primary();
    buttons.extend([
        btn(3, "Scroll left", mouse_btn(MouseButton::WheelLeft), None),
        btn(4, "Scroll right", mouse_btn(MouseButton::WheelRight), None),
        btn(5, "DPI up", ButtonAction::Dpi(DpiFunction::Up), None),
        btn(6, "DPI down", ButtonAction::Dpi(DpiFunction::Down), None),
    ]);
    buttons.extend((1..=12u16).map(|n| {
        btn(
            6 + n,
            &format!("Side button {n}"),
            ButtonAction::Key {
                usage: 0x1d + n as u8,
                mods: Modifiers(0),
            },
            None,
        )
    }));
    let fx = &EffectKind::ALL[..5];
    razer_mouse(Mouse {
        name: "Naga V2 Pro",
        pid: 0x00a8,
        wireless: true,
        dpi_max: 30_000,
        buttons,
        zones: vec![
            zone(0, "Logo", fx),
            zone(1, "Scroll wheel", fx),
            zone(2, "Thumb grid", fx),
        ],
        lighting: vec![
            (0, lit(Rgb::new(68, 214, 44))),
            (1, lit(Rgb::new(68, 214, 44))),
            (2, lit(Rgb::new(68, 214, 44))),
        ],
        custom: vec![(9, ButtonAction::Disabled), (12, ctrl(0x16))],
    })
}

/// The Cobra Pro (wireless): the pill-shaped DPI key behind the wheel, two thumb buttons; Chroma
/// on the logo, the wheel and the underglow strip.
fn cobra_pro() -> DeviceSnapshot {
    let mut buttons = primary();
    buttons.extend([
        btn(3, "DPI button", ButtonAction::Dpi(DpiFunction::Cycle), None),
        btn(4, "Back", mouse_btn(MouseButton::Back), None),
        btn(5, "Forward", mouse_btn(MouseButton::Forward), None),
    ]);
    let fx = [
        EffectKind::Off,
        EffectKind::Static,
        EffectKind::ColorCycle,
        EffectKind::Wave,
    ];
    razer_mouse(Mouse {
        name: "Cobra Pro",
        pid: 0x00b0,
        wireless: true,
        dpi_max: 30_000,
        buttons,
        zones: vec![
            zone(0, "Logo", &fx),
            zone(1, "Scroll wheel", &fx),
            zone(2, "Underglow", &fx),
        ],
        lighting: vec![
            (0, lit(Rgb::new(68, 214, 44))),
            (1, lit(Rgb::new(68, 214, 44))),
            (2, lit(Rgb::new(255, 77, 94))),
        ],
        custom: vec![(5, ButtonAction::Media(media::PLAY_PAUSE))],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::illustration::{Design, MouseArt};

    /// Every demo mouse here, placed on its drawing: each control lands on a part, and each
    /// device has a model key of its own.
    #[test]
    fn every_demo_button_lands_on_a_part() {
        let mut keys = std::collections::HashSet::new();
        for dev in devices() {
            let name = &dev.descriptor.name;
            assert!(
                keys.insert(dev.descriptor.model_key.clone()),
                "{name}: model key is not unique"
            );
            let design = Design::for_name(name);
            assert!(
                matches!(design, Design::Model(_)),
                "{name} has no model art"
            );
            let rect = egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 320.0 / design.aspect()),
            );
            let placed = MouseArt::new(rect, design).place(&dev.capabilities.buttons);
            assert_eq!(placed.len(), dev.capabilities.buttons.len(), "{name}");
            assert!(placed.iter().all(|p| p.role.is_some()), "{name}");
        }
    }

    #[test]
    fn steps_only_name_devices_this_file_creates() {
        for step in steps() {
            assert!(step.targets(&devices()), "{}", step.name());
        }
    }
}
