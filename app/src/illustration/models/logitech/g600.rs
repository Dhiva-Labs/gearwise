//! The G600 MMO: a broad, boxy right-hand shell with a long seam either side of the wheel (tilt
//! chevrons beside it), the G8 / G7 pair behind it, the ring-finger (G-shift) button down the
//! right flank, a palm logo and, bulging out of the left flank, the thumb panel of twelve
//! buttons (G9 to G20) in four columns of three. Geometry lives in `g600_shapes`.

use egui::epaint::PathStroke;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::common::{self, WheelLook};
use super::g600_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{fill_poly, linear, rot_rrect, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct G600;

impl ModelArt for G600 {
    fn id(&self) -> &'static str {
        "g600"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["g600"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &["G600 MMO Gaming Mouse", "G600 MMO (demo)"]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn overhang(&self) -> f32 {
        g::CABLE
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
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
        let wheel = f.rect_px(g::WHEEL);
        let mut parts = vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(Role::TiltLeft, around(g::TILT_L, 18.0, 26.0), g::TILT_L).labelled(Side::Left),
            part(Role::TiltRight, around(g::TILT_R, 18.0, 26.0), g::TILT_R).labelled(Side::Right),
            part(
                Role::DpiUp,
                rounded_poly(f.rect_px(g::G8), 5.0 * s, 3),
                g::ANCHOR_G8,
            ),
            part(
                Role::DpiDown,
                rounded_poly(f.rect_px(g::G7), 5.0 * s, 3),
                g::ANCHOR_G7,
            ),
            part(Role::Sniper, f.pts(g::SHIFT), g::ANCHOR_SHIFT).labelled(Side::Right),
        ];
        // The thumb grid, G9 first. Its labels go in the left margin.
        for i in 0..12u8 {
            let c = g::cell(i as usize);
            parts.push(
                Part::new(Role::Grid(i), cell_poly(&f, c, s))
                    .anchored(f.px(c))
                    .labelled(Side::Left),
            );
        }
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
        paint_cable(p, &f, &sk, s);
        common::shell(p, &outline, sk.mid, sk.low, &sk);
        common::sheen(
            p,
            f.px((250.0, 380.0)),
            vec2(150.0 * s, 230.0 * s),
            &sk,
            pal,
        );

        // The ring-finger button: a lighter strip down the right, split off by its long seam.
        let shift = f.pts(g::SHIFT);
        fill_poly(
            p,
            &shift,
            linear(&shift, vec2(-1.0, 0.3), mix(sk.high, sk.hi, 0.1), sk.low),
        );
        common::groove(p, &f.norm(&sh.seam_shift), 3.4 * s, &sk);
        for seam in g::SEAMS {
            common::groove(p, &f.pts(seam), 3.0 * s, &sk);
        }

        paint_panel(p, &f, &sk, s);

        // G8 and G7 behind the wheel.
        for (rect, label, c) in [(g::G8, "G8", 264.0), (g::G7, "G7", 317.0)] {
            let poly = rounded_poly(f.rect_px(rect), 5.0 * s, 3);
            common::key(p, &poly, DOWN, mix(sk.high, sk.hi, 0.2), sk.mid, &sk);
            common::engrave(p, f.px((244.0, c)), label, 12.0 * s, with_alpha(sk.hi, 170));
        }

        common::rubber_wheel(
            p,
            f.rect_px(g::WHEEL_SLOT),
            f.rect_px(g::WHEEL),
            s,
            &sk,
            look.wheel,
            WheelLook {
                flanges: false,
                pitch: 14.0,
                split: false,
                slant: 0.0,
            },
        );
        // Tilt chevrons printed beside the wheel.
        for (c, dir) in [(g::TILT_L, -1.0), (g::TILT_R, 1.0)] {
            let c = f.px(c);
            let (w, h) = (4.0 * s, 9.0 * s);
            p.add(Shape::line(
                vec![
                    c + vec2(-dir * w, -h),
                    c + vec2(dir * w, 0.0),
                    c + vec2(-dir * w, h),
                ],
                PathStroke::new((2.0 * s).max(1.2), with_alpha(sk.hi, 130)),
            ));
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

/// A cell of the thumb panel: a narrow, slightly slanted key at `c` (reference px).
fn cell_poly(f: &Frame, c: P, s: f32) -> Vec<Pos2> {
    rot_rrect(
        f.px(c),
        g::CELL_SIZE.0 * s,
        g::CELL_SIZE.1 * s,
        0.04,
        2.5 * s,
    )
}

/// The thumb panel: a dark pocket holding the twelve cells, each with a faint backlit number.
fn paint_panel(p: &Painter, f: &Frame, sk: &Plastic, s: f32) {
    let pocket = f.pts(g::PANEL);
    kit::facet(p, &pocket, vec2(1.0, 0.0), sk.black, sk.deep, sk.hi, 0.4);
    for i in 0..12 {
        let c = g::cell(i);
        let poly = cell_poly(f, c, s);
        let shade = [0.0, 0.5, 1.0][i % 3];
        kit::fill_flat(p, &poly, sk.black);
        let face = common::inset_poly(&poly, 1.0);
        kit::facet(
            p,
            &face,
            vec2(1.0, 0.0),
            mix(sk.mid, sk.high, shade),
            mix(sk.low, sk.mid, shade),
            sk.hi,
            0.7,
        );
        // The backlit number: a number when there is room for one, else a lit tick.
        let ink = Color32::from_rgb(0x9a, 0xc6, 0xff);
        let at = f.px(c);
        if 6.2 * s >= 5.6 {
            common::engrave(p, at, &format!("{}", i + 9), 6.2 * s, with_alpha(ink, 150));
        } else {
            let tick = rot_rrect(at, 3.0 * s, 13.0 * s, 0.04, 1.2 * s);
            p.add(Shape::convex_polygon(
                tick,
                with_alpha(ink, 170),
                Stroke::NONE,
            ));
        }
    }
}

/// The cable and its strain relief, leaving the nose.
fn paint_cable(p: &Painter, f: &Frame, sk: &Plastic, s: f32) {
    let top = f.rect.min.y - g::CABLE * f.rect.height();
    let x = f.px((g::CABLE_X, 0.0)).x;
    let boot_bottom = f.px((0.0, 52.0)).y;
    p.line_segment(
        [pos2(x, top), pos2(x, boot_bottom)],
        Stroke::new(12.0 * s, sk.deep),
    );
    p.line_segment(
        [pos2(x - 1.4 * s, top), pos2(x - 1.4 * s, boot_bottom)],
        Stroke::new(1.3, with_alpha(sk.hi, 60)),
    );
    let boot = Rect::from_min_max(
        pos2(x - 10.0 * s, f.px((0.0, 34.0)).y),
        pos2(x + 10.0 * s, boot_bottom),
    );
    p.rect_filled(boot, CornerRadius::same((3.0 * s) as u8), sk.black);
}
