//! Corsair demo devices and tour steps for the model-accurate art (see `super`).
//!
//! The Corsair drivers report no remappable buttons, so these simulated mice list the physical
//! buttons with the names the artwork knows, which puts every callout through its paces.

use gearwise_core::model::{
    Battery, ButtonAction, Connection, DeviceKind, DpiCaps, DpiFunction, DpiStage, DpiState,
    EffectKind, Modifiers, MouseButton, Rgb, Vendor,
};

use crate::screenshot::Step;
use crate::screenshot_demo::{Spec, btn, lit, mouse_btn, mouse_like, zone};
use crate::state::Tab;
use crate::worker::DeviceSnapshot;

const M65: &str = "corsair:1b5a";
const DARK_CORE: &str = "corsair:1b7e";
const KATAR: &str = "corsair:1bac";
const HARPOON: &str = "corsair:1b5e";
const SCIMITAR: &str = "corsair:1b8b";

pub(super) fn devices() -> Vec<DeviceSnapshot> {
    vec![
        m65_elite(),
        dark_core_se(),
        katar_pro_xt(),
        harpoon_wireless(),
        scimitar_elite(),
    ]
}

pub(super) fn steps() -> Vec<Step> {
    vec![
        Step::device("51-m65-rgb-ultra", M65, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(8)
        }),
        Step::device("52-dark-core-rgb-pro-se", DARK_CORE, Tab::Lighting, |a| {
            a.pages.light.zone = 3
        })
        .light(),
        Step::device("53-scimitar-rgb-elite", SCIMITAR, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(11)
        }),
        Step::device("54-katar-pro-xt", KATAR, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(4)
        })
        .light(),
        Step::device("55-harpoon-rgb-wireless", HARPOON, Tab::Lighting, |a| {
            a.pages.light.zone = 1
        })
        .light(),
    ]
}

/// One physical button: its name and what it does out of the box.
pub(super) type Key = (&'static str, ButtonAction);

/// A simulated mouse of any brand. `keys` are numbered from 1; `zones` are (id, name, colour);
/// `dpi` is (min, max, step).
pub(super) struct Sim<'a> {
    pub vendor: Vendor,
    pub name: &'a str,
    pub pid: u16,
    pub keys: &'a [Key],
    pub zones: &'a [(u8, &'a str, (u8, u8, u8))],
    pub dpi: (u16, u16, u16),
    pub wireless: bool,
    pub firmware: &'a str,
}

pub(super) fn sim_mouse(sim: Sim) -> DeviceSnapshot {
    let buttons = sim
        .keys
        .iter()
        .enumerate()
        .map(|(i, (n, a))| btn(i as u16 + 1, n, a.clone(), None))
        .collect();
    let effects = [
        EffectKind::Off,
        EffectKind::Static,
        EffectKind::Breathing,
        EffectKind::ColorCycle,
    ];
    let mut snap = mouse_like(Spec {
        name: sim.name,
        vendor: sim.vendor,
        kind: DeviceKind::Mouse,
        pid: sim.pid,
        buttons,
        zones: sim
            .zones
            .iter()
            .map(|(id, n, _)| zone(*id, n, &effects))
            .collect(),
        lighting: sim
            .zones
            .iter()
            .map(|(id, _, (r, g, b))| (*id, lit(Rgb::new(*r, *g, *b))))
            .collect(),
        custom: Vec::new(),
    });
    snap.descriptor.connection = if sim.wireless {
        Connection::Wireless
    } else {
        Connection::Wired
    };
    snap.descriptor.firmware = Some(sim.firmware.into());
    let (min, max, step) = sim.dpi;
    snap.capabilities.dpi = Some(DpiCaps {
        min,
        max,
        step,
        max_stages: 5,
        separate_xy: false,
    });
    snap.capabilities.battery = sim.wireless;
    snap.state.battery = sim.wireless.then_some(Battery {
        percent: Some(78),
        charging: false,
    });
    snap.state.dpi = Some(DpiState {
        stages: vec![
            DpiStage::uniform(800),
            DpiStage::uniform(1600),
            DpiStage::uniform(3200),
        ],
        active: 1,
    });
    snap.capabilities.onboard_profiles = 3;
    snap.state.onboard_profile = Some(0);
    snap
}

/// A simulated Corsair mouse: `dpi` is (max, wireless).
fn corsair_mouse(
    name: &str,
    pid: u16,
    keys: &[Key],
    zones: &[(u8, &str, (u8, u8, u8))],
    dpi: (u16, bool),
) -> DeviceSnapshot {
    sim_mouse(Sim {
        vendor: Vendor::Corsair,
        name,
        pid,
        keys,
        zones,
        dpi: (100, dpi.0, 10),
        wireless: dpi.1,
        firmware: "4.9.88",
    })
}

pub(super) fn dpi(f: DpiFunction) -> ButtonAction {
    ButtonAction::Dpi(f)
}

/// The M65 RGB Elite: eight buttons (the sniper key on the thumb side) and the zones the NXP
/// driver reports for the M65 family.
fn m65_elite() -> DeviceSnapshot {
    corsair_mouse(
        "Corsair M65 RGB Elite",
        0x1b5a,
        &[
            ("Left click", mouse_btn(MouseButton::Left)),
            ("Right click", mouse_btn(MouseButton::Right)),
            ("Middle click", mouse_btn(MouseButton::Middle)),
            ("DPI up", dpi(DpiFunction::Up)),
            ("DPI down", dpi(DpiFunction::Down)),
            ("Back", mouse_btn(MouseButton::Back)),
            ("Forward", mouse_btn(MouseButton::Forward)),
            ("Sniper", dpi(DpiFunction::Shift(400))),
        ],
        &[
            (1, "Front", (255, 40, 60)),
            (2, "Logo", (0, 212, 170)),
            (4, "Scroll wheel", (0, 184, 252)),
            (6, "Side", (255, 160, 0)),
        ],
        (18_000, false),
    )
}

