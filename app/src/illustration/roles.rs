//! What a mouse button is, anatomically, and which part of a device a lighting zone lights.

use egui::Color32;
use gearwise_core::model::{ButtonAction, ButtonCaps, DpiFunction, MouseButton};

/// Which part of the device a lighting zone lights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneRole {
    Logo,
    Wheel,
    Body,
}

pub fn zone_role(name: &str) -> ZoneRole {
    let n = name.to_ascii_lowercase();
    if n.contains("logo") {
        ZoneRole::Logo
    } else if n.contains("scroll") || n.contains("wheel") {
        ZoneRole::Wheel
    } else {
        ZoneRole::Body
    }
}

/// Colours a lighting preview paints on the mouse, per role.
#[derive(Clone, Copy, Default)]
pub struct MouseLook {
    pub logo: Option<Color32>,
    pub wheel: Option<Color32>,
    pub body: Option<Color32>,
    pub highlight: Option<ZoneRole>,
}

/// What a button is, anatomically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Left,
    Right,
    Middle,
    TiltLeft,
    TiltRight,
    Back,
    Forward,
    Sniper,
    DpiUp,
    DpiDown,
    DpiCycle,
    Profile,
    Grid(u8),
}

impl Role {
    /// Derive the role from the button's factory action, then from its name.
    pub fn of(b: &ButtonCaps) -> Option<Role> {
        let by_action = match &b.default_action {
            ButtonAction::Mouse(MouseButton::Left) => Some(Role::Left),
            ButtonAction::Mouse(MouseButton::Right) => Some(Role::Right),
            ButtonAction::Mouse(MouseButton::Middle) => Some(Role::Middle),
            ButtonAction::Mouse(MouseButton::Back) => Some(Role::Back),
            ButtonAction::Mouse(MouseButton::Forward) => Some(Role::Forward),
            ButtonAction::Mouse(MouseButton::WheelLeft) => Some(Role::TiltLeft),
            ButtonAction::Mouse(MouseButton::WheelRight) => Some(Role::TiltRight),
            ButtonAction::Dpi(DpiFunction::Up) => Some(Role::DpiUp),
            ButtonAction::Dpi(DpiFunction::Down) => Some(Role::DpiDown),
            ButtonAction::Dpi(DpiFunction::Cycle) => Some(Role::DpiCycle),
            ButtonAction::Dpi(DpiFunction::Shift(_)) => Some(Role::Sniper),
            ButtonAction::ProfileCycle => Some(Role::Profile),
            _ => None,
        };
        by_action.or_else(|| {
            let n = b.name.to_ascii_lowercase();
            let has = |s: &str| n.contains(s);
            Some(
                if has("scroll left") || has("tilt left") || has("wheel left") {
                    Role::TiltLeft
                } else if has("scroll right") || has("tilt right") || has("wheel right") {
                    Role::TiltRight
                } else if has("sniper") || has("dpi shift") {
                    Role::Sniper
                } else if has("dpi up") {
                    Role::DpiUp
                } else if has("dpi down") {
                    Role::DpiDown
                } else if has("dpi") {
                    Role::DpiCycle
                } else if has("profile") {
                    Role::Profile
                } else if has("middle") {
                    Role::Middle
                } else if has("left click") {
                    Role::Left
                } else if has("right click") {
                    Role::Right
                } else if n.ends_with("back") {
                    Role::Back
                } else if n.ends_with("forward") {
                    Role::Forward
                } else {
                    return None;
                },
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(id: u16, name: &str, default: ButtonAction) -> ButtonCaps {
        ButtonCaps {
            id,
            name: name.into(),
            remappable: true,
            default_action: default,
            hotspot: None,
        }
    }

    #[test]
    fn roles_come_from_the_factory_action_then_the_name() {
        assert_eq!(
            Role::of(&caps(
                1,
                "Left click",
                ButtonAction::Mouse(MouseButton::Left)
            )),
            Some(Role::Left)
        );
        assert_eq!(
            Role::of(&caps(
                2,
                "G6 · DPI shift (1200)",
                ButtonAction::Dpi(DpiFunction::Shift(1200))
            )),
            Some(Role::Sniper)
        );
        assert_eq!(
            Role::of(&caps(3, "G9 · Profile cycle", ButtonAction::ProfileCycle)),
            Some(Role::Profile)
        );
        assert_eq!(
            Role::of(&caps(4, "G11 · Scroll left", ButtonAction::Default)),
            Some(Role::TiltLeft)
        );
        assert_eq!(
            Role::of(&caps(5, "Back", ButtonAction::Default)),
            Some(Role::Back)
        );
        assert_eq!(Role::of(&caps(6, "Button 7", ButtonAction::Default)), None);
    }

    #[test]
    fn zone_names_map_to_roles() {
        assert_eq!(zone_role("Logo"), ZoneRole::Logo);
        assert_eq!(zone_role("Scroll wheel"), ZoneRole::Wheel);
        assert_eq!(zone_role("Primary"), ZoneRole::Body);
        assert_eq!(zone_role("Underglow"), ZoneRole::Body);
    }
}
