//! The G903: an ambidextrous, angular mouse with a metal-look spine (ribbed wheel, a cycle key,
//! two DPI keys and three DPI lights) between two wing-shaped primary buttons, a pair of thumb
//! buttons on each flank and a ridged, domed palm cover with the logo. Geometry lives in
//! `g903_shapes`; the right half is the mirror image of the measured left half.

use egui::epaint::PathStroke;
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::common::{self, WheelLook};
use super::g903_shapes::{self as g, mirrored, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{glow, glow_line, rot_rrect, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct G903;

impl ModelArt for G903 {
    fn id(&self) -> &'static str {
        "g903"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["g903"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "G903 LIGHTSPEED Wireless Gaming Mouse",
            "G903 LIGHTSPEED (demo)",
            "G903 HERO Wireless Gaming Mouse",
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
        let mirror_pt = |a: P| (2.0 * (g::ORIGIN.0 + g::SIZE.0 / 2.0) - a.0, a.1);
        let around = |c: P, w: f32, h: f32| {
            rounded_poly(
                Rect::from_center_size(f.px(c), vec2(w * s, h * s)),
                3.0 * s,
                3,
            )
        };
        let wheel = f.rect_px(g::WHEEL);
        let parts = vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(&mirrored(g::LEFT_BTN)), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(Role::TiltLeft, around(g::TILT_L, 22.0, 26.0), g::TILT_L).labelled(Side::Left),
            part(Role::TiltRight, around(g::TILT_R, 22.0, 26.0), g::TILT_R).labelled(Side::Right),
            part(
                Role::DpiCycle,
                rounded_poly(f.rect_px(g::KEY_RING), 6.0 * s, 4),
                g::ANCHOR_CYCLE,
            ),
            part(Role::DpiDown, f.pts(g::DPI_DOWN), g::ANCHOR_DOWN).labelled(Side::Left),
            part(Role::DpiUp, f.pts(g::DPI_UP), g::ANCHOR_UP),
            part(Role::Forward, f.pts(g::FORWARD), g::ANCHOR_FORWARD),
            part(Role::Back, f.pts(g::BACK), g::ANCHOR_BACK),
            part(
                Role::RightForward,
                f.pts(&mirrored(g::FORWARD)),
                mirror_pt(g::ANCHOR_FORWARD),
            ),
            part(
                Role::RightBack,
                f.pts(&mirrored(g::BACK)),
                mirror_pt(g::ANCHOR_BACK),
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
        common::shell(p, &outline, sk.low, sk.deep, &sk);

        // The domed palm cover, with a bloom on the lit right lobe and the grip ridges.
        let cover = f.norm(&sh.cover);
        kit::facet(p, &cover, DOWN, sk.mid, sk.deep, sk.hi, 0.0);
        common::sheen(
            p,
            f.px((290.0, 560.0)),
            vec2(120.0 * s, 170.0 * s),
            &sk,
            pal,
        );
        for rib in g::GRIP {
            for side in [false, true] {
                let pts: Vec<Pos2> = rib
                    .iter()
                    .map(|q| {
                        let q = if side { (1.0 - q.0, q.1) } else { *q };
                        f.pt(q)
                    })
                    .collect();
                p.add(Shape::line(
                    pts,
                    PathStroke::new((1.6 * s).max(1.0), with_alpha(sk.hi, 40)),
                ));
            }
        }

        // The dark walls round the spine, then the wings over them.
        kit::fill_flat(p, &f.pts(g::WALL), sk.black);
        for (poly, from) in [
            (f.pts(g::LEFT_BTN), sk.high),
            (f.pts(&mirrored(g::LEFT_BTN)), mix(sk.high, sk.hi, 0.12)),
        ] {
            common::panel(p, &poly, vec2(0.15, 1.0), from, sk.low, &sk, 2.2 * s, 0.9);
        }
        paint_bays(p, &f, &sk, s);
        paint_arrows(p, &f, &sk, s);

        paint_spine(p, &f, &sk, look, s);

        // Thumb buttons, both flanks.
        for poly in [g::FORWARD, g::BACK] {
            common::key(p, &f.pts(poly), vec2(0.6, 1.0), sk.high, sk.mid, &sk);
            common::key(
                p,
                &f.pts(&mirrored(poly)),
                vec2(-0.6, 1.0),
                sk.high,
                sk.mid,
                &sk,
            );
        }
        common::logo(p, art, &sk, look);

        common::rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let (a, b) = (f.px((176.0, 350.0)), f.px((236.0, 400.0)));
            let o = rounded_poly(Rect::from_two_pos(a, b).expand(5.0), 8.0, 4);
            p.extend(Shape::dashed_line(&common::closed(&o), ring, 5.0, 4.0));
        });
    }
}

/// The recess under each wing, with its three ridges.
fn paint_bays(p: &Painter, f: &Frame, sk: &Plastic, s: f32) {
    for side in [false, true] {
        let poly = if side {
            mirrored(g::BAY)
        } else {
            g::BAY.to_vec()
        };
        kit::facet(p, &f.pts(&poly), DOWN, sk.deep, sk.black, sk.hi, 0.0);
        for rib in g::RIDGES {
            let pts: Vec<Pos2> = rib
                .iter()
                .map(|q| f.pt(if side { (1.0 - q.0, q.1) } else { *q }))
                .collect();
            p.add(Shape::line(
                pts.clone(),
                PathStroke::new((3.2 * s).max(1.4), with_alpha(sk.hi, 70)),
            ));
            p.add(Shape::line(
                pts,
                PathStroke::new((1.2 * s).max(0.8), with_alpha(sk.hi, 120)),
            ));
        }
    }
}

/// The wheel-tilt chevrons printed on the wings.
fn paint_arrows(p: &Painter, f: &Frame, sk: &Plastic, s: f32) {
    for (c, dir) in [(g::TILT_L, -1.0), (g::TILT_R, 1.0)] {
        let c = f.px(c);
        let (w, h) = (5.0 * s, 9.0 * s);
        p.add(Shape::line(
            vec![
                c + vec2(-dir * w, -h),
                c + vec2(dir * w, 0.0),
                c + vec2(-dir * w, h),
            ],
            PathStroke::new((2.0 * s).max(1.2), with_alpha(sk.hi, 90)),
        ));
    }
}

/// The spine: lit metal column, wheel, cycle key, the two DPI keys and the three DPI lights.
fn paint_spine(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook, s: f32) {
    let spine = f.pts(g::SPINE);
    kit::facet(p, &spine, vec2(0.05, 1.0), sk.high, sk.mid, sk.hi, 1.0);
    // The bright bridge in front of the wheel.
    let bridge = f.pts(g::BRIDGE);
    kit::facet(
        p,
        &bridge,
        vec2(0.9, 0.2),
        mix(sk.high, sk.hi, 0.7),
        mix(sk.high, sk.hi, 0.25),
        sk.hi,
        0.9,
    );
    // The tail, a darker step over the cover.
    kit::facet(p, &f.pts(g::TAIL), DOWN, sk.mid, sk.low, sk.hi, 0.7);

    common::rubber_wheel(
        p,
        f.rect_px(g::WHEEL_SLOT),
        f.rect_px(g::WHEEL),
        s,
        sk,
        look.wheel,
        WheelLook {
            flanges: false,
            pitch: 24.0,
            split: false,
            slant: 0.0,
        },
    );

    // Cycle key: a ring, a dark pad and a bright bar at its foot.
    let ring = rounded_poly(f.rect_px(g::KEY_RING), 6.0 * s, 4);
    kit::fill_flat(p, &ring, sk.black);
    let pad = rounded_poly(f.rect_px(g::KEY_PAD), 4.0 * s, 4);
    kit::facet(p, &pad, DOWN, sk.mid, sk.low, sk.hi, 0.8);
    let bar = f.rect_px(g::KEY_BAR);
    p.rect_filled(bar, 1.5, with_alpha(Color32::WHITE, 215));
    let c = f.rect_px(g::KEY_PAD).center();
    p.circle_stroke(c, 6.0 * s, Stroke::new(1.2, with_alpha(sk.hi, 90)));

    // The two DPI keys, each with a bright wedge at its front.
    for (poly, sign) in [(g::DPI_DOWN, -1.0), (g::DPI_UP, 1.0)] {
        let pts = f.pts(poly);
        kit::fill_flat(p, &pts, sk.black);
        let face = common::inset_poly(&pts, 2.2 * s);
        kit::facet(p, &face, DOWN, sk.mid, sk.low, sk.hi, 0.6);
        let c = common::centroid_of(&face);
        let wedge = vec![
            c + vec2(-9.0 * s, -14.0 * s),
            c + vec2(9.0 * s, -14.0 * s),
            c + vec2(-sign * 9.0 * s, -4.0 * s),
        ];
        p.add(Shape::convex_polygon(
            wedge,
            with_alpha(Color32::WHITE, 215),
            Stroke::NONE,
        ));
        let mark = c + vec2(0.0, 10.0 * s);
        p.hline(
            mark.x - 3.5 * s..=mark.x + 3.5 * s,
            mark.y,
            Stroke::new(1.4, with_alpha(sk.hi, 130)),
        );
        if sign > 0.0 {
            p.vline(
                mark.x,
                mark.y - 3.5 * s..=mark.y + 3.5 * s,
                Stroke::new(1.4, with_alpha(sk.hi, 130)),
            );
        }
    }

    // The three DPI lights: slanted bars, glowing in the body zone's colour.
    let slot = Rect::from_two_pos(f.px((178.0, 354.0)), f.px((234.0, 398.0)));
    kit::fill_flat(p, &rounded_poly(slot, 3.0 * s, 3), sk.black);
    for (i, c) in g::BARS.iter().enumerate() {
        let c = f.px(*c);
        let tilt = (i as f32 - 1.0) * 0.14;
        let pts = rot_rrect(c, 7.0 * s, 36.0 * s, tilt, 3.0 * s);
        match look.body {
            Some(col) => {
                glow_line(
                    p,
                    &[c + vec2(0.0, -12.0 * s), c + vec2(0.0, 12.0 * s)],
                    6.0 * s,
                    col,
                );
                p.add(Shape::convex_polygon(
                    pts,
                    mix(col, Color32::WHITE, 0.3),
                    Stroke::NONE,
                ));
                glow(p, c, 2.0 * s, with_alpha(col, 70));
            }
            None => {
                p.add(Shape::convex_polygon(
                    pts,
                    mix(sk.deep, sk.hi, 0.14),
                    Stroke::NONE,
                ));
            }
        }
    }
}
