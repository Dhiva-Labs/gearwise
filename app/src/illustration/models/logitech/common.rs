//! Painting helpers shared by the Logitech drawings: the rim light, panels with their gaps,
//! lit bands along a flank, the neutral logo mark, LED dots and the ribbed scroll wheel.
//! Every model keeps its own geometry and decides the order things are painted in.

use std::f32::consts::PI;

use egui::epaint::PathStroke;
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
    pos2, vec2,
};

use crate::illustration::models::kit::{self, LIGHT, Plastic};
use crate::illustration::mouse::MouseArt;
use crate::illustration::roles::MouseLook;
use crate::illustration::shapes::{
    centroid, edge_lights, fill_poly, glow, glow_line, gradient_fan, linear, rounded_poly,
};
use crate::theme::{Palette, mix, with_alpha};

/// A faint rim all round keeps the dark shell readable on a dark card, then a crisp highlight
/// along the lit upper-left of the silhouette.
pub fn rim(p: &Painter, outline: &[Pos2], sk: &Plastic) {
    if std::env::var("ART_DBG").is_ok() {
        let r = Rect::from_points(outline);
        eprintln!(
            "ARTBOX {} {} {} {}",
            r.min.x,
            r.min.y,
            r.width(),
            r.height()
        );
    }
    p.add(Shape::closed_line(
        outline.to_vec(),
        PathStroke::new(1.0, with_alpha(sk.hi, 34)),
    ));
    edge_lights(p, outline, LIGHT, sk.hi, 0.55);
}

/// A soft specular bloom (an ellipse of `radii` around `c`) on a domed surface.
pub fn sheen(p: &Painter, c: Pos2, radii: Vec2, sk: &Plastic, pal: &Palette) {
    let ring: Vec<Pos2> = (0..32)
        .map(|i| {
            let a = i as f32 * PI / 16.0;
            c + vec2(a.cos() * radii.x, a.sin() * radii.y)
        })
        .collect();
    let peak = if pal.dark { 40.0 } else { 46.0 };
    p.add(gradient_fan(&ring, |q| {
        let d = ((q - c) / radii).length().min(1.0);
        with_alpha(sk.hi, (peak * (1.0 - d * d)) as u8)
    }));
}

/// `pts` pulled `d` px inwards along each edge's normal (mitred, so a long panel keeps an even
/// border); a negative `d` grows it. Either winding.
pub fn inset_poly(pts: &[Pos2], d: f32) -> Vec<Pos2> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let area: f32 = (0..n)
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    let sign = if area >= 0.0 { 1.0 } else { -1.0 };
    let normal = |a: Pos2, b: Pos2| {
        let e = (b - a).normalized();
        vec2(-e.y, e.x) * sign
    };
    (0..n)
        .map(|i| {
            let (a, v, b) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
            let (n1, n2) = (normal(a, v), normal(v, b));
            let miter = (n1 + n2) / (1.0 + n1.dot(n2)).max(0.35);
            v + miter * d
        })
        .collect()
}

/// A button panel: a dark underlay (the gap round it) with the lit face inset by `gap` px.
#[allow(clippy::too_many_arguments)]
pub fn panel(
    p: &Painter,
    pts: &[Pos2],
    dir: Vec2,
    from: Color32,
    to: Color32,
    sk: &Plastic,
    gap: f32,
    edge: f32,
) {
    kit::fill_flat(p, pts, sk.black);
    let face = inset_poly(pts, gap.max(1.0));
    kit::facet(p, &face, dir, from, to, sk.hi, edge);
}

/// `run` pulled `px` towards `toward`, tapering to nothing at both ends.
pub fn inset_run(run: &[Pos2], toward: Pos2, px: f32, taper: f32) -> Vec<Pos2> {
    let last = (run.len().max(2) - 1) as f32;
    run.iter()
        .enumerate()
        .map(|(i, q)| {
            let t = i as f32 / last;
            let k = (t.min(1.0 - t) / taper).clamp(0.0, 1.0);
            let k = k * k * (3.0 - 2.0 * k);
            let d = toward - *q;
            *q + d.normalized() * px * k
        })
        .collect()
}

/// A band lit from the outside: `run` (a stretch of the silhouette) and the same run pulled
/// `px` inwards, filled from `alpha` opacity of the highlight down to nothing.
pub fn band(p: &Painter, run: &[Pos2], toward: Pos2, px: f32, sk: &Plastic, alpha: u8) {
    let mut pts = run.to_vec();
    pts.extend(inset_run(run, toward, px, 0.25).into_iter().rev());
    let dir = (toward - centroid(run)).normalized();
    fill_poly(
        p,
        &pts,
        linear(&pts, dir, with_alpha(sk.hi, alpha), with_alpha(sk.hi, 0)),
    );
}

