//! The G305 / G304: a compact egg with two wedge-shaped primary buttons, a long centre slot that
//! carries a wide-ribbed wheel and a DPI key with its LED, two side buttons on the left flank and
//! a palm mark. No lighting zones. Geometry lives in `g305_shapes`.

use egui::{Color32, Painter, Rect, Shape, vec2};

use super::common::{self, WheelLook};
use super::g305_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{fill_poly, linear, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct G305;

impl ModelArt for G305 {
    fn id(&self) -> &'static str {
        "g305"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["g305", "g304"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "G305 LIGHTSPEED Wireless Gaming Mouse",
            "G305 LIGHTSPEED (demo)",
            "G304 LIGHTSPEED Wireless Gaming Mouse",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
        let s = f.s();
        let part = |role: Role, poly: Vec<egui::Pos2>, anchor: P| {
            Part::new(role, poly).anchored(f.px(anchor))
        };
        let wheel = f.rect_px(g::WHEEL);
        let parts = vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 5.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(
                Role::DpiCycle,
                rounded_poly(f.rect_px(g::DPI_RING), 14.0 * s, 5),
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

        // The lower body: a darker, domed palm.
        kit::shadow(p, &outline, art.rect, pal);
        common::shell(p, &outline, sk.low, sk.deep, &sk);
        common::sheen(
            p,
            f.px((200.0, 540.0)),
            vec2(170.0 * s, 190.0 * s),
            &sk,
            pal,
        );

        // Primary buttons: two lit wedges over a dark gap.
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

        // The palm cover overlaps the buttons: its front edge catches the light.
        for seam in [g::SEAM_L, g::SEAM_R] {
            let pts: Vec<_> = f
                .pts(seam)
                .iter()
                .map(|q| *q + vec2(0.0, 2.4 * s))
                .collect();
            p.add(Shape::line(
                pts,
                egui::epaint::PathStroke::new(1.4, with_alpha(sk.hi, 70)),
            ));
        }

        // The slot: a dark channel, the lit cap in front of the wheel, the spine behind it.
        kit::fill_flat(p, &f.pts(g::SLOT), sk.black);
        let cap = f.rect_px(g::CAP);
        kit::shaded_rect(p, cap, 6, 3, |q| {
            let t = ((q.x - cap.min.x) / cap.width()).clamp(0.0, 1.0);
            mix(sk.deep, mix(sk.high, sk.hi, 0.35), t.powf(1.4))
        });
        common::rubber_wheel(
            p,
            f.rect_px(g::WHEEL_SLOT),
            f.rect_px(g::WHEEL),
            s,
            &sk,
            None,
            WheelLook {
                flanges: false,
                pitch: 11.0,
                split: true,
                slant: 0.0,
            },
        );
        let spine = f.pts(g::SPINE);
        fill_poly(
            p,
            &spine,
            linear(&spine, vec2(1.0, 0.15), sk.black, mix(sk.mid, sk.hi, 0.25)),
        );
        p.add(Shape::closed_line(
            spine.clone(),
            egui::epaint::PathStroke::new(1.0, with_alpha(sk.hi, 70)),
        ));

        // The DPI key with the LED under it.
        let ring = rounded_poly(f.rect_px(g::DPI_RING), 14.0 * s, 5);
        kit::fill_flat(p, &ring, sk.black);
        let pad = rounded_poly(f.rect_px(g::DPI_PAD), 11.0 * s, 5);
        kit::facet(p, &pad, DOWN, sk.mid, sk.low, sk.hi, 0.8);
        common::led_dot(p, f.px(g::LED), 3.4 * s, Some(cyan()), &sk);

        for poly in [g::FORWARD, g::BACK] {
            common::key(p, &f.pts(poly), vec2(0.6, 1.0), sk.high, sk.mid, &sk);
        }

        for dot in g::DOTS {
            common::led_dot(p, f.px(dot), 3.4 * s, None, &sk);
        }
        common::logo(p, art, &sk, look);

        common::rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let o = kit::grown(&outline, 6.0);
            p.extend(Shape::dashed_line(&common::closed(&o), ring, 6.0, 5.0));
        });
    }
}

/// The status LED beside the DPI key.
fn cyan() -> Color32 {
    Color32::from_rgb(0x2c, 0xcf, 0xf2)
}
