//! The Viper Ultimate and Viper V2 Pro: a symmetrical, featherweight shell. The primary buttons
//! end in a V-shaped seam where the palm cover begins, the wheel runs in a long dark channel
//! with a status LED at its rear, and both flanks carry a pair of thumb buttons (so the same
//! shell serves left- and right-handed hands). The Ultimate lights its palm logo. Geometry
//! lives in `viper_ultimate_shapes`; the look is matte black plastic.

use egui::{Painter, Pos2, Rect, vec2};

use super::shared;
use super::viper_ultimate_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::rounded_poly;
use crate::theme::{Palette, mix};

/// The Viper family's symmetrical shell: Ultimate, V2 Pro, and the older Viper and Viper Mini
/// that share its look. The V3 Pro has a drawing of its own, which outranks this one.
pub struct ViperUltimate;

impl ModelArt for ViperUltimate {
    fn id(&self) -> &'static str {
        "razer-viper-ultimate"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["viper"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Razer Viper Ultimate (Wired)",
            "Razer Viper Ultimate (Wireless)",
            "Razer Viper V2 Pro (Wired)",
            "Razer Viper V2 Pro (Wireless)",
            "Viper Ultimate (demo)",
        ]
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
            part(
                Role::RightForward,
                f.pts(g::RIGHT_FORWARD),
                g::ANCHOR_RIGHT_FORWARD,
            ),
            part(Role::RightBack, f.pts(g::RIGHT_BACK), g::ANCHOR_RIGHT_BACK),
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
    shared::sheen(p, f.px((955.0, 500.0)), vec2(96.0 * s, 120.0 * s), &sk, pal);

    // Primary buttons set into their seams, then the V of the palm cover's front edge.
    let gap = (1.2 * s).max(1.0);
    let face = mix(sk.mid, sk.high, 0.8);
    for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
        shared::button(p, &f.pts(poly), gap, face, sk.mid, &sk, 0.8);
    }
    shared::seam_line(p, &f.pts(g::CHEVRON), (4.0 * s).max(2.6), &sk);
    shared::seam_line(p, &f.pts(g::SPINE), (2.2 * s).max(1.6), &sk);

    let ch = shared::Channel {
        well: g::CHANNEL,
        cover: g::COVER,
        led: Some(g::LED),
        window: g::WHEEL_SLOT,
        wheel: g::WHEEL,
        housed: false,
        tone: 0.7,
    };
    shared::channel(p, &f, &ch, &sk, look, (sk.low, sk.deep));

    // Thumb buttons on both flanks.
    for poly in [g::FORWARD, g::BACK, g::RIGHT_FORWARD, g::RIGHT_BACK] {
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