/// A dark groove with a light flank under it: a seam in the shell.
pub fn groove(p: &Painter, pts: &[Pos2], width: f32, sk: &Plastic) {
    p.add(Shape::line(pts.to_vec(), PathStroke::new(width, sk.black)));
    let lit: Vec<Pos2> = pts.iter().map(|q| *q + vec2(0.8, 1.1)).collect();
    p.add(Shape::line(
        lit,
        PathStroke::new(1.0, with_alpha(sk.hi, 46)),
    ));
}

/// A key or flank button: the lit face over a dark rim.
pub fn key(p: &Painter, pts: &[Pos2], dir: Vec2, from: Color32, to: Color32, sk: &Plastic) {
    kit::fill_flat(p, pts, sk.black);
    let face = inset_poly(pts, 1.4);
    kit::facet(p, &face, dir, from, to, sk.hi, 0.9);
}

/// An engraved label, shown when the render is big enough to read it.
pub fn engrave(p: &Painter, c: Pos2, text: &str, size: f32, color: Color32) {
    if size >= 5.6 {
        p.text(
            c,
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(size),
            color,
        );
    }
}

/// A small round LED: lit in `color`, else a dull lens.
pub fn led_dot(p: &Painter, c: Pos2, r: f32, color: Option<Color32>, sk: &Plastic) {
    match color {
        Some(col) => glow(p, c, r, mix(col, Color32::WHITE, 0.25)),
        None => {
            p.circle_filled(c, r, mix(sk.deep, sk.hi, 0.12));
            p.circle_stroke(c, r, Stroke::new(0.8, with_alpha(sk.hi, 40)));
        }
    }
}

/// The palm mark: an open ring and a dot (a neutral stand-in for the maker's mark), glowing
/// when its zone is lit.
pub fn logo(p: &Painter, art: &MouseArt, sk: &Plastic, look: &MouseLook) {
    let (c, r) = art.logo;
    let arc: Vec<Pos2> = (0..=24)
        .map(|i| {
            let a = (14.0 + 284.0 * i as f32 / 24.0).to_radians();
            c + vec2(a.cos(), a.sin()) * (r * 0.78)
        })
        .collect();
    let width = r * 0.34;
    match look.logo {
        Some(col) => {
            glow_line(p, &arc, width, col);
            p.add(Shape::line(
                arc,
                PathStroke::new(width, mix(col, Color32::WHITE, 0.3)),
            ));
            glow(p, c, r * 0.16, mix(col, Color32::WHITE, 0.25));
        }
        None => {
            p.add(Shape::line(
                arc,
                PathStroke::new(width, mix(sk.low, sk.hi, 0.2)),
            ));
            p.circle_filled(c, r * 0.16, mix(sk.low, sk.hi, 0.2));
        }
    }
}

/// How a scroll wheel looks.
#[derive(Clone, Copy)]
pub struct WheelLook {
    /// Bright metal flanges along both sides of the tyre (Superlight, MagSpeed).
    pub flanges: bool,
    /// Rib spacing in reference px; the wheel draws fewer, wider ribs when small.
    pub pitch: f32,
    /// A groove down the middle of the tyre (two columns of ribs).
    pub split: bool,
    /// How far (reference px) the ribs climb from the left side of the tyre to the right: 0 for
    /// straight ribs, more for a helical tread.
    pub slant: f32,
}

