//! The G502 X family (X, X Lightspeed, X Plus): a smoother shell than the HERO, with one straight
//! channel (helical wheel, a cycle key and G9) between two long primary buttons, the G8 / G7
//! pair and a DPI-shift paddle on the dark left flank, G5 / G4 above a ribbed thumb grip and,
//! on the Plus, an eight-segment light strip sweeping round the palm. Geometry lives in
//! `g502x_shapes`.

use egui::epaint::PathStroke;
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2, pos2, vec2};

use super::common::{self, WheelLook};
use super::g502x_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{fill_poly, glow, glow_line, linear, rot_rrect, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct G502X;

impl ModelArt for G502X {
    fn id(&self) -> &'static str {
        "g502-x"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["g502 x", "g502x"])
    }

    /// The X shell is not the HERO's: it beats the generic G502 drawing.
    fn priority(&self) -> i32 {
        10
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "G502 X PLUS",
            "G502 X Plus (demo)",
            "G502 X LIGHTSPEED",
            "G502 X Gaming Mouse",
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
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                g::ANCHOR_MIDDLE,
            ),
            part(Role::TiltLeft, around(g::TILT_L, 22.0, 28.0), g::TILT_L).labelled(Side::Left),
            part(Role::TiltRight, around(g::TILT_R, 22.0, 28.0), g::TILT_R).labelled(Side::Right),
            part(
                Role::DpiCycle,
                rounded_poly(f.rect_px(g::CYCLE_KEY), 5.0 * s, 3),
                g::ANCHOR_CYCLE,
            ),
            part(
                Role::Profile,
                rounded_poly(f.rect_px(g::G9_KEY), 5.0 * s, 3),
                g::ANCHOR_G9,
            ),
            // G8 / G7 sit left of the left button, so their labels go in the left margin.
            part(Role::DpiUp, f.pts(g::G8), g::ANCHOR_G8).labelled(Side::Left),
            part(Role::DpiDown, f.pts(g::G7), g::ANCHOR_G7).labelled(Side::Left),
            part(Role::Sniper, f.pts(g::SNIPER), g::ANCHOR_SNIPER),
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
        let sh = shapes();
        let outline = f.norm(&sh.outline);

        kit::shadow(p, &outline, art.rect, pal);
        common::shell(p, &outline, sk.mid, sk.deep, &sk);

        // The palm cover and its bloom, the ribbed thumb grip on the flank.
        let palm = f.norm(&sh.palm);
        kit::facet(p, &palm, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        common::sheen(
            p,
            f.px((290.0, 500.0)),
            vec2(130.0 * s, 150.0 * s),
            &sk,
            pal,
        );
        paint_grip(p, &f, &sk, s);

        // Primary buttons and the channel, each over its dark gap.
        let gap = 2.4 * s;
        common::panel(
            p,
            &f.pts(g::LEFT_BTN),
            vec2(0.15, 1.0),
            sk.high,
            sk.mid,
            &sk,
            gap,
            0.95,
        );
        common::panel(
            p,
            &f.pts(g::RIGHT_BTN),
            vec2(-0.1, 1.0),
            mix(sk.high, sk.hi, 0.12),
            sk.mid,
            &sk,
            gap,
            0.7,
        );
        common::panel(
            p,
            &f.pts(g::CHANNEL),
            vec2(0.0, 1.0),
            mix(sk.high, sk.hi, 0.22),
            mix(sk.high, sk.mid, 0.5),
            &sk,
            gap,
            0.7,
        );
        let chamfer = f.pts(g::CHAMFER);
        fill_poly(
            p,
            &chamfer,
            linear(
                &chamfer,
                vec2(0.0, 1.0),
                with_alpha(sk.hi, 70),
                with_alpha(sk.hi, 8),
            ),
        );

        // G8 / G7, the paddle, G5 / G4.
        let bright = mix(sk.high, sk.hi, 0.5);
        for poly in [g::G8, g::G7] {
            common::key(p, &f.pts(poly), vec2(0.4, 1.0), bright, sk.high, &sk);
        }
        common::engrave(
            p,
            f.px((121.0, 172.0)),
            "G8",
            8.5 * s,
            with_alpha(sk.black, 140),
        );
        common::engrave(
            p,
            f.px((108.0, 234.0)),
            "G7",
            8.5 * s,
            with_alpha(sk.black, 140),
        );
        common::key(p, &f.pts(g::SNIPER), vec2(0.3, 1.0), sk.mid, sk.low, &sk);
        paint_crosshair(p, f.px((84.0, 320.0)), 5.0 * s, &sk);
        for poly in [g::FORWARD, g::BACK] {
            common::key(p, &f.pts(poly), vec2(0.5, 1.0), sk.low, sk.deep, &sk);
        }

        paint_channel(p, &f, &sk, look, s);
        paint_strip(p, &f, &sk, look, s);
        common::logo(p, art, &sk, look);

        common::rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            for side in [-1.0, 1.0] {
                let line = offset_path(&f.norm(&sh.strip), side * 9.0 * s);
                p.extend(Shape::dashed_line(&line, ring, 5.0, 4.0));
            }
        });
    }
}

