//! The MX Master family (3S, 3, 2S): a tall, sculpted right-hand shell with a wide thumb shelf,
//! two narrow primary panels either side of a rail that carries the metal MagSpeed wheel, a
//! mode-shift button behind it, and on the thumb side a housing with the thumb wheel and the
//! Back / Forward buttons. The gesture button is the thumb rest itself, marked by a small
//! engraved dash. No lighting, so the lighting zones are ignored. Geometry lives in
//! `mx_master_shapes`; the look is flat-shaded graphite with a lit rim.

use egui::epaint::PathStroke;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, vec2};

use super::mx_master_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, LIGHT, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{
    bezier, edge_lights, fill_poly, gradient_fan, linear, rot_rrect, rounded_poly, seam,
};
use crate::theme::{Palette, mix, with_alpha};

pub struct MxMaster;

impl ModelArt for MxMaster {
    fn id(&self) -> &'static str {
        "mx-master"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["mx master"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "MX Master 3S",
            "MX Master 3S (demo)",
            "MX Master 3S for Mac",
            "MX Master 3",
            "MX Master 2S",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn build(&self, rect: Rect) -> Anatomy {
        build(rect)
    }

    fn paint(
        &self,
        art: &MouseArt,
        p: &Painter,
        pal: &Palette,
        look: &MouseLook,
        tint: &[(Role, f32)],
    ) {
        paint(art, p, pal, look, tint);
    }
}

fn frame(rect: Rect) -> Frame {
    Frame::new(rect, g::ORIGIN, g::SIZE)
}

fn build(rect: Rect) -> Anatomy {
    let f = frame(rect);
    let s = f.s();
    let part =
        |role: Role, poly: Vec<Pos2>, anchor: P| Part::new(role, poly).anchored(f.px(anchor));
    let pill = |r: (P, P), radius: f32| rounded_poly(f.rect_px(r), radius * s, 4);
    let wheel = f.rect_px((g::WHEEL_L.0, g::WHEEL_R.1));
    let parts = vec![
        part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
        part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
        part(
            Role::Middle,
            rounded_poly(wheel, 6.0 * s, 4),
            g::ANCHOR_MIDDLE,
        ),
        part(Role::WheelMode, pill(g::MODE_BTN, 8.0), g::ANCHOR_MODE),
        part(Role::ThumbWheel, pill(g::THUMB_WHEEL, 9.0), g::ANCHOR_THUMB).labelled(Side::Left),
        part(Role::Forward, pill(g::FORWARD, 9.0), g::ANCHOR_FORWARD),
        part(Role::Back, pill(g::BACK, 9.0), g::ANCHOR_BACK),
        part(
            Role::Gesture,
            pill(g::GESTURE_AREA, 16.0),
            g::ANCHOR_GESTURE,
        )
        .labelled(Side::Left),
    ];
    Anatomy {
        parts,
        wheel,
        // The badge position; the mouse has no logo light, so nothing is drawn there.
        logo: (f.px((530.0, 395.0)), 14.0 * s),
    }
}

fn paint(art: &MouseArt, p: &Painter, pal: &Palette, look: &MouseLook, tint: &[(Role, f32)]) {
    let f = frame(art.rect);
    let sk = Plastic::graphite(pal);
    let sh = shapes();
    let outline = f.norm(&sh.outline);

    kit::shadow(p, &outline, art.rect, pal);
    kit::facet(p, &outline, vec2(0.12, 1.0), sk.mid, sk.low, sk.hi, 0.0);
    sheen(p, &f, &sk, pal);

    // Thumb shelf with its ridges, then the lit strip down the right flank and the heel's skirt.
    kit::facet(
        p,
        &f.norm(&sh.shelf),
        vec2(1.0, -0.25),
        sk.deep,
        sk.low,
        sk.hi,
        0.0,
    );
    paint_ridges(p, &f, &sk);
    let rim = f.norm(&sh.rim);
    fill_poly(
        p,
        &rim,
        linear(
            &rim,
            vec2(-1.0, 0.0),
            with_alpha(sk.hi, 70),
            with_alpha(sk.hi, 0),
        ),
    );
    kit::facet(
        p,
        &f.norm(&sh.skirt),
        vec2(0.1, 1.0),
        mix(sk.high, sk.hi, 0.18),
        sk.high,
        sk.hi,
        0.8,
    );

    paint_housing(p, &f, &sk);
    paint_panels(p, &f, &sk);
    paint_wheel(p, &f);
    paint_mode_button(p, &f, &sk);
    paint_gesture_mark(p, &f, &sk);

    // A faint rim all round keeps the shell readable on a dark card, then the crisp light
    // along the upper-left of the silhouette.
    p.add(Shape::closed_line(
        outline.clone(),
        PathStroke::new(1.0, with_alpha(sk.hi, 36)),
    ));
    edge_lights(p, &outline, LIGHT, sk.hi, 0.55);

    kit::paint_tint(p, art, pal, tint);
    kit::zone_ring(p, art, pal, look, |p, ring| {
        p.extend(Shape::dashed_line(
            &[outline.as_slice(), &outline[..1]].concat(),
            ring,
            6.0,
            5.0,
        ));
    });
}

/// A soft bloom on the palm.
fn sheen(p: &Painter, f: &Frame, sk: &Plastic, pal: &Palette) {
    let c = f.px((690.0, 560.0));
    let radii = vec2(170.0 * f.s(), 190.0 * f.s());
    let ring: Vec<Pos2> = (0..32)
        .map(|i| {
            let a = i as f32 * std::f32::consts::PI / 16.0;
            c + vec2(a.cos() * radii.x, a.sin() * radii.y)
        })
        .collect();
    let peak = if pal.dark { 36.0 } else { 44.0 };
    p.add(gradient_fan(&ring, |q| {
        let d = ((q - c) / radii).length().min(1.0);
        with_alpha(sk.hi, (peak * (1.0 - d * d)) as u8)
    }));
}

/// The fan of curved ridges on the thumb panel.
fn paint_ridges(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    for (a, b, bow) in g::RIDGES {
        let (a, b) = (f.px(*a), f.px(*b));
        let d = b - a;
        let normal = vec2(d.y, -d.x).normalized();
        let toward_up_left = if normal.x + normal.y > 0.0 {
            -normal
        } else {
            normal
        };
        let mid = a + d * 0.5 + toward_up_left * (bow * s);
        // A quadratic curve, written as the cubic with both controls on the same side.
        let pts = bezier(
            a,
            a + (mid - a) * (2.0 / 3.0),
            b + (mid - b) * (2.0 / 3.0),
            b,
            18,
        );
        seam(p, &pts, with_alpha(sk.black, 150), with_alpha(sk.hi, 34));
    }
}

/// The thumb housing: a raised lip round a dark pocket holding the thumb wheel and the
/// Forward / Back buttons.
fn paint_housing(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    let lip = rounded_poly(f.rect_px(g::HOUSING).expand(2.4 * s), 24.0 * s, 6);
    kit::facet(p, &lip, vec2(0.3, 1.0), sk.high, sk.mid, sk.hi, 0.9);
    let pocket = rounded_poly(f.rect_px(g::HOUSING), 22.0 * s, 6);
    kit::facet(p, &pocket, DOWN, sk.deep, sk.black, sk.hi, 0.0);

    // Thumb wheel: two knurled metal bands with a groove between them.
    for band in [g::THUMB_UPPER, g::THUMB_LOWER] {
        let r = f.rect_px(band);
        metal_band(p, r, 5.0 * s, false, 4.4 * s);
    }

    // Forward (front) and Back (rear), side by side below the wheel.
    for key in [g::FORWARD, g::BACK] {
        let poly = rounded_poly(f.rect_px(key), 9.0 * s, 4);
        kit::facet(p, &poly, vec2(0.0, 1.0), sk.low, sk.deep, sk.hi, 0.9);
    }
}

/// The two primary panels and the rail between them.
fn paint_panels(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    // The seam between the panels, up on the nose.
    let (top, bottom) = (f.px((666.0, 113.0)), f.px((666.0, 186.0)));
    p.line_segment([top, bottom], Stroke::new((3.4 * s).max(1.4), sk.black));

    for (poly, from) in [
        (g::LEFT_BTN, sk.high),
        (g::RIGHT_BTN, mix(sk.high, sk.hi, 0.22)),
    ] {
        let pts = f.pts(poly);
        kit::facet(p, &pts, vec2(0.2, 1.0), sk.black, sk.black, sk.hi, 0.0);
        let face = kit::grown(&pts, -(2.0 * s).max(1.4));
        kit::facet(p, &face, vec2(0.15, 1.0), from, sk.mid, sk.hi, 0.9);
    }

    let rail = f.pts(g::RAIL);
    kit::facet(p, &rail, DOWN, sk.black, sk.black, sk.hi, 0.0);
    let face = kit::grown(&rail, -(2.4 * s).max(1.6));
    kit::facet(p, &face, vec2(0.0, 1.0), sk.high, sk.mid, sk.hi, 0.7);
}

/// A knurled metal band (a cylinder seen from above): bright across most of its length, dark
/// where it curves away, with ridges across it. `across_x` is true when the cylinder's axis
/// runs along x (the main wheel), else along y (the thumb wheel).
fn metal_band(p: &Painter, r: Rect, radius: f32, across_x: bool, pitch: f32) {
    let (c, half) = (r.center(), r.size() / 2.0);
    let bright = Color32::from_rgb(0xfb, 0xfc, 0xfe);
    let dark = Color32::from_rgb(0x5a, 0x5f, 0x68);
    kit::shaded_rect(p, r, 7, 9, |q| {
        // Position along the cylinder's axis and around it (the sides curve away and darken).
        let (along, around) = if across_x {
            ((q.x - c.x) / half.x, (q.y - c.y) / half.y)
        } else {
            ((q.y - c.y) / half.y, (q.x - c.x) / half.x)
        };
        let shade = (1.0 - 0.7 * around.abs().powi(4)) * (1.0 - 0.25 * along.abs().powi(2));
        mix(dark, bright, shade.clamp(0.0, 1.0))
    });
    // Ridges: a dark groove with a light flank, spaced so they stay distinct when small.
    let pitch = pitch.max(3.0);
    let groove = with_alpha(Color32::BLACK, 120);
    let flank = with_alpha(Color32::WHITE, 90);
    let (len, n) = if across_x {
        (r.height(), (r.height() / pitch) as i32)
    } else {
        (r.width(), (r.width() / pitch) as i32)
    };
    let step = len / n.max(1) as f32;
    for i in 1..n {
        let t = step * i as f32;
        if across_x {
            let y = r.min.y + t;
            p.hline(r.min.x + 0.5..=r.max.x - 0.5, y, Stroke::new(1.0, groove));
            p.hline(
                r.min.x + 0.5..=r.max.x - 0.5,
                y + 1.0,
                Stroke::new(1.0, flank),
            );
        } else {
            let x = r.min.x + t;
            p.vline(x, r.min.y + 0.5..=r.max.y - 0.5, Stroke::new(1.0, groove));
            p.vline(
                x + 1.0,
                r.min.y + 0.5..=r.max.y - 0.5,
                Stroke::new(1.0, flank),
            );
        }
    }
    let corner = CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8);
    p.rect_stroke(
        r,
        corner,
        Stroke::new(1.0, with_alpha(Color32::BLACK, 150)),
        StrokeKind::Inside,
    );
}

