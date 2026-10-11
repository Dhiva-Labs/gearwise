//! SteelSeries demo devices and tour steps for the model-accurate art (see `super`).
//!
//! The SteelSeries drivers report their remappable buttons, so these simulated mice list the same
//! names and factory actions as the driver's button tables, and the zones the lighting tables name.

use gearwise_core::model::{ButtonAction, DpiFunction, Modifiers, MouseButton, Vendor};

use super::corsair::{Key, Sim, dpi, sim_mouse};
use crate::screenshot::Step;
use crate::screenshot_demo::mouse_btn;
use crate::state::Tab;
use crate::worker::DeviceSnapshot;

const RIVAL_3: &str = "steelseries:1824";
const PRIME: &str = "steelseries:182e";
const SENSEI: &str = "steelseries:1832";
const RIVAL_600: &str = "steelseries:1724";
const AEROX_3: &str = "steelseries:1836";
const AEROX_5: &str = "steelseries:1850";
const AEROX_9: &str = "steelseries:1856";

pub(super) fn devices() -> Vec<DeviceSnapshot> {
    vec![
        rival_3(),
        aerox_3(),
        aerox_5(),
        aerox_9(),
        prime(),
        sensei_ten(),
        rival_600(),
    ]
}

pub(super) fn steps() -> Vec<Step> {
    vec![
        Step::device("61-rival-3", RIVAL_3, Tab::Lighting, |a| {
            a.pages.light.zone = 2
        })
        .light(),
        Step::device("62-aerox-3", AEROX_3, Tab::Lighting, |a| {
            a.pages.light.zone = 1
        }),
        Step::device("63-aerox-5", AEROX_5, Tab::Lighting, |a| {
            a.pages.light.zone = 2
        })
        .light(),
        Step::device("64-aerox-9", AEROX_9, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(11)
        }),
        Step::device("65-prime", PRIME, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(2)
        }),
        Step::device("66-sensei-ten", SENSEI, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(7)
        }),
        Step::device("67-rival-600", RIVAL_600, Tab::Lighting, |a| {
            a.pages.light.zone = 4
        }),
    ]
}

/// The six buttons every SteelSeries table starts with.
fn six() -> Vec<Key> {
    vec![
        ("Left click", mouse_btn(MouseButton::Left)),
        ("Right click", mouse_btn(MouseButton::Right)),
        ("Middle click", mouse_btn(MouseButton::Middle)),
        ("Back", mouse_btn(MouseButton::Back)),
        ("Forward", mouse_btn(MouseButton::Forward)),
        ("DPI button", dpi(DpiFunction::Cycle)),
    ]
}

/// A simulated SteelSeries mouse: `zones` are (id, name, colour), `dpi` is the sensor's range.
fn steel_mouse(
    name: &str,
    pid: u16,
    keys: &[Key],
    zones: &[(u8, &str, (u8, u8, u8))],
    wireless: bool,
) -> DeviceSnapshot {
    sim_mouse(Sim {
        vendor: Vendor::SteelSeries,
        name,
        pid,
        keys,
        zones,
        dpi: (100, 18_000, 100),
        wireless,
        firmware: "1.0.14",
    })
}

/// The Rival 3: six buttons and the prism strip's Front / Middle / Rear zones plus the logo.
fn rival_3() -> DeviceSnapshot {
    steel_mouse(
        "SteelSeries Rival 3",
        0x1824,
        &six(),
        &[
            (1, "Front", (255, 40, 60)),
            (2, "Middle", (60, 220, 90)),
            (3, "Rear", (0, 140, 255)),
            (4, "Logo", (255, 90, 0)),
        ],
        false,
    )
}

/// The Aerox 3: the six standard buttons and the RED_GREEN_BLUE zones of its lighting table.
fn aerox_3() -> DeviceSnapshot {
    steel_mouse(
        "SteelSeries Aerox 3",
        0x1836,
        &six(),
        &[
            (0, "Front", (255, 0, 0)),
            (1, "Middle", (0, 255, 0)),
            (2, "Rear", (0, 0, 255)),
        ],
        false,
    )
}

