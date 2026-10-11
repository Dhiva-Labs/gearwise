//! Geometry checks for the SteelSeries drawings: that each control sits where the real one does.

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

fn id(name: &str) -> Option<&'static str> {
    match Design::for_name(name) {
        Design::Model(m) => Some(m.id()),
        Design::Generic(_) => None,
    }
}

fn at(art: &MouseArt, r: Role) -> egui::Pos2 {
    art.part(r)
        .unwrap_or_else(|| panic!("no part for {r:?}"))
        .anchor
}

fn front_row_is_in_order(a: &MouseArt) {
    let (l, m, r) = (at(a, Role::Left), at(a, Role::Middle), at(a, Role::Right));
    assert!(l.x < m.x && m.x < r.x, "left, wheel, right in a row");
    assert!(a.wheel.contains(m), "the wheel click is on the wheel");
    assert!(
        (m.x - a.rect.center().x).abs() < a.rect.width() * 0.2,
        "the wheel is near the centre line"
    );
}

#[test]
fn matchers_pick_the_right_drawing_for_each_product_name() {
    assert_eq!(id("SteelSeries Rival 3"), Some("rival-3"));
    assert_eq!(id("SteelSeries Rival 3 Wireless"), Some("rival-3"));
    // The Rival 300 family is a different shell and keeps the generic silhouette.
    assert_eq!(id("SteelSeries Rival 300"), None);
    assert_eq!(id("SteelSeries Rival 310"), Some("rival-600"));
    assert_eq!(id("SteelSeries Rival 600"), Some("rival-600"));
    assert_eq!(id("SteelSeries Rival 650 Wireless"), Some("rival-600"));
    assert_eq!(id("SteelSeries Aerox 3 Wireless"), Some("aerox-3"));
    assert_eq!(id("SteelSeries Aerox 5"), Some("aerox-5"));
    assert_eq!(id("SteelSeries Aerox 9 Wireless"), Some("aerox-9"));
    assert_eq!(id("SteelSeries Aerox"), Some("aerox-3"));
    assert_eq!(id("SteelSeries Prime Mini"), Some("prime"));
    assert_eq!(id("SteelSeries Prime+"), Some("prime"));
    assert_eq!(id("SteelSeries Sensei 310"), Some("sensei"));
    assert_eq!(id("SteelSeries Rival 100"), None);
}

#[test]
fn rival_3_prime_and_rival_600_have_the_standard_front_and_a_thumb_pair() {
    for name in [
        "SteelSeries Rival 3",
        "SteelSeries Prime",
        "SteelSeries Rival 600",
    ] {
        let a = art(name);
        front_row_is_in_order(&a);
        let cx = a.rect.center().x;
        let (f, b) = (at(&a, Role::Forward), at(&a, Role::Back));
        assert!(f.x < cx - a.rect.width() * 0.3 && b.x < cx - a.rect.width() * 0.3);
        assert!(f.y < b.y, "{name}: Forward is in front of Back");
        assert!(a.part(Role::RightBack).is_none() && a.part(Role::Grid(0)).is_none());
    }
    // The Prime's DPI key is underneath, so it has no part on the top view.
    assert!(art("SteelSeries Prime").part(Role::DpiCycle).is_none());
    for name in ["SteelSeries Rival 3", "SteelSeries Rival 600"] {
        let a = art(name);
        assert!(
            at(&a, Role::DpiCycle).y > a.wheel.max.y,
            "{name}: DPI key behind the wheel"
        );
    }
}

#[test]
fn sensei_has_a_thumb_pair_on_both_flanks() {
    let a = art("SteelSeries Sensei Ten");
    front_row_is_in_order(&a);
    let cx = a.rect.center().x;
    for (left, right) in [
        (Role::Forward, Role::RightForward),
        (Role::Back, Role::RightBack),
    ] {
        let (l, r) = (at(&a, left), at(&a, right));
        assert!(
            l.x < cx && r.x > cx,
            "{left:?} / {right:?} are on opposite flanks"
        );
        assert!((l.y - r.y).abs() < 1.0, "the flanks mirror each other");
        assert!(((cx - l.x) - (r.x - cx)).abs() < 3.0);
    }
    assert!(at(&a, Role::RightForward).y < at(&a, Role::RightBack).y);
    assert!(at(&a, Role::DpiCycle).y > a.wheel.max.y);
}

#[test]
fn aerox_models_differ_in_their_side_keys() {
    let a3 = art("SteelSeries Aerox 3");
    let a5 = art("SteelSeries Aerox 5 Wireless");
    let a9 = art("SteelSeries Aerox 9 Wireless");
    for a in [&a3, &a5, &a9] {
        front_row_is_in_order(a);
        assert!(
            at(a, Role::DpiCycle).y > a.wheel.max.y,
            "the DPI key is behind the wheel"
        );
    }
    for a in [&a3, &a5] {
        assert!(at(a, Role::Forward).y < at(a, Role::Back).y);
        assert!(a.part(Role::Grid(0)).is_none());
    }
    // The 9 trades the thumb pair for a panel of twelve keys, three columns by four rows.
    assert!(a9.part(Role::Forward).is_none());
    for i in 0..12u8 {
        let p = a9.part(Role::Grid(i)).expect("grid key");
        assert!(p.anchor.x < at(&a9, Role::Left).x);
        assert_eq!(p.side, Some(Side::Left));
    }
    assert!(a9.part(Role::Grid(12)).is_none());
    let key = |i: u8| at(&a9, Role::Grid(i));
    assert!(key(0).x < key(1).x && key(1).x < key(2).x);
    assert!(key(3).y > key(0).y && key(9).y > key(6).y);
}
