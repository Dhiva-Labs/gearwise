//! The Rival 3 (wired and wireless): a small, low, right-hand shell in matte black. Two buttons
//! split by a thin slot that holds the wheel, a DPI key behind it and a long curved seam where
//! the buttons end. Two thumb keys sit on the left flank. A prism strip of light runs round the
//! lower body along both skirts and the tail (the Front, Middle and Rear zones) and a mark sits at
//! the back of the palm. Geometry lives in `rival3_shapes`.

use egui::{Color32, Painter, Pos2, Rect, Shape, vec2};

use super::rival3_shapes::{self as g};
use crate::illustration::models::ModelArt;
use crate::illustration::models::corsair::common::{
    self, diamond_mark, groove, key, rim, rubber_wheel, sheen,
};
use crate::illustration::models::kit::{self, DOWN, Frame, Plastic};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{glow, glow_line, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.1;

pub struct Rival3;

impl ModelArt for Rival3 {
    fn id(&self) -> &'static str {
        "rival-3"
    }

    fn matches(&self, name: &str) -> bool {
        // "Rival 3" and "Rival 3 Wireless", but not the Rival 300 / 310 family.
        name.match_indices("rival 3")
            .any(|(i, _)| !name[i + 7..].starts_with(|c: char| c.is_ascii_digit()))
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "SteelSeries Rival 3",
            "SteelSeries Rival 3 Wireless",
            "SteelSeries Rival 3 (demo)",
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
                    (192.0, 121.0),
                ),
                part(Role::DpiCycle, f.pts(g::DPI_KEY), (193.0, 280.0)),
                part(Role::Forward, f.pts(g::FORWARD), (15.0, 362.0)),
                part(Role::Back, f.pts(g::BACK), (16.0, 424.0)),
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
            f.px((192.0, 0.0)).x,
            art.rect.min.y - CABLE * art.rect.height(),
            f.px((0.0, 60.0)).y,
            6.0 * s,
            &sk,
        );
        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((190.0, 430.0)),
            vec2(150.0, 200.0) * s,
            &sk,
            pal,
            34.0,
        );

        // The two buttons, with the long seam where they end and the skirt lines either side.
        for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(0.2, 1.0),
                mix(sk.high, sk.mid, 0.35),
                sk.mid,
                sk.hi,
                0.7,
            );
        }
        groove(p, &f.pts(g::SEAM), 2.2 * s, &sk);
        for run in [g::LINE_L, g::LINE_R] {
            groove(p, &f.pts(run), 1.8 * s, &sk);
        }

        paint_slot(p, &f, &sk, look);
        for poly in [g::FORWARD, g::BACK] {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        paint_strip(p, &f, &sk, look);
        diamond_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let run = kit::grown(&f.pts(g::STRIP), 8.0);
            p.extend(Shape::dashed_line(&run, ring, 5.0, 4.0));
        });
    }
}

/// The slot between the buttons with the wheel in its window and the DPI key below.
fn paint_slot(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let slot = Rect::from_min_max(f.px((187.0, 2.0)), f.px((197.0, 250.0)));
    p.rect_filled(slot, egui::CornerRadius::ZERO, sk.black);
    let window = f.pts(g::WINDOW);
    kit::fill_flat(p, &window, sk.black);
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.body {
        glow(p, wheel.center(), wheel.width() * 0.6, with_alpha(c, 28));
    }
    rubber_wheel(p, wheel, sk, look.body, s);
    let pts = f.pts(g::DPI_KEY);
    key(p, &pts, sk, (1.4 * s).max(1.0), DOWN, sk.mid, sk.deep, 0.9);
}

/// The prism strip: the light that spills from under the lower skirt and the tail, a halo along
/// the edge of the shell in the body colour with a thin bright line right on it.
fn paint_strip(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let run = f.pts(g::STRIP);
    match look.body {
        Some(c) => {
            let out = kit::grown(&run, 2.5 * s);
            glow_line(p, &out, 4.2 * s, c);
            p.add(Shape::line(
                run,
                egui::epaint::PathStroke::new((1.8 * s).max(1.4), mix(c, Color32::WHITE, 0.35)),
            ));
        }
        None => {
            p.add(Shape::line(
                run,
                egui::epaint::PathStroke::new((1.4 * s).max(1.0), mix(sk.low, sk.hi, 0.12)),
            ));
        }
    }
}
