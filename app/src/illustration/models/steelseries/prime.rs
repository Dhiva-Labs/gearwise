//! The Prime family (Prime, Prime+, Prime Mini, Prime Wireless): a clean, egg-shaped right-hand
//! shell in matte black. Two flush button panels, each outlined by a thin gap, split by a slot
//! with the wheel in a window, a braided cable and two thumb keys on the left flank. The DPI key
//! is on the underside, so it has no part here. The mark at the back is the logo zone. Geometry
//! lives in `prime_shapes`.

use egui::{Painter, Pos2, Rect, vec2};

use super::prime_shapes::{self as g};
use crate::illustration::models::corsair::common::{
    self, diamond_mark, groove, key, rim, rubber_wheel, sheen,
};
use crate::illustration::models::kit::{self, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.1;

pub struct Prime;

impl ModelArt for Prime {
    fn id(&self) -> &'static str {
        "prime"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["prime"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "SteelSeries Prime",
            "SteelSeries Prime+",
            "SteelSeries Prime Mini",
            "SteelSeries Prime Wireless",
            "SteelSeries Prime (demo)",
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
        let part = |role, poly: Vec<Pos2>, anchor| Part::new(role, poly).anchored(f.px(anchor));
        let wheel = f.rect_px(g::WHEEL);
        Anatomy {
            parts: vec![
                part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
                part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
                part(
                    Role::Middle,
                    rounded_poly(wheel, 6.0 * s, 4),
                    (179.0, 126.0),
                ),
                part(Role::Forward, f.pts(g::FORWARD), (13.0, 322.0)),
                part(Role::Back, f.pts(g::BACK), (13.0, 380.0)),
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
            f.px((177.0, 0.0)).x,
            art.rect.min.y - CABLE * art.rect.height(),
            f.px((0.0, 30.0)).y,
            9.0 * s,
            &sk,
        );
        kit::facet(p, &outline, vec2(0.45, 0.9), sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((230.0, 330.0)),
            vec2(120.0, 260.0) * s,
            &sk,
            pal,
            40.0,
        );

        // The two button panels: a hair lighter than the shell, outlined by thin gaps.
        for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(0.3, 1.0),
                mix(sk.high, sk.mid, 0.3),
                sk.mid,
                sk.hi,
                0.7,
            );
        }
        for run in [g::GAP_L, g::GAP_R, g::SEAM_L, g::SEAM_R] {
            groove(p, &f.pts(run), 2.0 * s, &sk);
        }

        // The slot, the wheel window and the wheel with its light.
        kit::fill_flat(p, &f.pts(g::SLOT), sk.black);
        kit::fill_flat(p, &f.pts(g::WELL), sk.black);
        let wheel = f.rect_px(g::WHEEL);
        if let Some(c) = look.logo {
            glow(p, wheel.center(), wheel.width() * 0.7, with_alpha(c, 30));
        }
        rubber_wheel(p, wheel, &sk, look.logo, s);

        for poly in [g::FORWARD, g::BACK] {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        diamond_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |_, _| {});
    }
}
