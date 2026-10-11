//! The G703 / G403: a right-handed mouse with a long thumb rest, two wedge buttons either side
//! of a centre slot that carries the ribbed wheel (a light strip down its middle) and a DPI key,
//! two side buttons on the left and a glowing palm mark. Geometry lives in `g703_shapes`.

use egui::epaint::PathStroke;
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::common::{self, WheelLook};
use super::g703_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{fill_poly, glow, glow_line, linear, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct G703;

impl ModelArt for G703 {
    fn id(&self) -> &'static str {
        "g703"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["g703", "g403"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "G703 LIGHTSPEED Wireless Gaming Mouse w/ HERO",
            "G703 LIGHTSPEED (demo)",
            "G403 HERO Gaming Mouse",
            "G403 Prodigy Gaming Mouse",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
        let s = f.s();
        let part =
            |role: Role, poly: Vec<Pos2>, anchor: P| Part::new(role, poly).anchored(f.px(anchor));
        let wheel = f.rect_px(g::WHEEL);
        let parts = vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(
                Role::DpiCycle,
                rounded_poly(f.rect_px(g::DPI_RING), 15.0 * s, 5),
                g::ANCHOR_DPI,
            ),
            part(Role::Forward, f.pts(g::FORWARD), g::ANCHOR_FORWARD),
            part(Role::Back, f.pts(g::BACK), g::ANCHOR_BACK),
        ];
        Anatomy {
            parts,
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
        let sk = Plastic::black(pal);
        let s = f.s();
        let outline = f.norm(&shapes().outline);

        kit::shadow(p, &outline, art.rect, pal);
        common::shell(p, &outline, sk.low, sk.deep, &sk);
        common::sheen(
            p,
            f.px((200.0, 560.0)),
            vec2(170.0 * s, 200.0 * s),
            &sk,
            pal,
        );

        // Primary buttons.
        for (poly, from) in [
            (g::LEFT_BTN, sk.high),
            (g::RIGHT_BTN, mix(sk.high, sk.hi, 0.12)),
        ] {
            common::panel(
                p,
                &f.pts(poly),
                vec2(0.1, 1.0),
                from,
                sk.mid,
                &sk,
                2.6 * s,
                0.9,
            );
        }
        for seam in [g::SEAM_L, g::SEAM_R] {
            let pts: Vec<Pos2> = f
                .pts(seam)
                .iter()
                .map(|q| *q + vec2(0.0, 2.4 * s))
                .collect();
            p.add(Shape::line(
                pts,
                PathStroke::new(1.4, with_alpha(sk.hi, 60)),
            ));
        }

        // The slot: cap, wheel with its light strip, spine with the DPI key.
        kit::fill_flat(p, &f.pts(g::SLOT), sk.black);
        let cap = f.rect_px(g::CAP);
        kit::shaded_rect(p, cap, 6, 3, |q| {
            let t = ((q.x - cap.min.x) / cap.width()).clamp(0.0, 1.0);
            mix(sk.deep, mix(sk.high, sk.hi, 0.18), t.powf(1.6))
        });
        let wheel_light = look.wheel.or(look.body);
        common::rubber_wheel(
            p,
            f.rect_px(g::WHEEL_SLOT),
            f.rect_px(g::WHEEL),
            s,
            &sk,
            None,
            WheelLook {
                flanges: false,
                pitch: 9.0,
                split: false,
                slant: 0.0,
            },
        );
        paint_strip(p, &f, &sk, wheel_light);
        let spine = f.pts(g::SPINE);
        fill_poly(
            p,
            &spine,
            linear(&spine, vec2(1.0, 0.1), sk.black, mix(sk.mid, sk.hi, 0.12)),
        );
        paint_dpi_key(p, &f, &sk);

        for poly in [g::FORWARD, g::BACK] {
            common::key(p, &f.pts(poly), vec2(0.6, 1.0), sk.high, sk.mid, &sk);
        }
        common::logo(p, art, &sk, look);

        common::rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let r = f.rect_px(g::WHEEL_SLOT).expand2(vec2(10.0, 8.0));
            let o = rounded_poly(r, 12.0, 5);
            p.extend(Shape::dashed_line(&common::closed(&o), ring, 5.0, 4.0));
        });
    }
}

/// The light strip down the middle of the wheel: glowing when its zone is lit, else a seam.
fn paint_strip(p: &Painter, f: &Frame, sk: &Plastic, color: Option<Color32>) {
    let r = f.rect_px(g::STRIP);
    let x = r.center().x;
    let line = [Pos2::new(x, r.min.y), Pos2::new(x, r.max.y)];
    match color {
        Some(c) => {
            glow_line(p, &line, r.width() * 0.9, c);
            p.line_segment(
                line,
                Stroke::new(r.width() * 0.75, mix(c, Color32::WHITE, 0.45)),
            );
            glow(p, r.center(), r.width() * 0.2, with_alpha(c, 60));
        }
        None => {
            p.line_segment(
                line,
                Stroke::new(r.width() * 0.5, with_alpha(sk.black, 200)),
            );
        }
    }
}

/// The DPI key: a dark pad in a ring, with a lit crescent on its left.
fn paint_dpi_key(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    let ring = rounded_poly(f.rect_px(g::DPI_RING), 15.0 * s, 5);
    kit::fill_flat(p, &ring, sk.black);
    let rim = f.rect_px(g::DPI_RING);
    let crescent = rounded_poly(
        Rect::from_min_max(
            rim.min + vec2(2.0 * s, 3.0 * s),
            Pos2::new(rim.min.x + 15.0 * s, rim.max.y - 3.0 * s),
        ),
        6.0 * s,
        4,
    );
    kit::facet(
        p,
        &crescent,
        DOWN,
        mix(sk.high, sk.hi, 0.6),
        sk.mid,
        sk.hi,
        0.6,
    );
    let pad = rounded_poly(f.rect_px(g::DPI_PAD), 7.0 * s, 4);
    kit::facet(p, &pad, DOWN, sk.deep, sk.black, sk.hi, 0.5);
}
