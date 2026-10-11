//! Geometry checks for the Razer drawings: that each control sits where the real one does, that
//! the right model is picked for each product name and that the lighting zones light the part
//! the driver's LED names say they do.

use egui::{Color32, Pos2, RawInput, Rect, pos2, vec2};
use gearwise_core::model::{ButtonAction, ButtonCaps, MouseButton};

use crate::illustration::models::lookup;
use crate::illustration::mouse::{Design, MouseArt};
use crate::illustration::roles::{MouseLook, Role};
use crate::theme::{Palette, apply};
use gearwise_core::profile::Theme;

fn art(name: &str, width: f32) -> MouseArt {
    let design = Design::for_name(name);
    assert!(
        matches!(design, Design::Model(_)),
        "{name} has no model art"
    );
    MouseArt::new(
        Rect::from_min_size(pos2(0.0, 0.0), vec2(width, width / design.aspect())),
        design,
    )
}

fn at(art: &MouseArt, role: Role) -> Pos2 {
    art.part(role)
        .unwrap_or_else(|| panic!("no part for {role:?}"))
        .anchor
}

fn id(name: &str) -> Option<&'static str> {
    lookup(name).map(|m| m.id())
}

#[test]
fn product_names_pick_the_right_drawing() {
    assert_eq!(
        id("Razer DeathAdder V3 Pro (Wireless)"),
        Some("razer-deathadder-v3")
    );
    assert_eq!(id("Razer DeathAdder V2"), Some("razer-deathadder-v3"));
    assert_eq!(id("DeathAdder"), Some("razer-deathadder-v3"));
    assert_eq!(id("Razer Viper V3 Pro (Wired)"), Some("razer-viper-v3-pro"));
    assert_eq!(id("Razer Viper V3 HyperSpeed"), Some("razer-viper-v3-pro"));
    assert_eq!(
        id("Razer Viper Ultimate (Wireless)"),
        Some("razer-viper-ultimate")
    );
    assert_eq!(
        id("Razer Viper V2 Pro (Wired)"),
        Some("razer-viper-ultimate")
    );
    assert_eq!(id("Viper"), Some("razer-viper-ultimate"));
    assert_eq!(
        id("Razer Basilisk V3 Pro (Wired)"),
        Some("razer-basilisk-v3")
    );
    assert_eq!(id("Razer Basilisk V3"), Some("razer-basilisk-v3"));
    assert_eq!(
        id("Razer Naga V2 Pro (Wireless)"),
        Some("razer-naga-v2-pro")
    );
    assert_eq!(id("Razer Naga Pro (Wired)"), Some("razer-naga-v2-pro"));
    assert_eq!(id("Razer Cobra Pro (Wired)"), Some("razer-cobra-pro"));
    assert_eq!(id("Razer Cobra"), Some("razer-cobra-pro"));
    // Other makers' mice (and Razer's other devices) are left to their own drawings.
    assert_eq!(id("Razer BlackWidow V4 Pro"), None);
}

/// The thumb buttons of a right-handed mouse are on the left flank, the front one ahead of the
/// back one; the wheel sits between the primary buttons and the logo is on the palm.
fn right_handed_basics(name: &str) {
    let art = art(name, 300.0);
    let cx = art.rect.center().x;
    assert!(at(&art, Role::Left).x < at(&art, Role::Middle).x, "{name}");
    assert!(at(&art, Role::Middle).x < at(&art, Role::Right).x, "{name}");
    for r in [Role::Forward, Role::Back] {
        assert!(
            at(&art, r).x < cx - art.rect.width() * 0.25,
            "{name}: {r:?}"
        );
    }
    assert!(at(&art, Role::Forward).y < at(&art, Role::Back).y, "{name}");
    assert!(
        art.logo.0.y > at(&art, Role::Back).y,
        "{name}: the logo is on the palm"
    );
    assert!(art.logo.0.y > art.wheel.max.y, "{name}");
    assert!(
        (art.logo.0.x - cx).abs() < art.rect.width() * 0.15,
        "{name}"
    );
    // None of these mice has a right-hand thumb pair or a thumb grid.
    for r in [Role::RightBack, Role::RightForward, Role::Grid(0)] {
        assert!(art.part(r).is_none(), "{name}: {r:?}");
    }
}

