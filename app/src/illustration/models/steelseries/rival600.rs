//! The Rival 600 / 650 family: an ergonomic right-hand shell with two big buttons, a cable gland
//! between them with the wheel in a lit window behind it and a long DPI key. Below the buttons
//! the body narrows into two side panels, each edged by a light strip that runs down and out
//! to the rear in three segments (the six "strip" zones), with the logo on the rear cover. Two
//! thumb keys sit mid-flank on the left. The preview has one colour for all the strips, since
//! the zones share the "body" role. Geometry lives in `rival600_shapes`.

use egui::{Painter, Pos2, Rect, Shape, vec2};

use super::rival600_shapes::{self as g};
use crate::illustration::models::corsair::common::{
    self, groove, key, light_line, rim, rubber_wheel, sheen,
};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{centroid, convex_hull, glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.13;

pub struct Rival600;

impl ModelArt for Rival600 {
    fn id(&self) -> &'static str {
        "rival-600"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(
            name,
            &[
                "rival 600",
                "rival 650",
                "rival 310",
                "rival 500",
                "rival 700",
            ],
        )
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "SteelSeries Rival 600",
            "SteelSeries Rival 600 Dota 2 Edition",
            "SteelSeries Rival 650 Wireless",
            "SteelSeries Rival 310",
            "SteelSeries Rival 310 CS:GO Howl Edition",
            "SteelSeries Rival 600 (demo)",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn overhang(&self) -> f32 {
        CABLE
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
        let s = f.s();
        let wheel = f.rect_px(g::WHEEL);
        let part = |role, poly: &[P]| {
            let pts = f.pts(poly);
            let c = centroid(&pts);
            Part::new(role, pts).anchored(c)
        };
        Anatomy {
            parts: vec![
                Part::new(Role::Left, f.pts(g::LEFT_BTN)).anchored(f.px(g::ANCHOR_LEFT)),
                Part::new(Role::Right, f.pts(g::RIGHT_BTN)).anchored(f.px(g::ANCHOR_RIGHT)),
                Part::new(Role::Middle, rounded_poly(wheel, 6.0 * s, 4)).anchored(wheel.center()),
                part(Role::DpiCycle, g::DPI_KEY),
                part(Role::Forward, g::FORWARD),
                part(Role::Back, g::BACK),
            ],
            wheel,
            logo: (f.px((g::LOGO.0, g::LOGO.1)), g::LOGO.2 * s),
        }
    }

    fn paint(
        &self,
        art: &MouseArt,
        p: &Painter,
        pal: &Palette,
        look: &MouseLook,
        tint: &[(Role, f32)],
    ) {
        let f = Frame::new(art.rect, g::ORIGIN, g::SIZE);
        let s = f.s();
        let sk = Plastic::black(pal);
        let outline = f.pts(g::OUTLINE);

        kit::shadow(p, &outline, art.rect, pal);
        // The cable and its gland between the buttons.
        let gland = Rect::from_min_max(f.px((134.0, -2.0)), f.px((216.0, 80.0)));
        let x = gland.center().x;
        common::cable(
            p,
            x,
            art.rect.min.y - CABLE * art.rect.height(),
            gland.min.y,
            8.0 * s,
            &sk,
        );
        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        p.rect_filled(gland, egui::CornerRadius::ZERO, sk.black);
        for i in 0..3 {
            let y = gland.min.y + gland.height() * (0.1 + 0.28 * i as f32);
            p.rect_filled(
                Rect::from_center_size(egui::pos2(x, y), vec2(18.0 * s, 9.0 * s)),
                egui::CornerRadius::same(2),
                mix(sk.deep, sk.low, 0.7),
            );
        }
        sheen(
            p,
            f.px((176.0, 380.0)),
            vec2(150.0, 220.0) * s,
            &sk,
            pal,
            34.0,
        );

        // Side panels and the rear cover, then the two big buttons over them.
        for (poly, dir) in [
            (g::LEFT_PANEL, vec2(1.0, 0.3)),
            (g::RIGHT_PANEL, vec2(-1.0, 0.3)),
        ] {
            let pts = f.pts(poly);
            kit::facet(p, &pts, dir, sk.high, sk.low, sk.hi, 0.6);
        }
        let palm = f.pts(g::PALM);
        kit::facet(p, &palm, DOWN, sk.mid, sk.deep, sk.hi, 0.5);
        for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(0.25, 1.0),
                mix(sk.high, sk.mid, 0.3),
                sk.mid,
                sk.hi,
                0.8,
            );
        }
        // The dark gap along the front of the side panels.
        let gap: Vec<Pos2> = [(14.0, 254.0), (62.0, 283.0), (110.0, 288.0), (162.0, 286.0)]
            .iter()
            .map(|q| f.px(*q))
            .collect();
        groove(p, &gap, 2.4 * s, &sk);
        let gap: Vec<Pos2> = [
            (204.0, 286.0),
            (262.0, 287.0),
            (310.0, 278.0),
            (358.0, 258.0),
        ]
        .iter()
        .map(|q| f.px(*q))
        .collect();
        groove(p, &gap, 2.4 * s, &sk);

        paint_window(p, &f, &sk, look);
        paint_strips(p, &f, &sk, look);
        for poly in [g::FORWARD, g::BACK] {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        paint_logo(p, art, &sk, look);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            // Outline each side's three strip segments together.
            for runs in [
                [g::STRIP_LT, g::STRIP_LM, g::STRIP_LB],
                [g::STRIP_RT, g::STRIP_RM, g::STRIP_RB],
            ] {
                let pts: Vec<Pos2> = runs
                    .iter()
                    .flat_map(|run| f.pts(run))
                    .flat_map(|q| [q + vec2(-8.0, 0.0), q + vec2(8.0, 0.0)])
                    .collect();
                let mut hull = convex_hull(&pts);
                hull.push(hull[0]);
                p.extend(Shape::dashed_line(&hull, ring, 5.0, 4.0));
            }
        });
    }
}

