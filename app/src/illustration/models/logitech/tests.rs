//! Geometry checks for the Logitech drawings: that each control sits where the real one does.

use egui::{Rect, pos2, vec2};
use gearwise_core::model::{ButtonAction, ButtonCaps, DpiFunction, MouseButton};

use crate::illustration::mouse::{Design, MouseArt};
use crate::illustration::roles::Role;

fn caps(id: u16, name: &str, default: ButtonAction) -> ButtonCaps {
    ButtonCaps {
        id,
        name: name.into(),
        remappable: true,
        default_action: default,
        hotspot: None,
    }
}

fn art(name: &str, width: f32) -> MouseArt {
    let design = Design::for_name(name);
    MouseArt::new(
        Rect::from_min_size(pos2(0.0, 0.0), vec2(width, width / design.aspect())),
        design,
    )
}

/// The real G502 layout: G7/G8 left of the left button, the wheel tilts either side of
/// the wheel, the profile key behind it and the thumb buttons on the left edge.
#[test]
fn g502_parts_sit_where_the_real_buttons_are() {
    let art = art("G502 HERO Gaming Mouse", 290.0);
    let at = |r: Role| art.part(r).unwrap().anchor;
    let cx = art.rect.center().x;
    for r in [Role::DpiUp, Role::DpiDown, Role::Left] {
        assert!(at(r).x < cx, "{r:?} is left of the wheel");
    }
    assert!(
        at(Role::DpiUp).y < at(Role::DpiDown).y,
        "G8 is in front of G7"
    );
    assert!(
        at(Role::DpiUp).x < at(Role::Left).x,
        "G7/G8 are left of the left button"
    );
    assert!(at(Role::TiltLeft).x < art.wheel.center().x);
    assert!(at(Role::TiltRight).x > art.wheel.center().x);
    assert!(
        at(Role::Profile).y > art.wheel.max.y,
        "G9 is behind the wheel"
    );
    assert!(at(Role::DpiCycle).y < at(Role::Profile).y);
    assert!(
        at(Role::Forward).y < at(Role::Back).y,
        "G5 sits in front of G4"
    );
    assert!(at(Role::Sniper).y < at(Role::Forward).y);
    assert!(
        art.logo.0.y > at(Role::Profile).y,
        "the logo is on the palm"
    );
}

/// The MX Master 3S as the Logitech driver lists it: only the controls that can be diverted.
fn mx_master_3s() -> Vec<ButtonCaps> {
    let mouse = ButtonAction::Mouse;
    vec![
        caps(0x52, "Middle click", mouse(MouseButton::Middle)),
        caps(0x53, "Back", mouse(MouseButton::Back)),
        caps(0x56, "Forward", mouse(MouseButton::Forward)),
        caps(0xc3, "Gesture button", ButtonAction::Default),
        caps(0xc4, "Smart shift", ButtonAction::Default),
        caps(0xd7, "Thumb wheel", ButtonAction::Default),
    ]
}

#[test]
fn every_mx_master_control_lands_on_its_own_part() {
    let art = art("MX Master 3S", 300.0);
    let placed = art.place(&mx_master_3s());
    assert_eq!(placed.len(), 6);
    assert!(
        placed.iter().all(|p| p.role.is_some()),
        "no control falls back to a hotspot"
    );
    let at = |r: Role| art.part(r).unwrap().anchor;
    let cx = art.rect.center().x;

    // The wheel sits between the primary panels, the mode-shift button behind it.
    assert!(at(Role::Left).x < at(Role::Middle).x && at(Role::Middle).x < at(Role::Right).x);
    assert!((at(Role::Middle).x - cx).abs() < art.rect.width() * 0.2);
    assert!(
        at(Role::WheelMode).y > art.wheel.max.y,
        "the mode button is behind the wheel"
    );
    assert!((at(Role::WheelMode).x - at(Role::Middle).x).abs() < 4.0);

    // Everything for the thumb is on the left, wheel above the Forward / Back pair.
    for r in [Role::ThumbWheel, Role::Forward, Role::Back, Role::Gesture] {
        assert!(
            at(r).x < cx - art.rect.width() * 0.1,
            "{r:?} is on the thumb side"
        );
    }
    assert!(at(Role::ThumbWheel).y < at(Role::Forward).y);
    assert!(
        at(Role::Forward).y < at(Role::Back).y,
        "Forward is in front of Back"
    );
    assert!(
        at(Role::Gesture).y > at(Role::Forward).y,
        "the gesture dash is on the rest, lower down"
    );
    assert!(at(Role::Gesture).x < at(Role::Back).x);

    // The thumb pieces ask for the left margin; the rest follow their role.
    let side = |r: Role| art.part(r).unwrap().side;
    assert_eq!(
        side(Role::ThumbWheel),
        Some(crate::illustration::roles::Side::Left)
    );
    assert_eq!(side(Role::Middle), None);
}