#[test]
fn deathadder_and_viper_v3_have_a_left_thumb_pair_and_nothing_else() {
    for name in [
        "Razer DeathAdder V3 Pro (Wired)",
        "Razer Viper V3 Pro (Wired)",
    ] {
        right_handed_basics(name);
        let art = art(name, 300.0);
        // Five controls: no tilt, DPI keys or clutch on the top.
        for r in [
            Role::TiltLeft,
            Role::DpiUp,
            Role::DpiCycle,
            Role::Sniper,
            Role::WheelMode,
        ] {
            assert!(art.part(r).is_none(), "{name}: {r:?}");
        }
    }
}

#[test]
fn the_viper_ultimate_mirrors_its_thumb_buttons_on_both_flanks() {
    let art = art("Razer Viper Ultimate (Wired)", 300.0);
    let cx = art.rect.center().x;
    let w = art.rect.width();
    for (left, right) in [
        (Role::Forward, Role::RightForward),
        (Role::Back, Role::RightBack),
    ] {
        let (l, r) = (at(&art, left), at(&art, right));
        assert!(
            l.x < cx - w * 0.3 && r.x > cx + w * 0.3,
            "{left:?} / {right:?}"
        );
        assert!((l.y - r.y).abs() < 2.0, "{left:?} and {right:?} are level");
        assert!(
            ((cx - l.x) - (r.x - cx)).abs() < w * 0.03,
            "{left:?} / {right:?} mirror"
        );
    }
    assert!(at(&art, Role::Forward).y < at(&art, Role::Back).y);
    assert!(at(&art, Role::RightForward).y < at(&art, Role::RightBack).y);
    // The thumb buttons sit behind the wheel and level with the V-shaped seam.
    assert!(at(&art, Role::Forward).y > art.wheel.max.y);
}

#[test]
fn the_viper_v3_pro_has_no_right_flank_buttons() {
    let art = art("Razer Viper V3 Pro (Wired)", 300.0);
    assert!(art.part(Role::RightBack).is_none() && art.part(Role::RightForward).is_none());
}

#[test]
fn the_basilisk_has_its_wheel_keys_and_thumb_clutch_where_the_real_ones_are() {
    let art = art("Razer Basilisk V3 Pro (Wired)", 300.0);
    let cx = art.rect.center().x;
    let wheel = art.wheel;
    assert!(at(&art, Role::TiltLeft).x < wheel.min.x);
    assert!(at(&art, Role::TiltRight).x > wheel.max.x);
    // HyperScroll's mode key, then the DPI key, stacked behind the wheel on the centre line.
    assert!(at(&art, Role::WheelMode).y > wheel.max.y);
    assert!(at(&art, Role::DpiCycle).y > at(&art, Role::WheelMode).y);
    assert!((at(&art, Role::WheelMode).x - at(&art, Role::DpiCycle).x).abs() < 6.0);
    // The clutch is the front thumb key, ahead of Forward and Back, all on the left wall.
    assert!(at(&art, Role::Sniper).y < at(&art, Role::Forward).y);
    assert!(at(&art, Role::Forward).y < at(&art, Role::Back).y);
    for r in [Role::Sniper, Role::Forward, Role::Back] {
        assert!(at(&art, r).x < cx - art.rect.width() * 0.2, "{r:?}");
    }
    assert!(art.logo.0.y > at(&art, Role::Back).y);
}

#[test]
fn the_naga_plate_is_twelve_cells_in_four_rows_of_three_on_the_left() {
    let art = art("Razer Naga V2 Pro (Wired)", 300.0);
    let cx = art.rect.center().x;
    let cell = |i: u8| at(&art, Role::Grid(i));
    for i in 0..12 {
        assert!(
            cell(i).x < cx - art.rect.width() * 0.25,
            "cell {i} is on the thumb side"
        );
    }
    assert!(art.part(Role::Grid(12)).is_none(), "twelve cells, no more");
    for row in 0..4u8 {
        let (a, b, c) = (cell(row * 3), cell(row * 3 + 1), cell(row * 3 + 2));
        assert!(a.x < b.x && b.x < c.x, "row {row} runs outwards to inwards");
        // A little shear aside, a row is level.
        assert!((a.y - c.y).abs() < 4.0, "row {row}");
        if row > 0 {
            assert!(
                cell(row * 3).y > cell(row * 3 - 3).y + 8.0,
                "rows run front to back"
            );
        }
    }
    // DPI up above DPI down, both behind the wheel; tilt either side of it.
    assert!(at(&art, Role::DpiUp).y > art.wheel.max.y);
    assert!(at(&art, Role::DpiUp).y < at(&art, Role::DpiDown).y);
    assert!(
        at(&art, Role::TiltLeft).x < art.wheel.min.x
            && at(&art, Role::TiltRight).x > art.wheel.max.x
    );
    // The plate replaces the Back / Forward buttons of the other mice.
    assert!(art.part(Role::Back).is_none() && art.part(Role::Forward).is_none());
}

