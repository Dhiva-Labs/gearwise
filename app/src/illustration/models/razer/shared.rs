//! Pieces every Razer drawing shares: the smoothed silhouette, polygon hatching for the rubber
//! grips, the knurled wheel, buttons set into dark seams, the neutral logo mark and the Chroma
//! glows. Nothing here belongs to one particular mouse.

use std::f32::consts::PI;

use egui::epaint::PathStroke;
use egui::{
    Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::illustration::models::kit::{self, DOWN, Frame, LIGHT, P, Plastic};
use crate::illustration::mouse::MouseArt;
use crate::illustration::roles::MouseLook;
use crate::illustration::shapes::{
    chaikin, edge_lights, fill_poly, glow, glow_line, gradient_fan, linear, rounded_poly,
};
use crate::theme::{Palette, mix, with_alpha};

/// Normalised points as positions.
pub fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

/// The silhouette, clockwise: the sharp `front` run as given, then the `body` run with its
/// corners rounded. `body` starts where `front` ends and ends where `front` starts.
pub fn outline(front: &[P], body: &[P]) -> Vec<Pos2> {
    let mut out = pos(front);
    out.extend(chaikin(&pos(body), 3).into_iter().skip(1));
    out.pop();
    out
}

/// The points of a silhouette at or below `y` (normalised), in order: the run that carries an
/// underglow strip.
pub fn below(outline: &[Pos2], y: f32) -> Vec<Pos2> {
    outline.iter().filter(|p| p.y >= y).copied().collect()
}

/// Parallel lines `gap` apart at `angle` radians (0 = horizontal), clipped to the simple polygon
/// `poly`: the building block of the rubber texture.
pub fn hatch(poly: &[Pos2], angle: f32, gap: f32) -> Vec<[Pos2; 2]> {
    let (s, c) = angle.sin_cos();
    let rot = |q: Pos2| pos2(q.x * c + q.y * s, -q.x * s + q.y * c);
    let back = |q: Pos2| pos2(q.x * c - q.y * s, q.x * s + q.y * c);
    let r: Vec<Pos2> = poly.iter().map(|q| rot(*q)).collect();
    let (lo, hi) = r
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q.y), b.max(q.y)));
    let mut out = Vec::new();
    let mut y = (lo / gap).ceil() * gap + gap * 0.5;
    while y < hi {
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..r.len() {
            let (a, b) = (r[i], r[(i + 1) % r.len()]);
            if (a.y <= y) != (b.y <= y) {
                xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
            }
        }
        xs.sort_by(f32::total_cmp);
        for pair in xs.chunks_exact(2) {
            out.push([back(pos2(pair[0], y)), back(pos2(pair[1], y))]);
        }
        y += gap;
    }
    out
}

/// A rubber grip panel: dark base with a fine diamond lattice that catches the light.
pub fn grip(p: &Painter, poly: &[Pos2], sk: &Plastic, s: f32, dir: Vec2) {
    fill_poly(p, poly, linear(poly, dir, sk.deep, sk.black));
    let gap = (3.3 * s).max(2.7);
    let mut lines = Vec::new();
    for (angle, color) in [
        (0.62, with_alpha(sk.hi, 34)),
        (-0.62, with_alpha(sk.hi, 22)),
    ] {
        for [a, b] in hatch(poly, angle, gap) {
            lines.push(Shape::line_segment([a, b], Stroke::new(0.9, color)));
        }
    }
    p.add(Shape::Vec(lines));
    p.add(Shape::closed_line(
        poly.to_vec(),
        PathStroke::new(1.0, with_alpha(Color32::BLACK, 120)),
    ));
}

/// A soft bloom of light on the palm.
pub fn sheen(p: &Painter, c: Pos2, radii: Vec2, sk: &Plastic, pal: &Palette) {
    let ring: Vec<Pos2> = (0..32)
        .map(|i| {
            let a = i as f32 * PI / 16.0;
            c + vec2(a.cos() * radii.x, a.sin() * radii.y)
        })
        .collect();
    let peak = if pal.dark { 30.0 } else { 40.0 };
    p.add(gradient_fan(&ring, |q| {
        let d = ((q - c) / radii).length().min(1.0);
        with_alpha(sk.hi, (peak * (1.0 - d * d)) as u8)
    }));
}