/// The ribbed thumb grip: a dark pad with diagonal ribs.
fn paint_grip(p: &Painter, f: &Frame, sk: &Plastic, s: f32) {
    let grip = f.pts(g::GRIP);
    kit::facet(p, &grip, vec2(1.0, 0.3), sk.deep, sk.low, sk.hi, 0.0);
    // Short, steep ribs from the lower left to the upper right, in rows across the pad.
    let (lo, hi) = (g::GRIP_RIBS.0, g::GRIP_RIBS.1);
    let ink = Stroke::new((1.1 * s).max(0.8), with_alpha(sk.hi, 50));
    let mut y = lo;
    while y < hi {
        if let Some((x0, x1)) = rib_span(&grip, f, y) {
            let mut x = x0 + 6.0 * s;
            while x < x1 - 5.0 * s {
                p.line_segment(
                    [
                        pos2(x - 3.0 * s, f.px((0.0, y + 7.0)).y),
                        pos2(x + 3.0 * s, f.px((0.0, y - 7.0)).y),
                    ],
                    ink,
                );
                x += 6.0 * s;
            }
        }
        y += 8.0;
    }
}

/// The x extent of the polygon `poly` (screen) at the reference row `y`.
fn rib_span(poly: &[Pos2], f: &Frame, y: f32) -> Option<(f32, f32)> {
    let sy = f.px((0.0, y)).y;
    let mut xs = Vec::new();
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.y <= sy && b.y > sy) || (b.y <= sy && a.y > sy) {
            xs.push(a.x + (b.x - a.x) * (sy - a.y) / (b.y - a.y));
        }
    }
    let lo = xs.iter().copied().fold(f32::MAX, f32::min);
    let hi = xs.iter().copied().fold(f32::MIN, f32::max);
    (lo <= hi).then_some((lo, hi))
}

/// A small crosshair on the DPI-shift paddle.
fn paint_crosshair(p: &Painter, c: Pos2, r: f32, sk: &Plastic) {
    let ink = Stroke::new(1.1, with_alpha(sk.hi, 160));
    p.circle_stroke(c, r, ink);
    p.hline(c.x - r * 1.5..=c.x + r * 1.5, c.y, ink);
    p.vline(c.x, c.y - r * 1.5..=c.y + r * 1.5, ink);
}

