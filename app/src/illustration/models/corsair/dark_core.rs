//! The Dark Core RGB Pro family (Pro, Pro SE): a tall right-hand shell in soft-touch black with
//! a diamond-textured palm, two glossy front buttons either side of a long V-shaped channel that
//! holds the wheel, a profile light and the diamond DPI key, two thumb keys on the left above the
//! wing-shaped thumb rest (the grip on the right is the same piece mirrored) and a glossy rear
//! band. Light comes from the wheel rails, the channel trim, the thumb-side strip and the palm
//! logo. Geometry lives in `dark_core_shapes`.

use egui::epaint::PathStroke;
use egui::{Color32, Painter, Pos2, Rect, Shape, vec2};

use super::common::{
    chevron, groove, hex_mark, key, lattice, light_line, rim, rubber_wheel, sheen,
};
use super::dark_core_shapes::{self as g};
use crate::illustration::models::kit::{self, DOWN, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{centroid, glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct DarkCore;

impl ModelArt for DarkCore {
    fn id(&self) -> &'static str {
        "dark-core"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["dark core", "darkcore"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Corsair Dark Core RGB Pro",
            "Corsair Dark Core RGB Pro SE",
            "Corsair Dark Core RGB Pro SE (demo)",
            "Corsair DARK CORE RGB SE",
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
                    rounded_poly(wheel, 4.0 * s, 3),
                    (254.0, 136.0),
                ),
                part(Role::Profile, f.pts(g::PROFILE), (252.0, 216.0)),
                part(Role::DpiCycle, f.pts(g::DPI_BTN), (255.0, 280.0)),
                part(Role::Sniper, f.pts(g::SNIPER), (111.0, 136.0)),
                part(Role::Forward, f.pts(g::FORWARD), (110.0, 240.0)),
                part(Role::Back, f.pts(g::BACK), (80.0, 350.0)),
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
        // Soft-touch shell: a little lighter than the glossy parts on top of it.
        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((250.0, 470.0)),
            vec2(150.0, 170.0) * s,
            &sk,
            pal,
            30.0,
        );
        lattice(
            p,
            &kit::grown(&outline, -6.0 * s),
            16.0 * s,
            with_alpha(sk.hi, 34),
        );

        // The two wings: thumb rest on the left, grip on the right.
        for poly in [g::WING_L, g::WING_R] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(1.0, 0.3),
                mix(sk.high, sk.hi, 0.1),
                sk.mid,
                sk.hi,
                0.8,
            );
            lattice(p, &pts, 14.0 * s, with_alpha(sk.hi, 70));
        }
        // The glossy rear band.
        let band = f.pts(g::REAR_BAND);
        kit::facet(p, &band, DOWN, sk.high, sk.deep, sk.hi, 0.8);

        paint_buttons(p, &f, &sk);
        paint_channel(p, &f, &sk, look);
        paint_dpi_key(p, &f, &sk, look);
        paint_side(p, &f, &sk, look);
        hex_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            for run in [f.pts(g::TRIM_L), f.pts(g::TRIM_R), f.pts(g::LED_L)] {
                p.extend(Shape::dashed_line(&run, ring, 5.0, 4.0));
            }
        });
    }
}

/// The glossy front buttons, the V-shaped seam and the thumb keys.
fn paint_buttons(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
        let pts = f.pts(poly);
        kit::facet(p, &pts, vec2(0.5, 0.9), sk.high, sk.low, sk.hi, 0.9);
    }
    groove(p, &f.pts(g::SEAM_L), 2.4 * s, sk);
    groove(p, &f.pts(g::SEAM_R), 2.4 * s, sk);
    for poly in [g::SNIPER, g::FORWARD] {
        let pts = f.pts(poly);
        key(
            p,
            &pts,
            sk,
            (1.8 * s).max(1.0),
            vec2(0.4, 1.0),
            sk.high,
            sk.low,
            1.0,
        );
    }
    let back = f.pts(g::BACK);
    key(p, &back, sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
}

/// The channel: a glossy V holding the wheel with its rails, the profile light and trim lines.
fn paint_channel(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let channel = f.pts(g::CHANNEL);
    kit::fill_flat(p, &channel, sk.black);
    kit::facet(
        p,
        &kit::grown(&channel, -(2.0 * s).max(1.2)),
        vec2(1.0, 0.2),
        sk.deep,
        sk.black,
        sk.hi,
        0.0,
    );

    // Wheel with its rails.
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        glow(p, wheel.center(), wheel.width() * 0.55, with_alpha(c, 36));
    }
    rubber_wheel(p, wheel, sk, look.wheel, s);

    // The profile light: a slim bullet in the channel.
    let dash = f.pts(g::PROFILE);
    match look.body {
        Some(c) => {
            glow(p, centroid(&dash), 3.5 * s, with_alpha(c, 70));
            kit::fill_flat(p, &dash, mix(c, Color32::WHITE, 0.2));
        }
        None => {
            kit::facet(p, &dash, DOWN, sk.high, sk.mid, sk.hi, 0.6);
        }
    }

    // Trim lines framing the diamond key, lit by the body colour.
    for run in [g::TRIM_L, g::TRIM_R] {
        let pts = f.pts(run);
        match look.body {
            Some(c) => light_line(p, &pts, 1.6 * s, c),
            None => {
                p.add(Shape::line(
                    pts,
                    PathStroke::new(1.2, with_alpha(sk.hi, 90)),
                ));
            }
        }
    }
}

/// The diamond DPI key at the foot of the channel.
fn paint_dpi_key(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let pts = f.pts(g::DPI_BTN);
    key(
        p,
        &pts,
        sk,
        (1.6 * s).max(1.0),
        vec2(0.3, 1.0),
        sk.high,
        sk.deep,
        1.0,
    );
    let mut line = kit::grown(&pts, -(3.0 * s).max(1.5));
    line.push(line[0]);
    p.add(Shape::line(
        line,
        PathStroke::new(
            1.0,
            match look.body {
                Some(c) => with_alpha(mix(c, Color32::WHITE, 0.4), 150),
                None => with_alpha(sk.hi, 100),
            },
        ),
    ));
    chevron(
        p,
        f.px((255.0, 268.0)),
        7.0 * s,
        true,
        with_alpha(sk.hi, 120),
    );
}

/// The light strip down the thumb side of the shell.
fn paint_side(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let strip = f.pts(g::LED_L);
    match look.body {
        Some(c) => light_line(p, &strip, 2.4 * s, c),
        None => {
            p.add(Shape::line(
                strip,
                PathStroke::new(1.6 * s, mix(sk.low, sk.hi, 0.18)),
            ));
        }
    }
}
