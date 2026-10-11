//! The Naga V2 Pro and Naga Pro: a tall right-handed MMO shell with a pronounced palm wing on
//! the right and, on the left flank, the swappable thumb plate. The drawing shows the 12-button
//! plate (four rows of three keys, numbered row by row); the 6- and 2-button plates use its
//! first cells. The two primary buttons run the length of the shell either side of the tilt
//! wheel, with the two DPI keys behind it. Geometry lives in `naga_v2_pro_shapes`.

use egui::epaint::PathStroke;
use egui::{Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::naga_v2_pro_shapes::{self as g, shapes};
use super::shared;
use crate::illustration::models::kit::{self, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::rounded_poly;
use crate::theme::{Palette, mix, with_alpha};

/// The thumb plate has this many cells.
const CELLS: usize = 12;

pub struct NagaV2Pro;

impl ModelArt for NagaV2Pro {
    fn id(&self) -> &'static str {
        "razer-naga-v2-pro"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["naga"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Razer Naga V2 Pro (Wired)",
            "Razer Naga V2 Pro (Wireless)",
            "Razer Naga Pro (Wired)",
            "Naga V2 Pro (demo)",
            "Naga Pro (demo)",
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
    let mut parts = vec![
        part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
        part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
        part(
            Role::Middle,
            rounded_poly(wheel, 4.0 * s, 3),
            g::ANCHOR_MIDDLE,
        ),
        part(
            Role::TiltLeft,
            around(g::TILT_L, 14.0, 20.0),
            g::ANCHOR_TILT_L,
        ),
        part(
            Role::TiltRight,
            around(g::TILT_R, 14.0, 20.0),
            g::ANCHOR_TILT_R,
        ),
        part(Role::DpiUp, pill(g::DPI_UP), g::ANCHOR_DPI_UP),
        part(Role::DpiDown, pill(g::DPI_DOWN), g::ANCHOR_DPI_DOWN),
    ];
    for i in 0..CELLS {
        parts.push(Part::new(Role::Grid(i as u8), f.pts(&g::grid_cell(i))));
    }
    Anatomy {
        parts,
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
    shared::sheen(p, f.px((255.0, 345.0)), vec2(78.0 * s, 96.0 * s), &sk, pal);

    // Primary buttons set into their seams.
    let gap = (1.3 * s).max(1.0);
    let face = mix(sk.mid, sk.high, 0.5);
    for (poly, from) in [
        (g::LEFT_BTN, face),
        (g::RIGHT_BTN, mix(sk.mid, sk.high, 0.65)),
    ] {
        shared::button(p, &f.pts(poly), gap, from, sk.mid, &sk, 0.8);
    }

    // The channel: the wheel block between two slits, the two DPI keys behind the wheel,
    // chevrons on the buttons.
    let ch = shared::Channel {
        well: g::CHANNEL,
        cover: g::CHANNEL,
        led: None,
        window: g::WHEEL_SLOT,
        wheel: g::WHEEL,
        housed: true,
        tone: 0.4,
    };
    shared::channel(p, &f, &ch, &sk, look, (face, sk.mid));
    for key in [g::DPI_UP, g::DPI_DOWN] {
        let poly = rounded_poly(f.rect_px(key), 3.5 * s, 3);
        shared::button(p, &poly, (1.4 * s).max(1.0), sk.high, sk.mid, &sk, 0.9);
    }
    shared::tilt_marks(p, &f, g::TILT_L, g::TILT_R, &sk);

    paint_plate(p, &f, &sk, look);

    shared::volume(p, &outline, &sk, pal);
    shared::logo(p, art, &sk, look);
    shared::rim(p, &outline, &sk);

    kit::paint_tint(p, art, pal, tint);
    kit::zone_ring(p, art, pal, look, |p, ring| {
        shared::dashed_ring(p, &kit::grown(&f.pts(g::PLATE), 5.0), ring);
    });
}

/// The thumb plate: a recessed bezel on the flank and 12 keys. Each row shows one broad key top
/// (the inner column) and the numbered faces of the other two, which light in the body zone's
/// colour.
fn paint_plate(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let bezel = f.pts(g::PLATE);
    kit::facet(p, &bezel, kit::DOWN, sk.deep, sk.black, sk.hi, 0.0);
    p.add(Shape::closed_line(
        bezel,
        PathStroke::new(1.0, with_alpha(sk.hi, 40)),
    ));
    for i in 0..CELLS {
        let cell = f.pts(&g::grid_cell(i));
        let top = i % 3 == 2;
        let (from, to) = if top {
            (sk.high, sk.mid)
        } else {
            (sk.mid, sk.low)
        };
        shared::button(p, &cell, (0.8 * s).max(0.8), from, to, sk, 0.9);
        if top {
            continue;
        }
        // The key's number: a short stroke on its face.
        let c = crate::illustration::shapes::centroid(&cell);
        let (a, b) = (c - vec2(0.0, 4.5 * s), c + vec2(0.0, 4.5 * s));
        match look.body {
            Some(col) => shared::led_bar(p, a, b, 1.5 * s, col),
            None => {
                p.line_segment([a, b], Stroke::new(1.5 * s, mix(sk.low, sk.hi, 0.24)));
            }
        }
    }
}