/// A soft light from the upper left across the whole shell, so the flat facets read as one
/// curved body.
pub fn volume(p: &Painter, outline: &[Pos2], sk: &Plastic, pal: &Palette) {
    let peak = if pal.dark { 30 } else { 38 };
    fill_poly(
        p,
        outline,
        linear(
            outline,
            vec2(1.0, 0.35),
            with_alpha(sk.hi, peak),
            with_alpha(sk.hi, 0),
        ),
    );
}

/// A dark seam: a wide black line with a thin light one just below it, along an open polyline.
pub fn seam_line(p: &Painter, pts: &[Pos2], width: f32, sk: &Plastic) {
    p.add(Shape::line(pts.to_vec(), PathStroke::new(width, sk.black)));
    let lit: Vec<Pos2> = pts
        .iter()
        .map(|q| *q + vec2(0.0, width * 0.7 + 0.6))
        .collect();
    p.add(Shape::line(
        lit,
        PathStroke::new(1.0, with_alpha(sk.hi, 38)),
    ));
}

/// A button set into a dark seam: a black plate, then the lit face a little inside it.
pub fn button(
    p: &Painter,
    pts: &[Pos2],
    gap: f32,
    from: Color32,
    to: Color32,
    sk: &Plastic,
    edge: f32,
) {
    fill_poly(p, pts, |_| sk.black);
    let face = kit::grown(pts, -gap);
    kit::facet(p, &face, DOWN, from, to, sk.hi, edge);
}

/// The dark window a wheel turns in, with a soft shadow round it.
pub fn window(p: &Painter, r: Rect, radius: f32, sk: &Plastic) {
    let corner = CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8);
    p.rect_filled(r.expand(1.6), corner, Color32::from_black_alpha(120));
    p.rect_filled(r, corner, sk.black);
    p.rect_stroke(
        r,
        corner,
        Stroke::new(1.0, with_alpha(sk.hi, 40)),
        StrokeKind::Inside,
    );
}

/// A knurled rubber wheel seen from above: shaded so its sides curve away, with ridges across
/// it. `tone` (0 dark rubber, 1 pale) sets how much light the tyre catches; `lit` washes it with
/// the wheel's Chroma colour.
pub fn wheel(p: &Painter, r: Rect, sk: &Plastic, lit: Option<Color32>, s: f32, tone: f32) {
    let (c, half) = (r.center(), r.width() / 2.0);
    let bright = mix(sk.mid, mix(sk.high, sk.hi, 0.55), tone);
    kit::shaded_rect(p, r, 7, 10, |q| {
        let t = ((q.x - c.x) / half).abs().min(1.0);
        let shade = 1.0 - t * t;
        let base = mix(sk.deep, bright, shade);
        lit.map_or(base, |col| mix(base, col, 0.22 * shade))
    });
    let pitch = (3.7 * s).max(3.0);
    let rows = (r.height() / pitch) as i32;
    let step = r.height() / rows.max(1) as f32;
    for i in 0..rows {
        let y = r.min.y + step * (i as f32 + 0.5);
        p.hline(
            r.min.x + 1.0..=r.max.x - 1.0,
            y,
            Stroke::new(1.3, with_alpha(sk.black, 210)),
        );
        p.hline(
            r.min.x + 1.5..=r.max.x - 1.5,
            y + 1.4,
            Stroke::new(1.0, with_alpha(sk.hi, 40 + (tone * 40.0) as u8)),
        );
    }
    p.rect_stroke(
        r,
        CornerRadius::same((2.0 * s).round().clamp(1.0, 8.0) as u8),
        Stroke::new(1.0, with_alpha(Color32::BLACK, 150)),
        StrokeKind::Inside,
    );
}