/// A scroll wheel in a dark slot: a ribbed tyre shaded towards its sides, a hairline each side
/// and, when `look` has one, the wheel light glowing behind it.
pub fn rubber_wheel(
    p: &Painter,
    slot: Rect,
    wheel: Rect,
    s: f32,
    sk: &Plastic,
    wheel_light: Option<Color32>,
    wl: WheelLook,
) {
    let r = CornerRadius::same((9.0 * s).round().clamp(1.0, 40.0) as u8);
    p.rect_filled(slot.expand(1.6 * s), r, with_alpha(Color32::BLACK, 120));
    p.rect_filled(slot, r, sk.black);
    p.rect_stroke(
        slot,
        r,
        Stroke::new(1.0, with_alpha(sk.hi, 50)),
        StrokeKind::Inside,
    );
    if let Some(c) = wheel_light {
        glow(p, wheel.center(), wheel.width() * 0.45, with_alpha(c, 60));
    }
    let poly = rounded_poly(wheel, 6.0 * s, 4);
    let cx = wheel.center().x;
    let half = wheel.width() / 2.0;
    let mid = wheel_light.map_or(sk.high, |c| mix(sk.high, c, 0.5));
    fill_poly(p, &poly, |q| {
        let side = ((q.x - cx).abs() / half).clamp(0.0, 1.0);
        mix(sk.low, mid, 1.0 - side.powi(2))
    });
    let pitch = (wl.pitch * s).max(3.4);
    let bands = (wheel.height() / pitch).floor().max(3.0) as i32;
    let step = wheel.height() / bands as f32;
    // With metal flanges the ribs only fill the middle of the tyre.
    let inner = if wl.flanges { 0.27 } else { 0.0 };
    let (x0, x1) = (
        wheel.min.x + wheel.width() * inner,
        wheel.max.x - wheel.width() * inner,
    );
    let rise = wl.slant * s;
    for i in 0..bands {
        let y = wheel.min.y + step * i as f32 + rise / 2.0;
        let (a, b) = (y + step * 0.6, y + step);
        let gap = vec![
            pos2(x0 + 0.5, a),
            pos2(x1 - 0.5, a - rise),
            pos2(x1 - 0.5, b - rise),
            pos2(x0 + 0.5, b),
        ];
        p.add(Shape::convex_polygon(
            gap,
            with_alpha(sk.black, 170),
            Stroke::NONE,
        ));
        p.add(Shape::line_segment(
            [pos2(x0 + 1.0, y + 0.7), pos2(x1 - 1.0, y + 0.7 - rise)],
            Stroke::new(1.0, with_alpha(sk.hi, 70)),
        ));
    }
    if wl.split {
        p.vline(
            wheel.center().x,
            wheel.min.y + 1.0..=wheel.max.y - 1.0,
            Stroke::new((1.6 * s).max(1.2), with_alpha(sk.black, 220)),
        );
    }
    if wl.flanges {
        let metal = Color32::from_rgb(0xf4, 0xf6, 0xfa);
        let shade = Color32::from_rgb(0x8a, 0x90, 0x9a);
        for (lo, hi) in [(0.0, inner), (1.0 - inner, 1.0)] {
            let r = Rect::from_min_max(
                pos2(wheel.min.x + wheel.width() * lo, wheel.min.y),
                pos2(wheel.min.x + wheel.width() * hi, wheel.max.y),
            )
            .shrink2(vec2(0.0, 0.8));
            // Brightest in the middle of the flange, darker at the outer edge.
            let outer = if lo == 0.0 { r.min.x } else { r.max.x };
            kit::shaded_rect(p, r, 5, 3, |q| {
                let t = ((q.x - outer).abs() / r.width()).clamp(0.0, 1.0);
                mix(shade, metal, (t * 1.6).min(1.0))
            });
        }
    }
    p.rect_stroke(
        wheel,
        CornerRadius::same((6.0 * s).round().clamp(0.0, 40.0) as u8),
        Stroke::new(1.0, with_alpha(Color32::BLACK, 150)),
        StrokeKind::Inside,
    );
}

/// A knurled metal wheel seen from above, its axis running across the mouse (the MagSpeed
/// wheel): bright where the cylinder faces the light, dark where it curves away, with fine
/// ribs along its length.
pub fn metal_wheel(p: &Painter, r: Rect, s: f32) {
    let bright = Color32::from_rgb(0xfa, 0xfb, 0xfd);
    let dark = Color32::from_rgb(0x4c, 0x51, 0x59);
    let (c, half) = (r.center(), r.size() / 2.0);
    kit::shaded_rect(p, r, 5, 12, |q| {
        let around = ((q.y - c.y) / half.y).clamp(-1.0, 1.0);
        let along = ((q.x - c.x) / half.x).clamp(-1.0, 1.0);
        let shade = (1.0 - 0.8 * around.abs().powf(2.4)) * (1.0 - 0.18 * along.powi(2));
        mix(dark, bright, shade.clamp(0.0, 1.0))
    });
    let pitch = (4.4 * s).max(3.4);
    let n = (r.height() / pitch).floor() as i32;
    for i in 1..n {
        let y = r.min.y + r.height() * i as f32 / n as f32;
        p.hline(
            r.min.x + 0.5..=r.max.x - 0.5,
            y,
            Stroke::new(1.0, with_alpha(Color32::BLACK, 90)),
        );
        p.hline(
            r.min.x + 0.5..=r.max.x - 0.5,
            y + 1.0,
            Stroke::new(1.0, with_alpha(Color32::WHITE, 70)),
        );
    }
    p.rect_stroke(
        r,
        CornerRadius::same((3.0 * s).round().clamp(0.0, 20.0) as u8),
        Stroke::new(1.0, with_alpha(Color32::BLACK, 150)),
        StrokeKind::Inside,
    );
}

/// The centre of a polygon's vertices.
pub fn centroid_of(pts: &[Pos2]) -> Pos2 {
    centroid(pts)
}

/// The shaded fill of a shell: lit at the top, darker towards the tail.
pub fn shell(p: &Painter, outline: &[Pos2], from: Color32, to: Color32, sk: &Plastic) {
    kit::facet(p, outline, kit::DOWN, from, to, sk.hi, 0.0);
}

/// `pts` with its first point repeated at the end, for a dashed or stroked loop.
pub fn closed(pts: &[Pos2]) -> Vec<Pos2> {
    [pts, &pts[..1]].concat()
}