/// The Dark Core RGB Pro SE: wireless, eight buttons and the four zones of the Bragi driver.
fn dark_core_se() -> DeviceSnapshot {
    corsair_mouse(
        "Corsair Dark Core RGB Pro SE",
        0x1b7e,
        &[
            ("Left click", mouse_btn(MouseButton::Left)),
            ("Right click", mouse_btn(MouseButton::Right)),
            ("Middle click", mouse_btn(MouseButton::Middle)),
            ("DPI button", dpi(DpiFunction::Cycle)),
            ("Profile cycle", ButtonAction::ProfileCycle),
            ("Sniper", dpi(DpiFunction::Shift(400))),
            ("Forward", mouse_btn(MouseButton::Forward)),
            ("Back", mouse_btn(MouseButton::Back)),
        ],
        &[
            (0, "Scroll wheel", (0, 140, 255)),
            (1, "Logo", (0, 212, 170)),
            (2, "Side accents", (150, 70, 255)),
            (3, "DPI indicators", (255, 160, 0)),
        ],
        (18_000, true),
    )
}

/// The Katar Pro XT: six buttons and a single "Scroll wheel" zone, as the Bragi driver reports.
fn katar_pro_xt() -> DeviceSnapshot {
    corsair_mouse(
        "Corsair Katar Pro XT",
        0x1bac,
        &[
            ("Left click", mouse_btn(MouseButton::Left)),
            ("Right click", mouse_btn(MouseButton::Right)),
            ("Middle click", mouse_btn(MouseButton::Middle)),
            ("DPI button", dpi(DpiFunction::Cycle)),
            ("Forward", mouse_btn(MouseButton::Forward)),
            ("Back", mouse_btn(MouseButton::Back)),
        ],
        &[(0, "Scroll wheel", (0, 212, 170))],
        (18_000, false),
    )
}

/// The Harpoon RGB Wireless: six buttons and the logo and DPI indicator zones.
fn harpoon_wireless() -> DeviceSnapshot {
    corsair_mouse(
        "Corsair Harpoon RGB Wireless",
        0x1b5e,
        &[
            ("Left click", mouse_btn(MouseButton::Left)),
            ("Right click", mouse_btn(MouseButton::Right)),
            ("Middle click", mouse_btn(MouseButton::Middle)),
            ("DPI button", dpi(DpiFunction::Cycle)),
            ("Forward", mouse_btn(MouseButton::Forward)),
            ("Back", mouse_btn(MouseButton::Back)),
        ],
        &[
            (0, "DPI indicator", (60, 220, 90)),
            (1, "Logo", (0, 184, 252)),
        ],
        (10_000, true),
    )
}

/// The Scimitar RGB Elite: 17 buttons (the twelve side keys of the sliding panel numbered as the
/// app numbers a thumb grid) and the zones the NXP driver reports.
fn scimitar_elite() -> DeviceSnapshot {
    let mut keys: Vec<Key> = vec![
        ("Left click", mouse_btn(MouseButton::Left)),
        ("Right click", mouse_btn(MouseButton::Right)),
        ("Middle click", mouse_btn(MouseButton::Middle)),
        ("DPI up", dpi(DpiFunction::Up)),
        ("DPI down", dpi(DpiFunction::Down)),
    ];
    let side = [
        "Side button 1",
        "Side button 2",
        "Side button 3",
        "Side button 4",
        "Side button 5",
        "Side button 6",
        "Side button 7",
        "Side button 8",
        "Side button 9",
        "Side button 10",
        "Side button 11",
        "Side button 12",
    ];
    let numbers = [
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2d, 0x2e,
    ];
    keys.extend(side.iter().zip(numbers).map(|(n, usage)| {
        (
            *n,
            ButtonAction::Key {
                usage,
                mods: Modifiers(0),
            },
        )
    }));
    corsair_mouse(
        "Corsair Scimitar RGB Elite",
        0x1b8b,
        &keys,
        &[
            (2, "Logo", (255, 90, 70)),
            (4, "Scroll wheel", (200, 220, 255)),
            (5, "Thumb", (0, 140, 255)),
        ],
        (18_000, false),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::illustration::{Design, MouseArt};

    /// Every simulated mouse is drawn by a model drawing and every button lands on a part.
    #[test]
    fn every_demo_button_lands_on_a_part_of_its_drawing() {
        for snap in devices() {
            let design = Design::for_name(&snap.descriptor.name);
            assert!(
                matches!(design, Design::Model(_)),
                "{} has no model drawing",
                snap.descriptor.name
            );
            let rect = egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(300.0, 300.0 / design.aspect()),
            );
            let art = MouseArt::new(rect, design);
            let buttons = &snap.capabilities.buttons;
            let placed = art.place(buttons);
            assert_eq!(placed.len(), buttons.len(), "{}", snap.descriptor.name);
            assert!(
                placed.iter().all(|p| p.role.is_some()),
                "{}: a button falls back to a hotspot",
                snap.descriptor.name
            );
        }
    }

    #[test]
    fn model_keys_are_unique_and_steps_name_devices_this_file_creates() {
        let all = devices();
        let mut keys: Vec<_> = all.iter().map(|d| d.descriptor.model_key.clone()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), all.len());
        for step in steps() {
            assert!(step.targets(&all), "{}", step.name());
        }
    }
}