/// The wheel channel as the shapes files measure it, in reference pixels: the black well between
/// the primary buttons, the cover in front of the wheel (it carries the status LED), the dark
/// window the wheel turns in and the wheel itself.
pub struct Channel {
    pub well: (P, P),
    pub cover: (P, P),
    pub led: Option<P>,
    pub window: (P, P),
    pub wheel: (P, P),
    /// The wheel block is plastic like the buttons (only thin slits of the well show either
    /// side), rather than a black pocket with a cover at its front.
    pub housed: bool,
    /// How much light the wheel's tyre catches (see [`wheel`]).
    pub tone: f32,
}

/// Paint the channel, cover, LED and wheel. `cover_ramp` is the cover's (or housing's) plastic,
/// light to dark.
pub fn channel(
    p: &Painter,
    f: &Frame,
    ch: &Channel,
    sk: &Plastic,
    look: &MouseLook,
    cover_ramp: (Color32, Color32),
) {
    let s = f.s();
    let well = f.rect_px(ch.well);
    p.rect_filled(well, CornerRadius::same((2.0 * s) as u8), sk.black);
    if ch.housed {
        let block =
            Rect::from_min_max(well.min + vec2(3.4 * s, 0.0), well.max - vec2(3.4 * s, 0.0));
        let poly = rounded_poly(block, 2.0 * s, 3);
        kit::facet(p, &poly, DOWN, cover_ramp.0, cover_ramp.1, sk.hi, 0.7);
    } else {
        let cover = rounded_poly(f.rect_px(ch.cover), 3.0 * s, 3);
        kit::facet(p, &cover, DOWN, cover_ramp.0, cover_ramp.1, sk.hi, 0.7);
    }
    if let Some(led) = ch.led {
        let led = f.px(led);
        let green = Color32::from_rgb(0x3a, 0xf0, 0x58);
        glow(p, led, 2.4 * s, with_alpha(green, 150));
        p.rect_filled(
            Rect::from_center_size(led, vec2(12.0 * s, 4.4 * s)),
            CornerRadius::same((2.2 * s) as u8),
            green,
        );
    }
    let wheel = f.rect_px(ch.wheel);
    if let Some(c) = look.wheel {
        glow(p, wheel.center(), wheel.width() * 0.5, with_alpha(c, 55));
    }
    window(p, f.rect_px(ch.window), 3.0 * s, sk);
    self::wheel(p, wheel, sk, look.wheel, s, ch.tone);
}

/// A bar of light: the Chroma strips beside a wheel and the status LED.
pub fn led_bar(p: &Painter, a: Pos2, b: Pos2, width: f32, color: Color32) {
    glow_line(p, &[a, b], width, color);
    p.add(Shape::line(
        vec![a, b],
        PathStroke::new(width, mix(color, Color32::WHITE, 0.35)),
    ));
}

/// The neutral logo mark: a thin hexagon round a dot. Lit in the logo zone's colour, or just
/// engraved when the zone is off (or the mouse has no logo light).
pub fn logo(p: &Painter, art: &MouseArt, sk: &Plastic, look: &MouseLook) {
    let (c, r) = art.logo;
    let ring: Vec<Pos2> = (0..=6)
        .map(|i| {
            let a = (30.0 + 60.0 * i as f32).to_radians();
            c + vec2(a.cos(), a.sin()) * (r * 0.8)
        })
        .collect();
    let width = r * 0.2;
    match look.logo {
        Some(col) => {
            glow_line(p, &ring, width, col);
            p.add(Shape::line(
                ring,
                PathStroke::new(width, mix(col, Color32::WHITE, 0.3)),
            ));
            glow(p, c, r * 0.16, mix(col, Color32::WHITE, 0.25));
        }
        None => {
            let ink = mix(sk.low, sk.hi, 0.1);
            p.add(Shape::line(ring, PathStroke::new(width, ink)));
            p.circle_filled(c, r * 0.16, ink);
        }
    }
}