#[test]
fn the_cobra_has_a_dpi_key_behind_the_wheel_and_a_left_thumb_pair() {
    let art = art("Razer Cobra Pro (Wired)", 300.0);
    right_handed_basics("Razer Cobra Pro (Wired)");
    assert!(at(&art, Role::DpiCycle).y > art.wheel.max.y);
    assert!(at(&art, Role::DpiCycle).y < at(&art, Role::Forward).y + art.rect.height() * 0.1);
    for r in [Role::TiltLeft, Role::Sniper, Role::WheelMode, Role::Grid(0)] {
        assert!(art.part(r).is_none(), "{r:?}");
    }
}

/// The lighting zones the driver names, by LED: "Logo", "Scroll wheel", and the underglow strip
/// ("Underglow" or "Backlight"). Each must change the drawing, on the models that have the part.
#[test]
fn lighting_zones_light_the_parts_the_driver_names() {
    fn shapes_for(name: &str, look: &MouseLook) -> usize {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 700.0));
        let ctx = egui::Context::default();
        let pal = Palette::new(Theme::Dark, Color32::from_rgb(0, 212, 170));
        apply(&ctx, &pal);
        let design = Design::for_name(name);
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                let rect = crate::illustration::mouse::art_rect(screen.shrink(60.0), design);
                MouseArt::new(rect, design).paint(ui.painter(), &pal, look, &[]);
            },
        );
        out.textures_delta.clear();
        out.shapes.len()
    }
    let off = MouseLook::default();
    let on = Some(Color32::from_rgb(255, 0, 80));
    let all = [
        "Razer DeathAdder V3 Pro",
        "Razer Viper V3 Pro",
        "Razer Viper Ultimate",
        "Razer Basilisk V3 Pro",
        "Razer Naga V2 Pro",
        "Razer Cobra Pro",
    ];
    for name in all {
        let plain = shapes_for(name, &off);
        let logo = shapes_for(name, &MouseLook { logo: on, ..off });
        let wheel = shapes_for(name, &MouseLook { wheel: on, ..off });
        assert!(logo > plain, "{name}: the logo zone lights the logo");
        assert!(wheel > plain, "{name}: the wheel zone lights the wheel");
    }
    for name in [
        "Razer Basilisk V3 Pro",
        "Razer Naga V2 Pro",
        "Razer Cobra Pro",
    ] {
        let plain = shapes_for(name, &off);
        let body = shapes_for(name, &MouseLook { body: on, ..off });
        assert!(
            body > plain,
            "{name}: the body zone lights the underglow / thumb plate"
        );
    }
}

/// A Razer mouse reports no remappable buttons in this app, but the demo devices list the
/// physical ones: each name must resolve to the part it is named after.
#[test]
fn physical_button_names_resolve_to_their_parts() {
    let caps = |id: u16, name: &str, default: ButtonAction| ButtonCaps {
        id,
        name: name.into(),
        remappable: true,
        default_action: default,
        hotspot: None,
    };
    let art = art("Razer Naga V2 Pro (Wired)", 300.0);
    let buttons: Vec<ButtonCaps> = std::iter::once(caps(
        0,
        "Left click",
        ButtonAction::Mouse(MouseButton::Left),
    ))
    .chain((1..=12).map(|n| caps(n, &format!("Side button {n}"), ButtonAction::Default)))
    .collect();
    let placed = art.place(&buttons);
    assert_eq!(placed.len(), 13);
    assert_eq!(placed[1].role, Some(Role::Grid(0)));
    assert_eq!(placed[12].role, Some(Role::Grid(11)));
}
