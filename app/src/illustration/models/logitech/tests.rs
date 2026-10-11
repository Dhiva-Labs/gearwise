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
