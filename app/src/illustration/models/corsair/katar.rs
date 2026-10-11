//! The Katar Pro family (Pro, Pro XT): a small, low, symmetric shell with two big buttons split
//! by a glossy slot that holds the wheel and, at its foot, a hexagonal DPI key over a light
//! chevron. Thumb keys sit on the left flank and the logo is engraved on the palm. Geometry lives
//! in `katar_shapes`.

use egui::{Color32, Painter, Pos2, Rect, Shape, vec2};

use super::common::{self, groove, hex_mark, key, light_line, rim, rubber_wheel, sheen};
use super::katar_shapes::{self as g};
use crate::illustration::models::kit::{self, DOWN, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.1;

pub struct Katar;

impl ModelArt for Katar {
    fn id(&self) -> &'static str {
        "katar"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["katar"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Corsair Katar Pro",
            "Corsair Katar Pro XT",
            "Corsair Katar Pro XT (demo)",
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
                    (201.0, 127.0),
                ),
                part(Role::DpiCycle, f.pts(g::HEX), (199.0, 245.0)),
                part(Role::Forward, f.pts(g::FORWARD), (5.0, 266.0)),
                part(Role::Back, f.pts(g::BACK), (6.0, 366.0)),
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
        let slot_x = f.px((202.0, 0.0)).x;
        common::cable(
            p,
            slot_x,
            art.rect.min.y - CABLE * art.rect.height(),
            f.px((0.0, 80.0)).y,
            8.0 * s,
            &sk,
        );
        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((198.0, 420.0)),
            vec2(130.0, 170.0) * s,
            &sk,
            pal,
            34.0,
        );

        for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(0.3, 1.0),
                mix(sk.high, sk.mid, 0.4),
                sk.mid,
                sk.hi,
                0.8,
            );
        }
        groove(p, &f.pts(g::SEAM_L), 2.2 * s, &sk);
        groove(p, &f.pts(g::SEAM_R), 2.2 * s, &sk);

        paint_slot(p, &f, &sk, look);
        for poly in [g::FORWARD, g::BACK] {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        hex_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            // "Body" has no lights on this mouse; ring the hex key and its chevron anyway.
            let r = Rect::from_min_max(f.px((166.0, 210.0)), f.px((236.0, 290.0))).expand(6.0);
            let poly = rounded_poly(r, 10.0, 4);
            p.extend(Shape::dashed_line(
                &[poly.as_slice(), &poly[..1]].concat(),
                ring,
                5.0,
                4.0,
            ));
        });
    }
}

/// The glossy slot, the wheel in it and the hexagonal DPI key with its light chevron.
fn paint_slot(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let slot = f.pts(g::SLOT);
    kit::fill_flat(p, &slot, sk.black);
    kit::facet(
        p,
        &kit::grown(&slot, -(1.6 * s).max(1.0)),
        vec2(1.0, 0.2),
        sk.deep,
        sk.black,
        sk.hi,
        0.0,
    );
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        glow(p, wheel.center(), wheel.width() * 0.6, with_alpha(c, 34));
    }
    rubber_wheel(p, wheel, sk, look.wheel, s);

    // The key: a dark hexagon in a thin bright frame.
    let hex = f.pts(g::HEX);
    kit::fill_flat(p, &hex, sk.deep);
    kit::facet(
        p,
        &kit::grown(&hex, -(2.2 * s).max(1.2)),
        vec2(0.3, 1.0),
        sk.mid,
        sk.black,
        sk.hi,
        1.0,
    );
    let mut frame = hex.clone();
    frame.push(frame[0]);
    p.add(Shape::line(
        frame,
        egui::epaint::PathStroke::new(1.0, with_alpha(sk.hi, 90)),
    ));

    // The chevron beneath it, which shows the wheel zone's colour.
    let chevron = f.pts(g::CHEVRON);
    match look.wheel {
        Some(c) => {
            let line = [chevron[0], chevron[1], chevron[2]].map(|q| q + vec2(0.0, 2.5 * s));
            light_line(p, &line, 4.0 * s, c);
            kit::fill_flat(p, &chevron, mix(c, Color32::WHITE, 0.2));
        }
        None => kit::fill_flat(p, &chevron, mix(sk.low, sk.hi, 0.2)),
    }
}
