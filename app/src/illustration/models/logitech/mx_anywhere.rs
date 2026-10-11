//! The MX Anywhere 3S / 3: a compact, symmetric shell. A domed lid sits inside a lighter skirt, a
//! long rounded channel runs down the middle with the MagSpeed metal wheel, the oval mode-shift
//! button and the status light, and two thumb buttons (Forward, Back) sit on the left flank.
//! No lighting. Geometry lives in `mx_anywhere_shapes`; the look is flat-shaded graphite.

use egui::{Color32, Painter, Pos2, Rect, Shape, vec2};

use super::common;
use super::mx_anywhere_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::rounded_poly;
use crate::theme::{Palette, mix};

pub struct MxAnywhere;

impl ModelArt for MxAnywhere {
    fn id(&self) -> &'static str {
        "mx-anywhere"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["mx anywhere"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "MX Anywhere 3S",
            "MX Anywhere 3S (demo)",
            "MX Anywhere 3",
            "MX Anywhere 3 for Mac",
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
                rounded_poly(wheel, 5.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(
                Role::WheelMode,
                rounded_poly(f.rect_px(g::MODE_BTN), 17.0 * s, 6),
                g::ANCHOR_MODE,
            ),
            part(Role::Forward, f.pts(g::FORWARD), g::ANCHOR_FORWARD),
            part(Role::Back, f.pts(g::BACK), g::ANCHOR_BACK),
        ];
        Anatomy {
            parts,
            wheel,
            // The badge position; the mouse has no logo light, so nothing is drawn there.
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
        let sk = Plastic::graphite(pal);
        let s = f.s();
        let sh = shapes();
        let outline = f.norm(&sh.outline);

        // The skirt, lit from above, and the lid inside it.
        kit::shadow(p, &outline, art.rect, pal);
        kit::facet(p, &outline, vec2(0.12, 1.0), sk.high, sk.mid, sk.hi, 0.0);
        let lid = f.norm(&sh.lid);
        common::groove(p, &common::closed(&lid), 2.4 * s, &sk);
        kit::facet(p, &lid, DOWN, mix(sk.mid, sk.high, 0.4), sk.low, sk.hi, 0.0);
        common::sheen(
            p,
            f.px((255.0, 360.0)),
            vec2(170.0 * s, 230.0 * s),
            &sk,
            pal,
        );

        // The channel: a pill with a dark seam round a lighter, brushed face.
        let (a, b) = g::CHANNEL;
        let rect = Rect::from_two_pos(f.px(a), f.px(b));
        let pill = rounded_poly(rect, g::CHANNEL_R * s, 10);
        common::panel(
            p,
            &pill,
            vec2(0.8, 0.2),
            mix(sk.high, sk.hi, 0.1),
            mix(sk.high, sk.mid, 0.2),
            &sk,
            2.2 * s,
            0.8,
        );

        // Wheel window, MagSpeed wheel.
        let slot = rounded_poly(f.rect_px(g::WHEEL_SLOT), 5.0 * s, 4);
        kit::fill_flat(p, &slot, Color32::from_rgb(0x03, 0x04, 0x05));
        common::metal_wheel(p, f.rect_px(g::WHEEL), s);

        // The mode button, and the status light under it.
        let ring = rounded_poly(f.rect_px(g::MODE_BTN).expand(2.2 * s), 19.0 * s, 6);
        kit::fill_flat(p, &ring, sk.black);
        let face = rounded_poly(f.rect_px(g::MODE_BTN), 17.0 * s, 6);
        kit::facet(p, &face, DOWN, mix(sk.high, sk.hi, 0.2), sk.mid, sk.hi, 0.9);
        let c = f.px(g::LED.0);
        let green = Color32::from_rgb(0x5c, 0xe8, 0x6c);
        common::led_dot(p, c, 3.8 * s, Some(green), &sk);

        for poly in [g::FORWARD, g::BACK] {
            common::key(p, &f.pts(poly), vec2(0.6, 1.0), sk.high, sk.mid, &sk);
        }

        common::rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let o = kit::grown(&outline, 6.0);
            p.extend(Shape::dashed_line(&common::closed(&o), ring, 6.0, 5.0));
        });
    }
}
