//! The G Pro X Superlight 2 (and the classic G Pro Wireless, which shares the shell): a smooth,
//! symmetric egg with two primary panels split by a centre seam, a ribbed wheel in a slot, a
//! status LED behind the buttons, a palm logo and a pair of thin buttons on each flank (the
//! Superlight 2 only on the left). Geometry lives in `superlight2_shapes`.

use egui::{Color32, Painter, Pos2, Rect, vec2};

use super::common::{self, WheelLook};
use super::superlight2_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::rounded_poly;
use crate::theme::{Palette, mix};

pub struct Superlight2;

impl ModelArt for Superlight2 {
    fn id(&self) -> &'static str {
        "superlight-2"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(
            name,
            &[
                "superlight",
                "g pro x",
                "pro x superlight",
                "g pro wireless",
                "pro wireless",
            ],
        )
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "G Pro X Superlight 2",
            "G Pro X Superlight 2 (demo)",
            "PRO X 2 SUPERLIGHT",
            "G Pro X Superlight",
            "G Pro Wireless Gaming Mouse",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
        let s = f.s();
        let sh = shapes();
        let part =
            |role: Role, poly: Vec<Pos2>, anchor: P| Part::new(role, poly).anchored(f.px(anchor));
        let wheel = f.rect_px(g::WHEEL);
        let mirror = |a: P| (2.0 * (g::ORIGIN.0 + g::SIZE.0 / 2.0) - a.0, a.1);
        let parts = vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(Role::Forward, f.pts(g::FORWARD), g::ANCHOR_FORWARD),
            part(Role::Back, f.pts(g::BACK), g::ANCHOR_BACK),
            part(
                Role::RightForward,
                f.norm(&sh.right_forward),
                mirror(g::ANCHOR_FORWARD),
            ),
            part(
                Role::RightBack,
                f.norm(&sh.right_back),
                mirror(g::ANCHOR_BACK),
            ),
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
        let sh = shapes();
        let outline = f.norm(&sh.outline);

        kit::shadow(p, &outline, art.rect, pal);
        common::shell(p, &outline, sk.mid, sk.low, &sk);
        common::sheen(
            p,
            f.px((150.0, 470.0)),
            vec2(140.0 * s, 190.0 * s),
            &sk,
            pal,
        );

        // The flank walls catch the light below the shoulders.
        let toward = f.px((206.0, 400.0));
        common::band(p, &f.norm(&sh.left_wall), toward, 16.0 * s, &sk, 110);
        common::band(p, &f.norm(&sh.right_wall), toward, 16.0 * s, &sk, 70);

        // Primary panels, split by the centre seam, over the dark gap round them.
        for (poly, from) in [
            (g::LEFT_BTN, sk.high),
            (g::RIGHT_BTN, mix(sk.high, sk.hi, 0.12)),
        ] {
            common::panel(
                p,
                &f.pts(poly),
                vec2(0.12, 1.0),
                from,
                sk.mid,
                &sk,
                3.0 * s,
                0.9,
            );
        }

        // Side buttons: slivers on the left flank (the right pair of a G Pro Wireless is a part, but
        // the Superlight 2 has none there, so nothing is painted).
        for poly in [g::FORWARD, g::BACK] {
            common::key(p, &f.pts(poly), vec2(0.6, 1.0), sk.high, sk.mid, &sk);
        }

        common::rubber_wheel(
            p,
            f.rect_px(g::WHEEL_SLOT),
            f.rect_px(g::WHEEL),
            s,
            &sk,
            look.wheel,
            WheelLook {
                flanges: true,
                pitch: 4.4,
                split: false,
                slant: 0.0,
            },
        );

        // Status LED, and the palm mark.
        common::led_dot(p, f.px(g::LED), 2.6 * s, Some(green()), &sk);
        common::logo(p, art, &sk, look);

        common::rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let o = kit::grown(&outline, 6.0);
            p.extend(egui::Shape::dashed_line(
                &common::closed(&o),
                ring,
                6.0,
                5.0,
            ));
        });
    }
}

/// The status LED is always green-ish: it shows battery state, not a lighting zone.
fn green() -> Color32 {
    Color32::from_rgb(0x2b, 0xe0, 0x4a)
}
