//! Geometry checks for the Corsair drawings: that each control sits where the real one does.

use egui::{Rect, pos2, vec2};

use crate::illustration::mouse::{Design, MouseArt};
use crate::illustration::roles::{Role, Side};

fn art(name: &str) -> MouseArt {
    let design = Design::for_name(name);
    assert!(matches!(design, Design::Model(_)), "{name} has a drawing");
    MouseArt::new(
        Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 300.0 / design.aspect())),
        design,
    )
}

fn at(art: &MouseArt, r: Role) -> egui::Pos2 {
    art.part(r)
        .unwrap_or_else(|| panic!("no part for {r:?}"))
        .anchor
}

/// The primary buttons flank the wheel, which sits near the centre line and in front of
/// everything else on top of the shell.
fn front_row_is_in_order(a: &MouseArt) {
    let (l, m, r) = (at(a, Role::Left), at(a, Role::Middle), at(a, Role::Right));
    assert!(l.x < m.x && m.x < r.x, "left, wheel, right in a row");
    assert!(
        (m.x - a.rect.center().x).abs() < a.rect.width() * 0.2,
        "the wheel is near the centre line"
    );
    assert!(a.wheel.contains(m), "the wheel click is on the wheel");
    assert!(
        a.logo.0.y > a.wheel.max.y + a.rect.height() * 0.3,
        "the logo is on the palm"
    );
}

#[test]
fn m65_has_dpi_keys_behind_the_wheel_and_the_sniper_key_on_the_thumb_side() {
    let a = art("Corsair M65 RGB Ultra");
    front_row_is_in_order(&a);
    let cx = a.rect.center().x;
    for r in [Role::DpiUp, Role::DpiDown] {
        assert!(at(&a, r).y > a.wheel.max.y, "{r:?} is behind the wheel");
        assert!((at(&a, r).x - cx).abs() < a.rect.width() * 0.15);
    }
    assert!(at(&a, Role::DpiUp).y < at(&a, Role::DpiDown).y);
    for r in [Role::Sniper, Role::Forward, Role::Back] {
        assert!(
            at(&a, r).x < cx - a.rect.width() * 0.25,
            "{r:?} is on the left flank"
        );
    }
    assert!(at(&a, Role::Forward).y < at(&a, Role::Back).y);
    assert!(
        at(&a, Role::Sniper).x < at(&a, Role::Forward).x,
        "the sniper key is outermost"
    );
    for r in [
        Role::Profile,
        Role::TiltLeft,
        Role::Grid(0),
        Role::RightBack,
    ] {
        assert!(a.part(r).is_none(), "{r:?} is not on an M65");
    }
}

#[test]
fn dark_core_has_the_profile_light_above_the_dpi_key_and_thumb_keys_left() {
    let a = art("Corsair Dark Core RGB Pro SE");
    front_row_is_in_order(&a);
    assert!(at(&a, Role::Profile).y > a.wheel.max.y);
    assert!(at(&a, Role::Profile).y < at(&a, Role::DpiCycle).y);
    let cx = a.rect.center().x;
    for r in [Role::Sniper, Role::Forward, Role::Back] {
        assert!(
            at(&a, r).x < cx - a.rect.width() * 0.15,
            "{r:?} is on the thumb side"
        );
    }
    assert!(at(&a, Role::Sniper).y < at(&a, Role::Forward).y);
    assert!(at(&a, Role::Forward).y < at(&a, Role::Back).y);
}

#[test]
fn katar_and_harpoon_are_small_symmetric_shells_with_a_single_dpi_key() {
    for name in ["Corsair Katar Pro XT", "Corsair Harpoon RGB Wireless"] {
        let a = art(name);
        front_row_is_in_order(&a);
        assert!(
            at(&a, Role::DpiCycle).y > a.wheel.max.y,
            "{name}: DPI key behind the wheel"
        );
        let cx = a.rect.center().x;
        assert!((at(&a, Role::DpiCycle).x - cx).abs() < a.rect.width() * 0.15);
        assert!(at(&a, Role::Forward).x < cx && at(&a, Role::Back).x < cx);
        assert!(at(&a, Role::Forward).y < at(&a, Role::Back).y);
        assert!(a.part(Role::Sniper).is_none() && a.part(Role::Profile).is_none());
    }
}

/// The Scimitar's twelve side keys: three columns by four rows, numbered row by row from the
/// front, in a panel on the left flank whose labels go in the left margin.
#[test]
fn scimitar_has_twelve_side_keys_in_a_three_by_four_grid_on_the_left() {
    let a = art("Corsair Scimitar RGB Elite");
    front_row_is_in_order(&a);
    let key = |i: u8| at(&a, Role::Grid(i));
    for i in 0..12u8 {
        assert!(
            key(i).x < at(&a, Role::Left).x,
            "key {i} is left of the left button"
        );
        assert_eq!(a.part(Role::Grid(i)).unwrap().side, Some(Side::Left));
    }
    assert!(a.part(Role::Grid(12)).is_none());
    for row in 0..4u8 {
        let (a0, a1, a2) = (key(row * 3), key(row * 3 + 1), key(row * 3 + 2));
        assert!(a0.x < a1.x && a1.x < a2.x, "row {row} runs left to right");
        assert!((a0.y - a2.y).abs() < 1.0, "row {row} is level");
        if row > 0 {
            assert!(key(row * 3).y > key((row - 1) * 3).y, "rows go back");
        }
    }
    assert!(at(&a, Role::DpiUp).y < at(&a, Role::DpiDown).y);
    assert!(at(&a, Role::DpiUp).y > a.wheel.max.y);
    // No thumb pair or sniper key: the panel is the thumb rest.
    assert!(a.part(Role::Forward).is_none() && a.part(Role::Sniper).is_none());
}

#[test]
fn every_corsair_family_is_matched_by_its_product_names() {
    let id = |n: &str| match Design::for_name(n) {
        Design::Model(m) => Some(m.id()),
        Design::Generic(_) => None,
    };
    assert_eq!(id("Corsair M65 PRO RGB"), Some("m65"));
    assert_eq!(id("Corsair M65 RGB Elite"), Some("m65"));
    assert_eq!(id("Corsair Dark Core RGB Pro"), Some("dark-core"));
    assert_eq!(id("Corsair Scimitar PRO RGB"), Some("scimitar"));
    assert_eq!(id("Corsair Katar Pro"), Some("katar"));
    assert_eq!(id("Corsair Harpoon RGB Pro"), Some("harpoon"));
    // Mice without a drawing keep their generic silhouette.
    assert_eq!(id("Corsair M75 Wireless"), None);
    assert_eq!(id("Corsair Ironclaw RGB Wireless"), None);
}