/// The channel: status light, helical wheel in its window, the cycle key and G9.
fn paint_channel(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook, s: f32) {
    // Status light: a short tilted bar, always green (it shows battery state).
    let (c, half, tilt) = g::STATUS;
    let c = f.px(c);
    let green = Color32::from_rgb(0x58, 0xe8, 0x6a);
    glow(p, c, 2.2 * s, with_alpha(green, 90));
    p.add(Shape::convex_polygon(
        rot_rrect(
            c,
            5.0 * s,
            half * 2.0 * s,
            tilt + std::f32::consts::FRAC_PI_2,
            2.4 * s,
        ),
        mix(green, Color32::WHITE, 0.45),
        Stroke::NONE,
    ));

    common::rubber_wheel(
        p,
        f.rect_px(g::WHEEL_SLOT),
        f.rect_px(g::WHEEL),
        s,
        sk,
        look.wheel,
        WheelLook {
            flanges: false,
            pitch: 8.0,
            split: false,
            slant: 7.0,
        },
    );
    // Tilt chevrons printed on the buttons.
    for (c, dir) in [(g::TILT_L, -1.0), (g::TILT_R, 1.0)] {
        let c = f.px(c);
        let (w, h) = (4.0 * s, 8.0 * s);
        p.add(Shape::line(
            vec![
                c + vec2(-dir * w, -h),
                c + vec2(dir * w, 0.0),
                c + vec2(-dir * w, h),
            ],
            PathStroke::new((1.6 * s).max(1.0), with_alpha(sk.hi, 70)),
        ));
    }

    // The keys behind the wheel: one dark ring, two pads.
    let ring = rounded_poly(f.rect_px(g::KEY_RING), 9.0 * s, 4);
    kit::fill_flat(p, &ring, sk.black);
    for key in [g::CYCLE_KEY, g::G9_KEY] {
        let poly = rounded_poly(f.rect_px(key), 4.0 * s, 3);
        kit::facet(
            p,
            &poly,
            DOWN,
            mix(sk.high, sk.hi, 0.45),
            sk.high,
            sk.hi,
            0.9,
        );
    }
    // The cycle key's mark: a dotted ring; G9's number.
    let c = f.rect_px(g::CYCLE_KEY).center();
    for i in 0..10 {
        let a = i as f32 * std::f32::consts::TAU / 10.0;
        p.circle_filled(
            c + vec2(a.cos(), a.sin()) * 6.0 * s,
            0.7 * s.max(1.0),
            with_alpha(sk.black, 150),
        );
    }
    common::engrave(
        p,
        f.rect_px(g::G9_KEY).center(),
        "G9",
        8.5 * s,
        with_alpha(sk.black, 150),
    );
}

/// The light strip: eight segments along the sweep and the spur, glowing in the body colour.
fn paint_strip(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook, s: f32) {
    let sh = shapes();
    let main = f.norm(&sh.strip);
    let spur = f.norm(&sh.spur);
    let width = 15.0 * s;
    let n = 8;
    let mut segs: Vec<Vec<Pos2>> = (0..n)
        .map(|i| {
            slice(
                &main,
                i as f32 / n as f32 + 0.004,
                (i + 1) as f32 / n as f32 - 0.004,
            )
        })
        .collect();
    segs.push(spur);
    for (i, seg) in segs.iter().enumerate() {
        match look.body {
            Some(col) => {
                // A slight lift towards white along the sweep keeps the segments readable.
                let c = mix(col, Color32::WHITE, 0.1 + 0.05 * (i % 2) as f32);
                glow_line(p, seg, width * 0.7, c);
                p.add(Shape::line(
                    seg.clone(),
                    PathStroke::new(width * 0.6, mix(c, Color32::WHITE, 0.35)),
                ));
            }
            None => {
                p.add(Shape::line(
                    seg.clone(),
                    PathStroke::new(width, with_alpha(sk.black, 220)),
                ));
                p.add(Shape::line(
                    seg.clone(),
                    PathStroke::new(width * 0.55, mix(sk.deep, sk.hi, 0.16)),
                ));
            }
        }
    }
}

/// The part of `path` between the fractions `t0` and `t1` of its length.
fn slice(path: &[Pos2], t0: f32, t1: f32) -> Vec<Pos2> {
    let total: f32 = path.windows(2).map(|w| (w[1] - w[0]).length()).sum();
    let (d0, d1) = (t0 * total, t1 * total);
    let mut out = Vec::new();
    let mut at = 0.0;
    for w in path.windows(2) {
        let len = (w[1] - w[0]).length();
        let (a, b) = (at, at + len);
        if b >= d0 && a <= d1 && len > 0.0 {
            let lerp = |d: f32| w[0] + (w[1] - w[0]) * ((d - a) / len).clamp(0.0, 1.0);
            if out.is_empty() {
                out.push(lerp(d0));
            }
            out.push(lerp(d1.min(b)));
        }
        at = b;
    }
    out
}

/// `path` pushed `d` px to the side of its direction of travel.
fn offset_path(path: &[Pos2], d: f32) -> Vec<Pos2> {
    (0..path.len())
        .map(|i| {
            let (a, b) = (path[i.saturating_sub(1)], path[(i + 1).min(path.len() - 1)]);
            let t: Vec2 = (b - a).normalized();
            path[i] + vec2(-t.y, t.x) * d
        })
        .collect()
}
