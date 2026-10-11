//! What a mouse button is, anatomically, and which part of a device a lighting zone lights.
//!
//! [`Role`] is the vocabulary shared by every drawing of a mouse: a model gives each physical
//! control a part tagged with the role it plays, and [`Role::of`] turns whatever the four
//! drivers report (control names, factory actions) into the same tags.

use egui::Color32;
use gearwise_core::model::{ButtonAction, ButtonCaps, DpiFunction, MouseButton};

/// Which part of the device a lighting zone lights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneRole {
    Logo,
    Wheel,
    Body,
}

/// Which body band (front, middle, rear) a zone name refers to, if any.
pub fn zone_band(name: &str) -> Option<usize> {
    let n = name.to_ascii_lowercase();
    if n.contains("front") {
        Some(0)
    } else if n.contains("middle") || n.contains("mid ") {
        Some(1)
    } else if n.contains("rear") || n.contains("back") {
        Some(2)
    } else {
        None
    }
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
    /// Front / middle / rear body zones, for mice that light their shell in
    /// bands (SteelSeries Aerox and similar). `None` where a band is off or
    /// the device has no such zone; models fall back to `body`.
    pub bands: [Option<Color32>; 3],
    pub highlight: Option<ZoneRole>,
}

/// Which margin a callout label goes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// What a button (or a wheel) is, anatomically. A model draws one part per role it has;
/// buttons whose role has no part on the drawing fall back to the driver's hotspot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// The primary buttons.
    Left,
    Right,
    /// The wheel click.
    Middle,
    /// The wheel tilted (or scrolled sideways) left / right.
    TiltLeft,
    TiltRight,
    /// The thumb buttons on the left flank (a right-handed mouse's thumb side).
    Back,
    Forward,
    /// The same pair on the right flank of an ambidextrous mouse.
    RightBack,
    RightForward,
    /// DPI shift, sniper or clutch: held for a temporary DPI.
    Sniper,
    DpiUp,
    DpiDown,
    /// A single DPI key that steps through the stages.
    DpiCycle,
    /// Onboard profile cycling.
    Profile,
    /// Wheel mode: Logitech's SmartShift / ratchet toggle, a wheel-unlock switch.
    WheelMode,
    /// A thumb wheel (horizontal scroll). Not a button, but it can be highlighted and labelled.
    ThumbWheel,
    /// The gesture button (Logitech MX thumb rest).
    Gesture,
    /// An extra programmable key printed with this number ("G7", "G12"...).
    GKey(u8),
    /// A cell of a thumb grid or plate, 0-based, row by row (Naga, Scimitar, G600 and the
    /// small 2- and 6-button plates).
    Grid(u8),
}

impl Role {
    /// Derive the role from the button's name and factory action. Right-flank thumb buttons
    /// are recognised by name first, since their factory action is the plain Back / Forward;
    /// then the factory action decides, then the rest of the name.
    pub fn of(b: &ButtonCaps) -> Option<Role> {
        let n = b.name.to_ascii_lowercase();
        right_flank(&n)
            .or_else(|| by_action(&b.default_action))
            .or_else(|| by_name(&n))
    }
}

fn by_action(action: &ButtonAction) -> Option<Role> {
    match action {
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
    }
}

/// "Right side rear", "Forward (right)"...: the thumb pair of the right flank.
fn right_flank(n: &str) -> Option<Role> {
    if !n.contains("right") || !(n.contains("side") || n.contains('(') || n.contains("thumb")) {
        return None;
    }
    let rear = n.contains("rear") || n.contains("back");
    let front = n.contains("front") || n.contains("forward");
    match (rear, front) {
        (true, false) => Some(Role::RightBack),
        (false, true) => Some(Role::RightForward),
        _ => None,
    }
}

fn by_name(n: &str) -> Option<Role> {
    let has = |s: &str| n.contains(s);
    let any = |list: &[&str]| list.iter().any(|s| n.contains(s));
    Some(if has("gesture") {
        Role::Gesture
    } else if any(&["thumb wheel", "thumbwheel", "horizontal scroll"]) {
        Role::ThumbWheel
    } else if any(&[
        "smart shift",
        "smartshift",
        "mode shift",
        "shift wheel",
        "wheel mode",
        "scroll mode",
        "ratchet",
        "free spin",
        "freespin",
        "wheel unlock",
        "wheel lock",
    ]) {
        Role::WheelMode
    } else if any(&["scroll left", "tilt left", "wheel left", "left tilt"]) {
        Role::TiltLeft
    } else if any(&["scroll right", "tilt right", "wheel right", "right tilt"]) {
        Role::TiltRight
    } else if any(&["sniper", "dpi shift", "clutch", "precision"]) {
        Role::Sniper
    } else if let Some(role) = dpi_key(n) {
        role
    } else if has("profile") {
        Role::Profile
    } else if any(&["middle", "wheel click", "scroll click"]) {
        Role::Middle
    } else if any(&["left click", "left button"]) {
        Role::Left
    } else if any(&["right click", "right button"]) {
        Role::Right
    } else if n.ends_with("back") || has("back button") || (has("side") && has("rear")) {
        Role::Back
    } else if n.ends_with("forward") || has("forward button") || (has("side") && has("front")) {
        Role::Forward
    } else {
        return numbered(n);
    })
}

/// A DPI key: up / down by a word or a sign in its name ("DPI up", "DPI stage +"), else the
/// single key that cycles ("DPI button", "DPI switch", "DPI change").
fn dpi_key(n: &str) -> Option<Role> {
    if !["dpi", "cpi", "sensitivity"].iter().any(|s| n.contains(s)) {
        return None;
    }
    let word = |w: &str| n.split(|c: char| !c.is_alphanumeric()).any(|t| t == w);
    Some(
        if word("up") || word("plus") || word("increase") || n.contains('+') {
            Role::DpiUp
        } else if word("down") || word("minus") || word("decrease") || n.contains(" -") {
            Role::DpiDown
        } else {
            Role::DpiCycle
        },
    )
}

/// "Side button 7" (a grid cell) and "G12 · ..." (an extra key).
fn numbered(n: &str) -> Option<Role> {
    for prefix in [
        "side button ",
        "thumb button ",
        "thumb grid ",
        "grid button ",
        "grid ",
    ] {
        if let Some(rest) = n.strip_prefix(prefix) {
            let num: u8 = rest.trim().parse().ok()?;
            return (1..=24).contains(&num).then(|| Role::Grid(num - 1));
        }
    }
    let rest = n.strip_prefix('g')?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let ends_there = rest[digits.len()..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric());
    if digits.is_empty() || !ends_there {
        return None;
    }
    digits.parse().ok().map(Role::GKey)
}

#[cfg(test)]
mod tests;