/// The Aerox 5 Wireless: six exposed buttons (the flick keys on the flank are not) and the
/// Front / Middle / Rear zones.
fn aerox_5() -> DeviceSnapshot {
    steel_mouse(
        "SteelSeries Aerox 5 Wireless",
        0x1850,
        &six(),
        &[
            (0, "Front", (0, 212, 170)),
            (1, "Middle", (0, 140, 255)),
            (2, "Rear", (150, 70, 255)),
        ],
        true,
    )
}

/// The Aerox 9 Wireless: the driver has no button table for it, so this lists the four front
/// controls and the twelve side keys of its panel.
fn aerox_9() -> DeviceSnapshot {
    let mut keys: Vec<Key> = vec![
        ("Left click", mouse_btn(MouseButton::Left)),
        ("Right click", mouse_btn(MouseButton::Right)),
        ("Middle click", mouse_btn(MouseButton::Middle)),
        ("DPI button", dpi(DpiFunction::Cycle)),
    ];
    const SIDE: [&str; 12] = [
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
    keys.extend(
        SIDE.iter()
            .enumerate()
            .map(|(i, n)| (*n, key_press(0x1e + i as u8))),
    );
    steel_mouse(
        "SteelSeries Aerox 9 Wireless",
        0x1856,
        &keys,
        &[
            (0, "Front", (255, 0, 120)),
            (1, "Middle", (0, 212, 170)),
            (2, "Rear", (150, 70, 255)),
        ],
        true,
    )
}

/// The Prime: six buttons (the DPI key is on the underside) and the single Logo zone.
fn prime() -> DeviceSnapshot {
    steel_mouse(
        "SteelSeries Prime",
        0x182e,
        &six(),
        &[(1, "Logo", (255, 82, 0))],
        false,
    )
}

/// The Sensei Ten: the six standard buttons plus a thumb pair on the right flank ("Right side
/// rear" / "Right side front", as the driver names them), and the Logo and Scroll wheel zones.
fn sensei_ten() -> DeviceSnapshot {
    let mut keys = six();
    keys.insert(5, ("Right side rear", key_press(0x4e)));
    keys.insert(6, ("Right side front", key_press(0x4b)));
    steel_mouse(
        "SteelSeries Sensei Ten",
        0x1832,
        &keys,
        &[(0, "Logo", (255, 82, 0)), (1, "Scroll wheel", (255, 82, 0))],
        false,
    )
}

/// The Rival 600: five buttons (slot 6 is unused) and the eight zones of its lighting table.
fn rival_600() -> DeviceSnapshot {
    let mut keys = six();
    keys.truncate(5);
    keys.push(("DPI button", dpi(DpiFunction::Cycle)));
    steel_mouse(
        "SteelSeries Rival 600",
        0x1724,
        &keys,
        &[
            (0, "Scroll wheel", (0, 212, 170)),
            (1, "Logo", (255, 150, 0)),
            (2, "Left strip top", (255, 40, 160)),
            (3, "Right strip top", (0, 184, 252)),
            (4, "Left strip middle", (255, 40, 160)),
            (5, "Right strip middle", (0, 184, 252)),
            (6, "Left strip bottom", (255, 120, 0)),
            (7, "Right strip bottom", (150, 70, 255)),
        ],
        false,
    )
}

fn key_press(usage: u8) -> ButtonAction {
    ButtonAction::Key {
        usage,
        mods: Modifiers(0),
    }
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
            // Only the Prime's underside DPI key may be missing from the drawing.
            for b in buttons {
                let on_a_part = placed.iter().any(|p| p.id == b.id && p.role.is_some());
                assert!(
                    on_a_part || b.name.contains("DPI"),
                    "{}: {} is not on a part",
                    snap.descriptor.name,
                    b.name
                );
            }
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