#[test]
fn mx_master_art_has_no_cable_and_the_right_proportions() {
    let m = Design::for_name("MX Master 3S");
    assert_eq!(m.overhang(), 0.0);
    assert!((m.aspect() - 0.674).abs() < 0.002);
}

/// An MX Master has no tilt wheel, DPI keys or profile key, so those roles have no part and
/// the drawing does not invent one.
#[test]
fn mx_master_has_only_the_parts_the_mouse_has() {
    let art = art("MX Master 3S", 300.0);
    for r in [
        Role::TiltLeft,
        Role::DpiUp,
        Role::Profile,
        Role::Sniper,
        Role::Grid(0),
    ] {
        assert!(art.part(r).is_none(), "{r:?}");
    }
    let dpi = caps(1, "DPI shift", ButtonAction::Dpi(DpiFunction::Shift(400)));
    assert!(art.place(&[dpi]).is_empty());
}

fn model_id(name: &str) -> Option<&'static str> {
    crate::illustration::models::lookup(name).map(|m| m.id())
}

fn at(art: &MouseArt, r: Role) -> egui::Pos2 {
    art.part(r)
        .unwrap_or_else(|| panic!("no {r:?} part"))
        .anchor
}

/// Every anchor sits on a part that the drawing really has, inside the art box.
fn assert_parts_inside(art: &MouseArt, roles: &[Role]) {
    for r in roles {
        let a = at(art, *r);
        assert!(
            art.rect.expand(2.0).contains(a),
            "{r:?} anchor is off the art"
        );
    }
}

#[test]
fn the_g502_x_shell_is_drawn_apart_from_the_hero() {
    // The newer shell wins on its names, the HERO family keeps the old one.
    for name in [
        "G502 X Plus (demo)",
        "G502 X PLUS",
        "G502 X LIGHTSPEED",
        "G502 X Gaming Mouse",
    ] {
        assert_eq!(model_id(name), Some("g502-x"), "{name}");
    }
    for name in [
        "G502 HERO Gaming Mouse",
        "G502 LIGHTSPEED",
        "G502 Proteus Spectrum",
    ] {
        assert_eq!(model_id(name), Some("g502"), "{name}");
    }
}

#[test]
fn g502_x_parts_sit_where_the_real_buttons_are() {
    let art = art("G502 X PLUS LIGHTSPEED", 300.0);
    let cx = art.rect.center().x;
    // G8 above G7, both on the left flank beyond the left button.
    assert!(at(&art, Role::DpiUp).y < at(&art, Role::DpiDown).y);
    for r in [
        Role::DpiUp,
        Role::DpiDown,
        Role::Sniper,
        Role::Forward,
        Role::Back,
    ] {
        assert!(
            at(&art, r).x < at(&art, Role::Left).x,
            "{r:?} is left of the left button"
        );
    }
    assert!(at(&art, Role::Sniper).y > at(&art, Role::DpiUp).y);
    assert!(
        at(&art, Role::Forward).y < at(&art, Role::Back).y,
        "G5 is in front of G4"
    );
    assert!(at(&art, Role::Left).x < cx && cx < at(&art, Role::Right).x);
    // Wheel, tilt marks either side, the cycle key and G9 behind it.
    assert!(at(&art, Role::TiltLeft).x < art.wheel.min.x);
    assert!(at(&art, Role::TiltRight).x > art.wheel.max.x);
    assert!(at(&art, Role::DpiCycle).y > art.wheel.max.y);
    assert!(at(&art, Role::DpiCycle).y < at(&art, Role::Profile).y);
    // The drawing has no right-flank pair and no thumb wheel.
    for r in [
        Role::RightBack,
        Role::RightForward,
        Role::ThumbWheel,
        Role::Gesture,
    ] {
        assert!(art.part(r).is_none(), "{r:?}");
    }
    assert_parts_inside(&art, &[Role::Left, Role::Right, Role::Middle, Role::Sniper]);
}

