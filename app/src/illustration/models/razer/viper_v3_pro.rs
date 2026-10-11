//! The Viper V3 Pro (and V3 HyperSpeed): a symmetrical, featherweight shell with a long smooth
//! hump, two broad primary buttons split by a centre seam and cut off by a rounded rear seam,
//! a wheel under a tall status-LED cover, and thumb buttons on the left flank only. No
//! lighting; the logo is just engraved. Geometry lives in `viper_v3_pro_shapes`; the look is
//! matte black plastic.

use egui::{Painter, Pos2, Rect, vec2};

use super::shared;
use super::viper_v3_pro_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::rounded_poly;
use crate::theme::{Palette, mix};

pub struct ViperV3Pro;

impl ModelArt for ViperV3Pro {
    fn id(&self) -> &'static str {
        "razer-viper-v3-pro"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["viper v3"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Razer Viper V3 Pro (Wired)",
            "Razer Viper V3 Pro (Wireless)",
            "Razer Viper V3 HyperSpeed",
            "Viper V3 Pro (demo)",
        ]
    }

    /// A variant of the Viper family that needs its own shell, so it outranks the generic
    /// "viper" drawing (the Ultimate / V2 Pro).
    fn priority(&self) -> i32 {
        10
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn build(&self, rect: Rect) -> Anatomy {
        build(rect)
    }

    fn paint(
        &self,
        art: &MouseArt,
        p: &Painter,
        pal: &Palette,
        look: &MouseLook,
        tint: &[(Role, f32)],
    ) {
        paint(art, p, pal, look, tint);
    }
}

fn frame(rect: Rect) -> Frame {
    Frame::new(rect, g::ORIGIN, g::SIZE)
}

fn build(rect: Rect) -> Anatomy {
    let f = frame(rect);
    let s = f.s();
    let part =
        |role: Role, poly: Vec<Pos2>, anchor: P| Part::new(role, poly).anchored(f.px(anchor));
    let wheel = f.rect_px(g::WHEEL);
    Anatomy {
        parts: vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 4.0 * s, 3),
                g::ANCHOR_MIDDLE,
            ),
            part(Role::Forward, f.pts(g::FORWARD), g::ANCHOR_FORWARD),
            part(Role::Back, f.pts(g::BACK), g::ANCHOR_BACK),
        ],
        wheel,
        logo: (f.px(g::LOGO), g::LOGO_RADIUS * s),
    }
}

fn paint(art: &MouseArt, p: &Painter, pal: &Palette, look: &MouseLook, tint: &[(Role, f32)]) {
    let f = frame(art.rect);
    let s = f.s();
    let sk = Plastic::black(pal);
    let outline = f.norm(&shapes().outline);

    kit::shadow(p, &outline, art.rect, pal);
    kit::facet(p, &outline, vec2(0.1, 1.0), sk.mid, sk.low, sk.hi, 0.0);
    shared::sheen(p, f.px((250.0, 330.0)), vec2(66.0 * s, 90.0 * s), &sk, pal);

    // Primary buttons set into their seams; the right one catches a little more light.
    let gap = (1.2 * s).max(1.0);
    for (poly, from) in [
        (g::LEFT_BTN, mix(sk.mid, sk.high, 0.5)),
        (g::RIGHT_BTN, mix(sk.mid, sk.high, 0.7)),
    ] {
        shared::button(p, &f.pts(poly), gap, from, sk.mid, &sk, 0.8);
    }

    let ch = shared::Channel {
        well: g::CHANNEL,
        cover: g::COVER,
        led: Some(g::LED),
        window: g::WHEEL_SLOT,
        wheel: g::WHEEL,
        housed: true,
        tone: 0.6,
    };
    shared::channel(p, &f, &ch, &sk, look, (mix(sk.mid, sk.high, 0.5), sk.mid));

    // The slim thumb buttons on the left flank.
    for poly in [g::FORWARD, g::BACK] {
        shared::button(
            p,
            &f.pts(poly),
            (1.0 * s).max(1.0),
            sk.mid,
            sk.low,
            &sk,
            0.9,
        );
    }

    shared::volume(p, &outline, &sk, pal);
    shared::logo(p, art, &sk, look);
    shared::rim(p, &outline, &sk);

    kit::paint_tint(p, art, pal, tint);
    kit::zone_ring(p, art, pal, look, |p, ring| {
        shared::dashed_ring(p, &outline, ring);
    });
}