/// The MagSpeed wheel: a dark window holding two knurled metal bands and the bright hub between.
fn paint_wheel(p: &Painter, f: &Frame) {
    let s = f.s();
    let window = f.rect_px(g::WINDOW);
    let radius = CornerRadius::same((5.0 * s).round().max(2.0) as u8);
    p.rect_filled(
        window.expand(2.0 * s),
        radius,
        Color32::from_black_alpha(160),
    );
    p.rect_filled(window, radius, Color32::from_rgb(0x03, 0x04, 0x05));
    p.rect_stroke(
        window,
        radius,
        Stroke::new(1.0, Color32::from_white_alpha(26)),
        StrokeKind::Inside,
    );

    // The hub shows between the bands: bright where the wheel faces the light, grey behind.
    let hub = f.rect_px(g::HUB);
    let poly = rounded_poly(hub, 2.0 * s, 3);
    let (a, b) = (
        Color32::from_rgb(0xf2, 0xf4, 0xf8),
        Color32::from_rgb(0x71, 0x76, 0x7e),
    );
    fill_poly(p, &poly, |q| {
        mix(
            a,
            b,
            ((q.y - hub.min.y) / hub.height() * 1.4 - 0.2).clamp(0.0, 1.0),
        )
    });
    for band in [g::WHEEL_L, g::WHEEL_R] {
        metal_band(p, f.rect_px(band), 3.0 * s, true, 5.0 * s);
    }
}