#[test]
fn the_superlight_is_ambidextrous_with_buttons_on_both_flanks() {
    for name in [
        "G Pro X Superlight 2",
        "PRO X 2 SUPERLIGHT",
        "G Pro X Superlight",
        "G Pro Wireless Gaming Mouse",
        "pro x superlight",
    ] {
        assert_eq!(model_id(name), Some("superlight-2"), "{name}");
    }
    let art = art("G Pro X Superlight 2", 300.0);
    let cx = art.rect.center().x;
    for r in [Role::Forward, Role::Back] {
        assert!(
            at(&art, r).x < cx - art.rect.width() * 0.3,
            "{r:?} is on the left flank"
        );
    }
    for r in [Role::RightForward, Role::RightBack] {
        assert!(
            at(&art, r).x > cx + art.rect.width() * 0.3,
            "{r:?} is on the right flank"
        );
    }
    // The pairs mirror each other, front before rear.
    assert!(at(&art, Role::Forward).y < at(&art, Role::Back).y);
    assert!((at(&art, Role::Forward).y - at(&art, Role::RightForward).y).abs() < 1.0);
    assert!(at(&art, Role::Left).x < at(&art, Role::Middle).x);
    assert!(at(&art, Role::Middle).x < at(&art, Role::Right).x);
    // The shell is a long egg: the logo is on the palm, well behind the buttons.
    assert!(art.logo.0.y > art.rect.center().y);
    // It has no cable, no tilt marks and no DPI key on top.
    assert_eq!(Design::for_name("G Pro X Superlight 2").overhang(), 0.0);
    for r in [Role::TiltLeft, Role::DpiCycle, Role::Sniper, Role::Profile] {
        assert!(art.part(r).is_none(), "{r:?}");
    }
}

#[test]
fn g305_and_g703_have_the_dpi_key_in_the_slot_behind_the_wheel() {
    for (name, id) in [
        ("G305 LIGHTSPEED", "g305"),
        ("G304 LIGHTSPEED Wireless Gaming Mouse", "g305"),
        ("G703 LIGHTSPEED", "g703"),
        ("G403 HERO Gaming Mouse", "g703"),
        ("G403 Prodigy Gaming Mouse", "g703"),
    ] {
        assert_eq!(model_id(name), Some(id), "{name}");
        let art = art(name, 300.0);
        let cx = art.rect.center().x;
        assert!((at(&art, Role::Middle).x - cx).abs() < art.rect.width() * 0.1);
        assert!(
            at(&art, Role::DpiCycle).y > art.wheel.max.y,
            "{name}: the key is behind the wheel"
        );
        assert!(
            (at(&art, Role::DpiCycle).x - at(&art, Role::Middle).x).abs() < art.rect.width() * 0.08
        );
        assert!(at(&art, Role::Left).x < at(&art, Role::Middle).x);
        assert!(at(&art, Role::Middle).x < at(&art, Role::Right).x);
        // Two thumb buttons on the left, front before rear; nothing on the right.
        assert!(at(&art, Role::Forward).x < cx && at(&art, Role::Back).x < cx);
        assert!(at(&art, Role::Forward).y < at(&art, Role::Back).y);
        assert!(art.part(Role::RightBack).is_none() && art.part(Role::RightForward).is_none());
        assert!(art.logo.0.y > at(&art, Role::DpiCycle).y);
    }
}

#[test]
fn g903_has_thumb_buttons_on_both_sides_and_the_dpi_keys_on_the_spine() {
    assert_eq!(model_id("G903 LIGHTSPEED"), Some("g903"));
    assert_eq!(model_id("G903 HERO Wireless Gaming Mouse"), Some("g903"));
    let art = art("G903 LIGHTSPEED", 300.0);
    let cx = art.rect.center().x;
    for r in [Role::Forward, Role::Back] {
        assert!(at(&art, r).x < cx - art.rect.width() * 0.3, "{r:?}");
    }
    for r in [Role::RightForward, Role::RightBack] {
        assert!(at(&art, r).x > cx + art.rect.width() * 0.3, "{r:?}");
    }
    // The spine: wheel, then the cycle key, then the two DPI keys side by side (down, up).
    assert!(at(&art, Role::DpiCycle).y > art.wheel.max.y);
    assert!(at(&art, Role::DpiDown).y > at(&art, Role::DpiCycle).y);
    assert!((at(&art, Role::DpiDown).y - at(&art, Role::DpiUp).y).abs() < 1.0);
    assert!(at(&art, Role::DpiDown).x < at(&art, Role::DpiUp).x);
    for r in [Role::DpiCycle, Role::DpiDown, Role::DpiUp, Role::Middle] {
        assert!(
            (at(&art, r).x - cx).abs() < art.rect.width() * 0.12,
            "{r:?} is on the spine"
        );
    }
    assert!(at(&art, Role::TiltLeft).x < art.wheel.min.x);
    assert!(at(&art, Role::TiltRight).x > art.wheel.max.x);
    assert!(at(&art, Role::Left).x < at(&art, Role::DpiDown).x);
    assert!(at(&art, Role::Right).x > at(&art, Role::DpiUp).x);
    assert!(art.logo.0.y > at(&art, Role::DpiDown).y);
}

