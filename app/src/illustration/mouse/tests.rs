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

fn model_id(name: &str) -> Option<&'static str> {
    match Design::for_name(name) {
        Design::Model(m) => Some(m.id()),
        Design::Generic(_) => None,
    }
}

#[test]
fn registered_models_take_precedence_over_the_generic_silhouettes() {
    assert_eq!(model_id("G502 X Plus (demo)"), Some("g502-x"));
    assert_eq!(model_id("G502 HERO Gaming Mouse"), Some("g502"));
    assert_eq!(model_id("MX Master 3S"), Some("mx-master"));
    assert_eq!(model_id("Corsair M65 RGB Ultra"), Some("m65"));
    assert!(matches!(
        Design::for_name("Some Unknown Mouse"),
        Design::Generic(Kind::ErgoLite)
    ));
    // Art boxes follow the model's proportions and leave room for its cable.
    let g502 = Design::for_name("G502 HERO Gaming Mouse");
    let rect = art_rect(
        Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 600.0)),
        g502,
    );
    assert!((rect.width() / rect.height() - g502.aspect()).abs() < 1e-3);
    assert!(rect.min.y > 0.0 && g502.overhang() > 0.0);
}

#[test]
fn silhouettes_follow_the_product_name() {
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
    for design in [
        Design::for_name("G502 HERO Gaming Mouse"),
        Design::Generic(Kind::Ergonomic),
    ] {
        let art = MouseArt::new(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 300.0 / design.aspect())),
            design,
        );
        let placed = art.place(&g502_hero());
        assert_eq!(placed.len(), 11, "{design:?}");
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

#[test]
fn buttons_without_a_matching_part_fall_back_to_hotspots_or_the_grid() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
    // Sniper has no part on an ambidextrous mouse, so its hotspot is used.
    let mut sniper = caps(1, "Sniper", ButtonAction::Dpi(DpiFunction::Shift(400)));
    sniper.hotspot = Some((0.1, 0.3));
    let placed = MouseArt::new(rect, Design::Generic(Kind::Ambidextrous)).place(&[sniper.clone()]);
    assert_eq!(
        (placed[0].role, placed[0].anchor),
        (None, pos2(30.0, 129.0))
    );
    // Without a hotspot it is left out of the callouts.
    sniper.hotspot = None;
    assert!(
        MouseArt::new(rect, Design::Generic(Kind::Ambidextrous))
            .place(&[sniper])
            .is_empty()
    );
    // MMO mice give unknown buttons a grid cell each, in order.
    let unknown: Vec<ButtonCaps> = (0..14)
        .map(|i| caps(i, &format!("Button {i}"), ButtonAction::Default))
        .collect();
    let placed = MouseArt::new(rect, Design::Generic(Kind::Mmo)).place(&unknown);
    assert_eq!(placed.len(), 12);
    assert_eq!(placed[0].role, Some(Role::Grid(0)));
    assert_eq!(placed[11].role, Some(Role::Grid(11)));
}

/// Buttons named after a grid cell take that cell; the rest fill the free ones in order.
#[test]
fn named_grid_buttons_keep_their_cell() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
    let buttons = vec![
        caps(1, "Side button 3", ButtonAction::Default),
        caps(2, "Mystery", ButtonAction::Default),
        caps(3, "Side button 1", ButtonAction::Default),
        caps(4, "Another mystery", ButtonAction::Default),
    ];
    let placed = MouseArt::new(rect, Design::Generic(Kind::Mmo)).place(&buttons);
    let role = |id| placed.iter().find(|p| p.id == id).unwrap().role;
    assert_eq!(role(1), Some(Role::Grid(2)));
    assert_eq!(role(3), Some(Role::Grid(0)));
    assert_eq!(role(2), Some(Role::Grid(1)), "first free cell");
    assert_eq!(role(4), Some(Role::Grid(3)), "cell 2 is taken");
}

/// Mice without a grid never invent grid cells for unknown buttons.
#[test]
fn unknown_buttons_stay_out_without_a_grid() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
    let unknown = vec![caps(1, "Button 9", ButtonAction::Default)];
    for kind in [Kind::Ergonomic, Kind::ErgoLite, Kind::Ambidextrous] {
        let art = MouseArt::new(rect, Design::Generic(kind));
        assert!(art.place(&unknown).is_empty());
    }
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
    assert_eq!(
        MouseArt::new(rect, Design::Generic(Kind::ErgoLite))
            .place(&twice)
            .len(),
        1
    );
}

#[test]
fn generic_parts_stay_inside_the_shell_box() {
    for kind in [
        Kind::Ergonomic,
        Kind::ErgoLite,
        Kind::Ambidextrous,
        Kind::Mmo,
    ] {
        let rect = Rect::from_min_size(pos2(50.0, 40.0), vec2(300.0 * kind.aspect() / 0.66, 455.0));
        let art = MouseArt::new(rect, Design::Generic(kind));
        for part in &art.parts {
            assert!(
                part.poly.iter().all(|q| rect.expand(3.0).contains(*q)),
                "{kind:?} {:?} leaves the box",
                part.role
            );
        }
        let outline = Geo::new(rect, kind.profile()).outline();
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