/// The little chevrons printed on the buttons either side of the wheel (`left` and `right` are
/// the marks' centres in reference pixels).
pub fn tilt_marks(p: &Painter, f: &Frame, left: P, right: P, sk: &Plastic) {
    let s = f.s();
    for (c, dir) in [(left, -1.0), (right, 1.0)] {
        let c = f.px(c);
        let pts = vec![
            c + vec2(-dir * 2.6 * s, -5.5 * s),
            c + vec2(dir * 2.6 * s, 0.0),
            c + vec2(-dir * 2.6 * s, 5.5 * s),
        ];
        p.add(Shape::line(
            pts,
            PathStroke::new(1.6, with_alpha(sk.hi, 120)),
        ));
    }
}

/// An underglow strip along `pts`: Chroma colour when lit, else a pale unlit ridge.
pub fn underglow(p: &Painter, pts: &[Pos2], width: f32, sk: &Plastic, body: Option<Color32>) {
    match body {
        Some(col) => {
            glow_line(p, pts, width * 1.6, col);
            p.add(Shape::line(
                pts.to_vec(),
                PathStroke::new(width, mix(col, Color32::WHITE, 0.25)),
            ));
        }
        None => {
            p.add(Shape::line(
                pts.to_vec(),
                PathStroke::new(width, mix(sk.low, sk.hi, 0.12)),
            ));
        }
    }
}

/// The faint rim all round that keeps a black shell readable on a dark card, then the crisp
/// light along the lit upper left of the silhouette.
pub fn rim(p: &Painter, outline: &[Pos2], sk: &Plastic) {
    p.add(Shape::closed_line(
        outline.to_vec(),
        PathStroke::new(1.0, with_alpha(sk.hi, 34)),
    ));
    edge_lights(p, outline, LIGHT, sk.hi, 0.55);
}

/// A dashed outline round a closed shape, for the "body" lighting zone's selection ring.
pub fn dashed_ring(p: &Painter, outline: &[Pos2], ring: Stroke) {
    p.extend(Shape::dashed_line(
        &[outline, &outline[..1]].concat(),
        ring,
        6.0,
        5.0,
    ));
}

/// A dashed line along an open run, for the selection ring of a strip zone (underglow).
pub fn dashed_run(p: &Painter, pts: &[Pos2], ring: Stroke) {
    p.extend(Shape::dashed_line(pts, ring, 6.0, 5.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hatching_stays_inside_the_polygon() {
        let square = [
            pos2(0.0, 0.0),
            pos2(40.0, 0.0),
            pos2(40.0, 40.0),
            pos2(0.0, 40.0),
        ];
        let lines = hatch(&square, 0.6, 4.0);
        assert!(lines.len() > 10);
        let bounds = Rect::from_min_max(pos2(-0.01, -0.01), pos2(40.01, 40.01));
        for [a, b] in lines {
            assert!(bounds.contains(a) && bounds.contains(b), "{a:?} {b:?}");
        }
    }

    #[test]
    fn hatching_skips_the_notch_of_a_concave_polygon() {
        // A U shape: lines crossing the gap must split in two.
        let u = [
            pos2(0.0, 0.0),
            pos2(30.0, 0.0),
            pos2(30.0, 30.0),
            pos2(20.0, 30.0),
            pos2(20.0, 10.0),
            pos2(10.0, 10.0),
            pos2(10.0, 30.0),
            pos2(0.0, 30.0),
        ];
        let lines = hatch(&u, 0.0, 5.0);
        let low: Vec<_> = lines.iter().filter(|l| l[0].y > 12.0).collect();
        assert!(!low.is_empty());
        assert!(low.iter().all(|l| (l[1].x - l[0].x).abs() <= 10.01));
    }

    #[test]
    fn the_outline_joins_front_and_body_without_repeating_points() {
        let front: &[P] = &[(0.5, 0.0), (1.0, 0.1)];
        let body: &[P] = &[(1.0, 0.1), (1.0, 1.0), (0.0, 1.0), (0.0, 0.1), (0.5, 0.0)];
        let o = outline(front, body);
        assert_eq!(o[0], pos2(0.5, 0.0));
        assert_eq!(o[1], pos2(1.0, 0.1));
        for w in o.windows(2) {
            assert_ne!(w[0], w[1]);
        }
        assert_ne!(o[0], *o.last().unwrap());
    }
}
