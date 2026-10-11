//! The Basilisk V3 and V3 Pro: a tall right-handed shell with a flared thumb rest and a rubber
//! grip on each flank. The two primary buttons end in a diagonal seam above the palm cover;
//! between them the HyperScroll tilt wheel (with a Chroma strip either side) runs in a deep
//! channel, with the wheel-mode key and the DPI key stacked behind it. On the left wall sit the
//! DPI clutch and the two thumb buttons; the underglow strip rims the lower body and the logo
//! sits on the palm. Geometry lives in `basilisk_v3_shapes`.

use egui::epaint::PathStroke;
use egui::{CornerRadius, Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::basilisk_v3_shapes::{self as g, shapes};
use super::shared;
use crate::illustration::models::kit::{self, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{fill_poly, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct BasiliskV3;

impl ModelArt for BasiliskV3 {
    fn id(&self) -> &'static str {
        "razer-basilisk-v3"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["basilisk"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Razer Basilisk V3",
            "Razer Basilisk V3 Pro (Wired)",
            "Razer Basilisk V3 Pro (Wireless)",
            "Basilisk V3 Pro (demo)",
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
    let around = |c: P, w: f32, h: f32| {
        rounded_poly(
            Rect::from_center_size(f.px(c), vec2(w * s, h * s)),
            3.0 * s,
            3,
        )
    };
    let pill = |r: (P, P)| rounded_poly(f.rect_px(r), 4.0 * s, 3);
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
            part(
                Role::TiltLeft,
                around(g::TILT_L, 16.0, 22.0),
                g::ANCHOR_TILT_L,
            ),
            part(
                Role::TiltRight,
                around(g::TILT_R, 16.0, 22.0),
                g::ANCHOR_TILT_R,
            ),
            part(
                Role::WheelMode,
                pill((g::MODE_BTN.0, g::MODE_BTN.1)),
                g::ANCHOR_MODE,
            ),
            part(Role::DpiCycle, f.pts(g::DPI_BTN), g::ANCHOR_DPI),
            part(Role::Sniper, f.pts(g::SNIPER), g::ANCHOR_SNIPER),
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
    let sh = shapes();
    let outline = f.norm(&sh.outline);
    let under = f.norm(&sh.underglow);

    kit::shadow(p, &outline, art.rect, pal);
    // The underglow strip sits under the shell's rim, so only its light shows beyond it.
    shared::underglow(p, &under, 4.0 * s, &sk, look.body);
    kit::facet(p, &outline, vec2(0.1, 1.0), sk.mid, sk.low, sk.hi, 0.0);
    shared::sheen(p, f.px((235.0, 500.0)), vec2(80.0 * s, 96.0 * s), &sk, pal);

    // Rubber grips: the thumb rest and the right wall.
    shared::grip(p, &f.pts(g::LEFT_GRIP), &sk, s, vec2(1.0, 0.4));
    shared::grip(p, &f.pts(g::RIGHT_GRIP), &sk, s, vec2(-1.0, 0.4));

    // Primary buttons, the trench under them and the palm cover's lit edge.
    let gap = (1.3 * s).max(1.0);
    let face = mix(sk.mid, sk.high, 0.55);
    for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
        shared::button(p, &f.pts(poly), gap, face, sk.mid, &sk, 0.8);
    }
    for line in [g::PALM_LEFT, g::PALM_RIGHT] {
        shared::seam_line(p, &f.pts(line), (2.2 * s).max(1.6), &sk);
    }
    fill_poly(p, &f.pts(g::TRENCH), |_| sk.black);
    p.add(Shape::line(
        f.pts(g::PALM_EDGE),
        PathStroke::new(1.3, with_alpha(sk.hi, 64)),
    ));

    paint_channel(p, &f, &sk, look);
    shared::tilt_marks(p, &f, g::TILT_L, g::TILT_R, &sk);

    // Wheel-mode key, DPI key, then the thumb keys on the left wall.
    let mode = rounded_poly(f.rect_px(g::MODE_BTN), 3.5 * s, 3);
    shared::button(p, &mode, (1.2 * s).max(1.0), sk.high, sk.mid, &sk, 0.9);
    shared::button(
        p,
        &f.pts(g::DPI_BTN),
        (1.2 * s).max(1.0),
        sk.high,
        sk.mid,
        &sk,
        0.9,
    );
    for poly in [g::SNIPER, g::FORWARD, g::BACK] {
        shared::button(p, &f.pts(poly), (1.0 * s).max(1.0), face, sk.mid, &sk, 0.9);
    }

    shared::volume(p, &outline, &sk, pal);
    shared::logo(p, art, &sk, look);
    shared::rim(p, &outline, &sk);

    kit::paint_tint(p, art, pal, tint);
    kit::zone_ring(p, art, pal, look, |p, ring| {
        shared::dashed_run(p, &kit::grown(&under, 6.0), ring);
    });
}

/// The deep channel: a cover in front, the wheel in its window with a light strip either side,
/// and the pocket that holds the two keys behind it.
fn paint_channel(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    p.rect_filled(
        f.rect_px(g::CHANNEL),
        CornerRadius::same((2.0 * s) as u8),
        sk.black,
    );
    let cover = rounded_poly(
        Rect::from_min_max(f.px((222.0, 158.0)), f.px((257.0, 205.0))),
        2.0 * s,
        3,
    );
    kit::facet(p, &cover, kit::DOWN, sk.low, sk.deep, sk.hi, 0.5);

    let win = f.rect_px(g::WHEEL_SLOT);
    shared::window(p, win, 3.0 * s, sk);
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        crate::illustration::shapes::glow(
            p,
            wheel.center(),
            wheel.width() * 0.5,
            with_alpha(c, 50),
        );
    }
    shared::wheel(p, wheel, sk, look.wheel, s, 0.35);
    // The Chroma strips, between the wheel and the window.
    for x in [win.min.x + 3.6 * s, win.max.x - 3.6 * s] {
        let (a, b) = (
            Pos2::new(x, wheel.min.y + 1.0),
            Pos2::new(x, wheel.max.y - 1.0),
        );
        match look.wheel {
            Some(c) => shared::led_bar(p, a, b, 2.2 * s, c),
            None => {
                p.line_segment([a, b], Stroke::new(2.2 * s, mix(sk.low, sk.hi, 0.2)));
            }
        }
    }
}
