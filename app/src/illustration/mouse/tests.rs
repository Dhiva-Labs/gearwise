use super::*;
use gearwise_core::model::{ButtonAction, DpiFunction, MouseButton};

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
fn silhouettes_follow_the_product_name() {
    assert_eq!(Kind::for_name("G502 X Plus (demo)"), Kind::G502);
    assert_eq!(Kind::for_name("G502 HERO Gaming Mouse"), Kind::G502);
    assert_eq!(Kind::for_name("Corsair M65 RGB Ultra"), Kind::Ergonomic);
    assert_eq!(Kind::for_name("Razer Basilisk V3"), Kind::Ergonomic);
    assert_eq!(Kind::for_name("Corsair Dark Core RGB"), Kind::Ergonomic);
    assert_eq!(Kind::for_name("G Pro X Superlight"), Kind::Ambidextrous);
    assert_eq!(Kind::for_name("Razer Viper Ultimate"), Kind::Ambidextrous);
    assert_eq!(Kind::for_name("Razer Naga Pro"), Kind::Mmo);
    assert_eq!(Kind::for_name("Corsair Scimitar Elite"), Kind::Mmo);
    assert_eq!(Kind::for_name("DeathAdder V3"), Kind::ErgoLite);
    assert_eq!(Kind::for_name("Some Unknown Mouse"), Kind::ErgoLite);
}

/// The G502 HERO as drivers report it: eleven buttons, five with hotspots.
fn g502_hero() -> Vec<ButtonCaps> {
    let mouse = |b| ButtonAction::Mouse(b);
    let mut v = vec![
        caps(0, "Left click", mouse(MouseButton::Left)),
        caps(1, "Right click", mouse(MouseButton::Right)),
        caps(2, "Middle click", mouse(MouseButton::Middle)),
        caps(3, "Back", mouse(MouseButton::Back)),
        caps(4, "Forward", mouse(MouseButton::Forward)),
        caps(
            5,
            "G6 · DPI shift (1200)",
            ButtonAction::Dpi(DpiFunction::Shift(1200)),
        ),
        caps(6, "G7 · DPI down", ButtonAction::Dpi(DpiFunction::Down)),
        caps(7, "G8 · DPI up", ButtonAction::Dpi(DpiFunction::Up)),
        caps(8, "G9 · Profile cycle", ButtonAction::ProfileCycle),
        caps(9, "G10 · Scroll right", mouse(MouseButton::WheelRight)),
        caps(10, "G11 · Scroll left", mouse(MouseButton::WheelLeft)),
    ];
    for (b, h) in v
        .iter_mut()
        .zip([(0.3, 0.2), (0.7, 0.2), (0.5, 0.17), (0.1, 0.5), (0.11, 0.4)])
    {
        b.hotspot = Some(h);
    }
    v
}

#[test]
fn every_g502_hero_button_gets_an_anatomical_anchor() {
    for kind in [Kind::G502, Kind::Ergonomic] {
        let art = MouseArt::new(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 300.0 / kind.aspect())),
            kind,
        );
        let placed = art.place(&g502_hero());
        assert_eq!(placed.len(), 11, "{kind:?}");
        assert!(
            placed.iter().all(|p| p.role.is_some()),
            "roles beat raw hotspots"
        );
        assert!(
            placed
                .iter()
                .all(|p| art.rect.expand(2.0).contains(p.anchor))
        );
        // Back sits on the left flank, below the sniper button; the right button is right of centre.
        let at = |r: Role| placed.iter().find(|p| p.role == Some(r)).unwrap().anchor;
        assert!(at(Role::Back).x < art.rect.center().x && at(Role::Back).y > at(Role::Sniper).y);
        assert!(at(Role::Right).x > art.rect.center().x);
    }
}

/// The real G502 layout: G7/G8 left of the left button, the wheel tilts either side of
/// the wheel, the profile key behind it and the thumb buttons on the left edge.
#[test]
fn g502_parts_sit_where_the_real_buttons_are() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(290.0, 290.0 / Kind::G502.aspect()));
    let art = MouseArt::new(rect, Kind::G502);
    let placed = art.place(&g502_hero());
    let at = |r: Role| art.part(r).unwrap().anchor;
    assert_eq!(placed.len(), 11);
    let cx = rect.center().x;
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

#[test]
fn buttons_without_a_matching_part_fall_back_to_hotspots_or_the_grid() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
    // Sniper has no part on an ambidextrous mouse, so its hotspot is used.
    let mut sniper = caps(1, "Sniper", ButtonAction::Dpi(DpiFunction::Shift(400)));
    sniper.hotspot = Some((0.1, 0.3));
    let placed = MouseArt::new(rect, Kind::Ambidextrous).place(&[sniper.clone()]);
    assert_eq!(
        (placed[0].role, placed[0].anchor),
        (None, pos2(30.0, 129.0))
    );
    // Without a hotspot it is left out of the callouts.
    sniper.hotspot = None;
    assert!(
        MouseArt::new(rect, Kind::Ambidextrous)
            .place(&[sniper])
            .is_empty()
    );
    // MMO mice give unknown buttons a grid cell each, in order.
    let unknown: Vec<ButtonCaps> = (0..14)
        .map(|i| caps(i, &format!("Button {i}"), ButtonAction::Default))
        .collect();
    let placed = MouseArt::new(rect, Kind::Mmo).place(&unknown);
    assert_eq!(placed.len(), 12);
    assert_eq!(placed[0].role, Some(Role::Grid(0)));
    assert_eq!(placed[11].role, Some(Role::Grid(11)));
}

#[test]
fn each_role_is_used_once() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
    let twice = vec![
        caps(1, "Left click", ButtonAction::Mouse(MouseButton::Left)),
        caps(
            2,
            "Left click again",
            ButtonAction::Mouse(MouseButton::Left),
        ),
    ];
    assert_eq!(MouseArt::new(rect, Kind::ErgoLite).place(&twice).len(), 1);
}

#[test]
fn parts_stay_inside_the_shell_box() {
    for kind in [
        Kind::G502,
        Kind::Ergonomic,
        Kind::ErgoLite,
        Kind::Ambidextrous,
        Kind::Mmo,
    ] {
        let rect = Rect::from_min_size(pos2(50.0, 40.0), vec2(300.0 * kind.aspect() / 0.66, 455.0));
        let art = MouseArt::new(rect, kind);
        for part in &art.parts {
            assert!(
                part.poly.iter().all(|q| rect.expand(3.0).contains(*q)),
                "{kind:?} {:?} leaves the box",
                part.role
            );
        }
        let outline = art.geo.outline();
        let (min, max) = outline
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q.x), b.max(q.x)));
        assert!(
            (min - rect.min.x).abs() < 1.0 && (max - rect.max.x).abs() < 1.0,
            "{kind:?} spans the full width"
        );
    }
}

#[test]
fn ergonomic_shell_is_wider_on_the_thumb_side() {
    let p = Kind::Ergonomic.profile();
    let (l, r) = p.edges(0.6);
    assert!(-l > r * 1.1, "thumb rest bulges on the left");
    let (l, r) = Kind::Ambidextrous.profile().edges(0.6);
    assert!((-l - r).abs() < 1e-4, "ambidextrous is symmetric");
}