/// The wheel window with its glow, the wheel and the DPI key behind it.
fn paint_window(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let well = f.pts(g::WELL);
    kit::fill_flat(p, &well, sk.black);
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        glow(p, wheel.center(), wheel.width() * 0.6, with_alpha(c, 34));
    }
    rubber_wheel(p, wheel, sk, look.wheel, s);
    let pts = f.pts(g::DPI_KEY);
    key(
        p,
        &pts,
        sk,
        (1.5 * s).max(1.0),
        vec2(0.2, 1.0),
        sk.high,
        sk.mid,
        1.0,
    );
    // The rail that joins the window to the key.
    let rail = Rect::from_min_max(f.px((178.0, 190.0)), f.px((188.0, 224.0)));
    p.rect_filled(rail, egui::CornerRadius::ZERO, sk.black);
}

/// The six light strips, three segments down each side of the rear cover.
fn paint_strips(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    for (run, width) in [
        (g::STRIP_LT, 9.0),
        (g::STRIP_LM, 7.5),
        (g::STRIP_LB, 6.0),
        (g::STRIP_RT, 9.0),
        (g::STRIP_RM, 7.5),
        (g::STRIP_RB, 6.0),
    ] {
        let pts = f.pts(run);
        match look.body {
            Some(c) => light_line(p, &pts, width * s * 1.05, c),
            None => {
                p.add(Shape::line(
                    pts,
                    egui::epaint::PathStroke::new(width * s * 0.9, mix(sk.low, sk.hi, 0.18)),
                ));
            }
        }
    }
}

/// The rear mark: the cover's own light, a hex-free neutral diamond.
fn paint_logo(p: &Painter, art: &MouseArt, sk: &Plastic, look: &MouseLook) {
    if let Some(c) = look.logo {
        glow(p, art.logo.0, art.logo.1 * 0.9, with_alpha(c, 20));
    }
    common::diamond_mark(p, art.logo.0, art.logo.1, look.logo, sk);
}
