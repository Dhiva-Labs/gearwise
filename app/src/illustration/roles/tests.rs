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

fn role(name: &str) -> Option<Role> {
    Role::of(&caps(0, name, ButtonAction::Default))
}

fn role_with(name: &str, default: ButtonAction) -> Option<Role> {
    Role::of(&caps(0, name, default))
}

#[test]
fn roles_come_from_the_factory_action_then_the_name() {
    let mouse = ButtonAction::Mouse;
    assert_eq!(
        role_with("Left click", mouse(MouseButton::Left)),
        Some(Role::Left)
    );
    assert_eq!(
        role_with(
            "G6 · DPI shift (1200)",
            ButtonAction::Dpi(DpiFunction::Shift(1200))
        ),
        Some(Role::Sniper)
    );
    assert_eq!(
        role_with("G9 · Profile cycle", ButtonAction::ProfileCycle),
        Some(Role::Profile)
    );
    assert_eq!(role("G11 · Scroll left"), Some(Role::TiltLeft));
    assert_eq!(role("Back"), Some(Role::Back));
    assert_eq!(role("Button 7"), None);
    // The factory action wins over a name that says nothing.
    assert_eq!(
        role_with("Button 4", mouse(MouseButton::Forward)),
        Some(Role::Forward)
    );
}

/// The names `gearwise-logitech`'s `features::control_name` gives 0x1b04 controls.
#[test]
fn logitech_control_names_land_on_their_parts() {
    for (name, want) in [
        ("Left click", Some(Role::Left)),
        ("Right click", Some(Role::Right)),
        ("Middle click", Some(Role::Middle)),
        ("Back", Some(Role::Back)),
        ("Forward", Some(Role::Forward)),
        ("Scroll left", Some(Role::TiltLeft)),
        ("Scroll right", Some(Role::TiltRight)),
        ("Gesture button", Some(Role::Gesture)),
        ("Smart shift", Some(Role::WheelMode)),
        ("Thumb wheel", Some(Role::ThumbWheel)),
        ("DPI switch", Some(Role::DpiCycle)),
        ("DPI change", Some(Role::DpiCycle)),
        ("Control 0x1234", None),
    ] {
        assert_eq!(role(name), want, "{name}");
    }
}

/// Onboard-memory slot names ("G6 · DPI shift (1200)" and the like), as the Logitech driver builds
/// them from the factory action's label.
#[test]
fn logitech_onboard_slot_names_land_on_their_parts() {
    for (name, want) in [
        ("G6 · DPI shift (1200)", Role::Sniper),
        ("G7 · DPI down", Role::DpiDown),
        ("G8 · DPI up", Role::DpiUp),
        ("G9 · Profile cycle", Role::Profile),
        ("G10 · Scroll right", Role::TiltRight),
        ("G11 · Scroll left", Role::TiltLeft),
        ("G4 · Back", Role::Back),
        ("G5 · Forward", Role::Forward),
        ("G12 · Default", Role::GKey(12)),
        ("G7", Role::GKey(7)),
        ("G20 · Macro: Reload", Role::GKey(20)),
    ] {
        assert_eq!(role(name), Some(want), "{name}");
    }
    // A name that merely starts with a "g" is not a G-key.
    assert_eq!(role("Gaming mode"), None);
    assert_eq!(role("G"), None);
}

#[test]
fn right_flank_buttons_are_told_from_left_ones() {
    use ButtonAction::Mouse;
    assert_eq!(
        role_with(
            "Right side rear",
            ButtonAction::Key {
                usage: 0x4e,
                mods: Default::default()
            }
        ),
        Some(Role::RightBack)
    );
    assert_eq!(role("Right side front"), Some(Role::RightForward));
    assert_eq!(role("Forward (right)"), Some(Role::RightForward));
    assert_eq!(role("Right thumb back"), Some(Role::RightBack));
    // Even when the factory action is the plain Back / Forward.
    assert_eq!(
        role_with("Right side back", Mouse(MouseButton::Back)),
        Some(Role::RightBack)
    );
    assert_eq!(role("Left side rear"), Some(Role::Back));
    assert_eq!(role("Left side front"), Some(Role::Forward));
    // The right *click* is not a flank button.
    assert_eq!(role("Right click"), Some(Role::Right));
}

#[test]
fn razer_and_corsair_names_map() {
    for (name, want) in [
        ("Left click", Role::Left),
        ("Right click", Role::Right),
        ("Middle click", Role::Middle),
        ("Back", Role::Back),
        ("Forward", Role::Forward),
        ("Sniper", Role::Sniper),
        ("DPI clutch", Role::Sniper),
        ("DPI cycle", Role::DpiCycle),
        ("DPI up", Role::DpiUp),
        ("DPI down", Role::DpiDown),
        ("DPI stage up", Role::DpiUp),
        ("Profile button", Role::Profile),
        ("Wheel tilt left", Role::TiltLeft),
        ("Tilt right", Role::TiltRight),
        ("Wheel click", Role::Middle),
        ("Side button 1", Role::Grid(0)),
        ("Side button 12", Role::Grid(11)),
        ("Thumb grid 5", Role::Grid(4)),
        ("Mode shift", Role::WheelMode),
        ("Free spin", Role::WheelMode),
    ] {
        assert_eq!(role(name), Some(want), "{name}");
    }
    assert_eq!(role("Side button 99"), None);
    assert_eq!(role("Side button 0"), None);
}

/// Every button any SteelSeries model reports (rivalcfg's tables) lands on a role.
#[test]
fn every_steelseries_button_has_a_role() {
    let mut seen = 0;
    for m in gearwise_steelseries::MODELS {
        let Some(table) = m.spec.buttons else {
            continue;
        };
        for field in table.fields {
            if let gearwise_steelseries::models::Field::Button(b) = field {
                let caps = caps(b.id, b.name, b.default.action());
                assert!(Role::of(&caps).is_some(), "{} · {}", m.name, b.name);
                seen += 1;
            }
        }
    }
    assert!(seen > 20);
}

/// Every button of the core demo mice (Logitech, Razer, Corsair stand-ins) has a role.
#[test]
fn every_demo_button_has_a_role() {
    for dev in gearwise_core::demo::devices() {
        for b in &dev.capabilities().buttons {
            if dev.descriptor().kind == gearwise_core::model::DeviceKind::Mouse {
                assert!(
                    Role::of(b).is_some(),
                    "{}: {}",
                    dev.descriptor().name,
                    b.name
                );
            }
        }
    }
}

#[test]
fn zone_names_map_to_roles() {
    assert_eq!(zone_role("Logo"), ZoneRole::Logo);
    assert_eq!(zone_role("Scroll wheel"), ZoneRole::Wheel);
    assert_eq!(zone_role("Primary"), ZoneRole::Body);
    assert_eq!(zone_role("Underglow"), ZoneRole::Body);
}
