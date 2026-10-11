//! The Harpoon family (RGB Pro, RGB Wireless): a compact right-hand shell in soft-touch black.
//! Two long buttons are split by a glossy slot that holds the wheel, with the DPI key and its
//! indicator light in the slot behind it; two thumb keys sit on the left flank and the logo
//! glows at the back of the palm. Geometry lives in `harpoon_shapes`.

use egui::{Painter, Pos2, Rect, Shape, vec2};

use super::common::{hex_mark, key, light_line, rim, rubber_wheel, sheen};
use super::harpoon_shapes::{self as g};
use crate::illustration::models::kit::{self, DOWN, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct Harpoon;

impl ModelArt for Harpoon {
    fn id(&self) -> &'static str {
        "harpoon"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["harpoon"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Corsair Harpoon RGB Pro",
            "Corsair Harpoon RGB Wireless",
            "Corsair Harpoon RGB Wireless (demo)",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
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
                    (210.0, 136.0),
                ),
                part(Role::DpiCycle, f.pts(g::DPI_KEY), (214.0, 292.0)),
                part(Role::Forward, f.pts(g::FORWARD), (33.0, 176.0)),
                part(Role::Back, f.pts(g::BACK), (31.0, 262.0)),
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
        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((210.0, 430.0)),
            vec2(140.0, 190.0) * s,
            &sk,
            pal,
            34.0,
        );

        // The slot with the wheel and the DPI key.
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
        rubber_wheel(p, wheel, &sk, None, s);
        paint_dpi_key(p, &f, &sk, look);

        for poly in [g::FORWARD, g::BACK] {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        paint_logo(p, art, &sk, look);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let r = Rect::from_min_max(f.px((186.0, 224.0)), f.px((242.0, 340.0))).expand(5.0);
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

/// The DPI key: a glossy tapered pad with its indicator light.
fn paint_dpi_key(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let pts = f.pts(g::DPI_KEY);
    key(
        p,
        &pts,
        sk,
        (1.4 * s).max(1.0),
        vec2(0.3, 1.0),
        sk.high,
        sk.deep,
        1.0,
    );
    let led = f.pts(g::LED);
    match look.body {
        Some(c) => light_line(p, &led, 3.2 * s, c),
        None => {
            p.add(Shape::line(
                led,
                egui::epaint::PathStroke::new(2.4 * s, mix(sk.low, sk.hi, 0.2)),
            ));
        }
    }
}

fn paint_logo(p: &Painter, art: &MouseArt, sk: &Plastic, look: &MouseLook) {
    if let Some(c) = look.logo {
        glow(p, art.logo.0, art.logo.1 * 0.9, with_alpha(c, 22));
    }
    hex_mark(p, art.logo.0, art.logo.1, look.logo, sk);
}