fn paint_mode_button(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    let r = f.rect_px(g::MODE_BTN);
    let ring = rounded_poly(r.expand(2.0 * s), 10.0 * s, 4);
    kit::facet(p, &ring, DOWN, sk.black, sk.black, sk.hi, 0.0);
    let face = rounded_poly(r, 8.0 * s, 4);
    kit::facet(p, &face, vec2(0.1, 1.0), sk.high, sk.mid, sk.hi, 0.9);
}

/// The gesture button is the thumb rest; its only mark is a small engraved dash.
fn paint_gesture_mark(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    let c = f.px(g::GESTURE_MARK);
    let dash = rot_rrect(c, 4.2 * s, 30.0 * s, -0.12, 2.0 * s);
    let lit: Vec<Pos2> = dash.iter().map(|q| *q + vec2(0.9, 0.9)).collect();
    p.add(Shape::convex_polygon(
        lit,
        with_alpha(sk.hi, 70),
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(dash, sk.deep, Stroke::NONE));
    // A very faint ring marks the pressable area.
    let area = rounded_poly(f.rect_px(g::GESTURE_AREA), 16.0 * s, 5);
    p.add(Shape::closed_line(
        area,
        PathStroke::new(1.0, with_alpha(sk.hi, 16)),
    ));
}
