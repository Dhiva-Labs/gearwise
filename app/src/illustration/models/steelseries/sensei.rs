//! The Sensei family (Ten, 310): an ambidextrous shell in matte black with a long centre rail
//! holding the wheel and a triangular DPI key, a flat button either side of it, and a pair of
//! thumb keys plus a rubber grip lobe on BOTH flanks, so left-handed and right-handed grips each
//! find two keys. The wheel rails and the mark on the palm are the lit zones. Geometry lives in
//! `sensei_shapes`.

use egui::{Painter, Rect, vec2};

use super::sensei_shapes::{self as g};
use crate::illustration::models::corsair::common::{
    self, diamond_mark, key, lattice, rim, rubber_wheel, sheen,
};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{centroid, glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.1;

pub struct Sensei;

impl ModelArt for Sensei {
    fn id(&self) -> &'static str {
        "sensei"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["sensei"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "SteelSeries Sensei Ten",
            "SteelSeries Sensei 310",
            "SteelSeries Sensei Ten (demo)",
            "SteelSeries Sensei Ten CS:GO Neon Rider Edition",
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
        let part = |role, poly: &[P]| {
            let pts = f.pts(poly);
            let c = centroid(&pts);
            Part::new(role, pts).anchored(c)
        };
        let wheel = f.rect_px(g::WHEEL);
        Anatomy {
            parts: vec![
                Part::new(Role::Left, f.pts(g::LEFT_BTN)).anchored(f.px(g::ANCHOR_LEFT)),
                Part::new(Role::Right, f.pts(g::RIGHT_BTN)).anchored(f.px(g::ANCHOR_RIGHT)),
                Part::new(Role::Middle, rounded_poly(wheel, 6.0 * s, 4)).anchored(wheel.center()),
                part(Role::DpiCycle, g::DPI_KEY),
                part(Role::Forward, g::FORWARD),
                part(Role::Back, g::BACK),
                part(Role::RightForward, g::RIGHT_FORWARD),
                part(Role::RightBack, g::RIGHT_BACK),
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
        common::cable(
            p,
            f.px((g::LOGO.0, 0.0)).x,
            art.rect.min.y - CABLE * art.rect.height(),
            f.px((0.0, 30.0)).y,
            9.0 * s,
            &sk,
        );
        kit::facet(p, &outline, vec2(0.3, 1.0), sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((g::LOGO.0, 330.0)),
            vec2(130.0, 250.0) * s,
            &sk,
            pal,
            34.0,
        );

        // Buttons and the rubber lobes at the rear of each flank.
        for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(0.3, 1.0),
                mix(sk.high, sk.mid, 0.35),
                sk.mid,
                sk.hi,
                0.7,
            );
        }
        for poly in [g::LOBE_L, g::LOBE_R] {
            let pts = f.pts(poly);
            kit::facet(p, &pts, vec2(1.0, 0.2), sk.high, sk.low, sk.hi, 0.5);
            lattice(p, &pts, 11.0 * s, with_alpha(sk.hi, 28));
        }

        // The centre rail, the wheel and the DPI key.
        let rail = f.pts(g::RAIL);
        key(
            p,
            &rail,
            &sk,
            (1.6 * s).max(1.0),
            vec2(0.2, 1.0),
            sk.mid,
            sk.deep,
            0.8,
        );
        kit::fill_flat(p, &f.pts(g::WELL), sk.black);
        let wheel = f.rect_px(g::WHEEL);
        if let Some(c) = look.wheel {
            glow(p, wheel.center(), wheel.width() * 0.7, with_alpha(c, 30));
        }
        rubber_wheel(p, wheel, &sk, look.wheel, s);
        p.circle_filled(
            f.px((g::LED.0, g::LED.1)),
            g::LED.2 * s,
            mix(sk.low, sk.hi, 0.18),
        );
        key(
            p,
            &f.pts(g::DPI_KEY),
            &sk,
            (1.4 * s).max(1.0),
            DOWN,
            sk.mid,
            sk.deep,
            0.9,
        );

        for poly in [g::FORWARD, g::BACK, g::RIGHT_FORWARD, g::RIGHT_BACK] {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        diamond_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |_, _| {});
    }
}