#[test]
fn g600_has_a_twelve_cell_thumb_grid_and_a_ring_finger_button() {
    assert_eq!(model_id("G600 MMO Gaming Mouse"), Some("g600"));
    let art = art("G600 MMO", 300.0);
    let cx = art.rect.center().x;
    let cells: Vec<egui::Pos2> = (0..12).map(|i| at(&art, Role::Grid(i))).collect();
    // On the left flank, four columns front to back (G9..G11 share the front column).
    assert!(
        cells
            .iter()
            .all(|c| c.x < art.rect.min.x + art.rect.width() * 0.2)
    );
    assert!(cells[0].y < cells[3].y && cells[3].y < cells[6].y && cells[6].y < cells[9].y);
    assert!((cells[0].y - cells[2].y).abs() < 1.0, "one column");
    assert!(
        cells[0].x < cells[1].x && cells[1].x < cells[2].x,
        "bottom row is the outer one"
    );
    assert!(art.part(Role::Grid(12)).is_none());
    // G-shift on the right flank, G8 above G7 behind the wheel, tilt either side of it.
    assert!(at(&art, Role::Sniper).x > cx + art.rect.width() * 0.25);
    assert!(at(&art, Role::DpiUp).y < at(&art, Role::DpiDown).y);
    assert!(at(&art, Role::DpiUp).y > art.wheel.max.y);
    assert!(at(&art, Role::TiltLeft).x < art.wheel.min.x);
    assert!(at(&art, Role::TiltRight).x > art.wheel.max.x);
    // It is wired: the cable leaves the nose.
    assert!(Design::for_name("G600 MMO").overhang() > 0.05);
    assert!(art.part(Role::Back).is_none() && art.part(Role::Forward).is_none());
}

#[test]
fn mx_anywhere_has_a_mode_button_and_two_thumb_buttons() {
    for name in [
        "MX Anywhere 3S",
        "MX Anywhere 3S (demo)",
        "MX Anywhere 3",
        "MX Anywhere 3 for Mac",
    ] {
        assert_eq!(model_id(name), Some("mx-anywhere"), "{name}");
    }
    // The Master keeps its own drawing.
    assert_eq!(model_id("MX Master 3S"), Some("mx-master"));
    let art = art("MX Anywhere 3S", 300.0);
    let cx = art.rect.center().x;
    assert!((at(&art, Role::WheelMode).x - at(&art, Role::Middle).x).abs() < 3.0);
    assert!(
        at(&art, Role::WheelMode).y > art.wheel.max.y,
        "the mode button is behind the wheel"
    );
    assert!(at(&art, Role::Forward).x < cx - art.rect.width() * 0.3);
    assert!(at(&art, Role::Forward).y < at(&art, Role::Back).y);
    for r in [
        Role::ThumbWheel,
        Role::Gesture,
        Role::DpiCycle,
        Role::TiltLeft,
        Role::Sniper,
    ] {
        assert!(art.part(r).is_none(), "{r:?}");
    }
}

/// Every part of every Logitech drawing: a sane polygon with its anchor on it, at any size.
#[test]
fn every_logitech_drawing_places_its_controls_on_the_shell() {
    for m in super::models() {
        let design = Design::Model(*m);
        for width in [220.0, 340.0, 520.0] {
            let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(width, width / design.aspect()));
            let art = MouseArt::new(rect, design);
            assert!(art.parts.len() >= 5, "{} has too few parts", m.id());
            for part in &art.parts {
                let bounds = Rect::from_points(&part.poly).expand(4.0);
                assert!(
                    bounds.contains(part.anchor) && rect.expand(3.0).contains(part.anchor),
                    "{} {:?}: anchor off the part",
                    m.id(),
                    part.role
                );
            }
            assert!(
                rect.expand(3.0).contains_rect(art.wheel),
                "{} wheel",
                m.id()
            );
            assert!(rect.expand(3.0).contains(art.logo.0), "{} logo", m.id());
        }
    }
}
