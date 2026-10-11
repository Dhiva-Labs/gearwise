//! Painting helpers for the Corsair and SteelSeries drawings (the SteelSeries models reuse them):
//! keys with a dark gap round a lit face, grooves, a rubber wheel with light rails, a braided
//! cable, a neutral hex mark and the usual all-round rim. Everything is egui shapes, no textures.

use std::f32::consts::PI;

use egui::epaint::PathStroke;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, Vec2, pos2, vec2};

use crate::illustration::models::kit::{self, LIGHT, Plastic};
use crate::illustration::shapes::{edge_lights, fill_poly, glow, glow_line, gradient_fan};
use crate::theme::{Palette, mix, with_alpha};

/// A key or button: a dark gap polygon, then the face inset by `gap` screen px, lit from the
/// top and rim-lit along its upper-left edges.
#[allow(clippy::too_many_arguments)]
pub fn key(
    p: &Painter,
    pts: &[Pos2],
    sk: &Plastic,
    gap: f32,
    dir: Vec2,
    from: Color32,
    to: Color32,
    edge: f32,
) {
    kit::fill_flat(p, pts, sk.black);
    let face = if gap > 0.1 {
        kit::grown(pts, -gap)
    } else {
        pts.to_vec()
    };
    kit::facet(p, &face, dir, from, to, sk.hi, edge);
}

/// A dark groove with a faint lit lip below it, along a polyline.
pub fn groove(p: &Painter, pts: &[Pos2], width: f32, sk: &Plastic) {
    p.add(Shape::line(
        pts.to_vec(),
        PathStroke::new(width, with_alpha(sk.black, 235)),
    ));
    let lit: Vec<Pos2> = pts
        .iter()
        .map(|q| *q + vec2(0.5 * width, 0.9 * width))
        .collect();
    p.add(Shape::line(
        lit,
        PathStroke::new((width * 0.35).max(0.8), with_alpha(sk.hi, 44)),
    ));
}

/// A soft specular bloom over a region of the shell.
pub fn sheen(p: &Painter, c: Pos2, radii: Vec2, sk: &Plastic, pal: &Palette, peak: f32) {
    let ring: Vec<Pos2> = (0..32)
        .map(|i| {
            let a = i as f32 * PI / 16.0;
            c + vec2(a.cos() * radii.x, a.sin() * radii.y)
        })
        .collect();
    let peak = if pal.dark { peak } else { peak * 1.2 };
    p.add(gradient_fan(&ring, |q| {
        let d = ((q - c) / radii).length().min(1.0);
        with_alpha(sk.hi, (peak * (1.0 - d * d)) as u8)
    }));
}

/// The faint rim all round that keeps a dark shell readable on a dark card, then the crisp
/// highlight along the lit upper-left of the silhouette.
pub fn rim(p: &Painter, outline: &[Pos2], sk: &Plastic) {
    p.add(Shape::closed_line(
        outline.to_vec(),
        PathStroke::new(1.0, with_alpha(sk.hi, 34)),
    ));
    edge_lights(p, outline, LIGHT, sk.hi, 0.55);
}

/// A rubber scroll wheel in `rect` (axis along y): a dark tyre shaded towards its sides with a
/// triangular tread, and thin light rails at both edges that glow in `rails` when it is lit.
pub fn rubber_wheel(p: &Painter, rect: Rect, sk: &Plastic, rails: Option<Color32>, scale: f32) {
    let r = CornerRadius::same((3.0 * scale).clamp(1.0, 8.0) as u8);
    let cx = rect.center().x;
    let half = rect.width() / 2.0;
    let poly = crate::illustration::shapes::rounded_poly(rect, 3.0 * scale, 3);
    fill_poly(p, &poly, |q| {
        mix(
            sk.deep,
            sk.low,
            (1.0 - ((q.x - cx).abs() / half).powi(2)).clamp(0.0, 1.0),
        )
    });
    // Triangular tread: three families of lines 60 degrees apart, clipped to the wheel.
    let q = p.with_clip_rect(rect.intersect(p.clip_rect()));
    let pitch = (6.5 * scale).max(3.2);
    let ink = with_alpha(sk.high, 120);
    let h = rect.height();
    let mut y = rect.min.y;
    while y <= rect.max.y {
        q.hline(rect.x_range(), y, Stroke::new(0.9, ink));
        y += pitch * 0.866;
    }
    for sign in [-1.0f32, 1.0] {
        let slope = 1.732 * sign;
        let mut x0 = rect.min.x - h / 1.732;
        while x0 <= rect.max.x + h / 1.732 {
            q.line_segment(
                [pos2(x0, rect.min.y), pos2(x0 + h / slope, rect.max.y)],
                Stroke::new(0.9, ink),
            );
            x0 += pitch;
        }
    }
    p.rect_stroke(
        rect,
        r,
        Stroke::new(1.0, with_alpha(Color32::BLACK, 160)),
        egui::StrokeKind::Inside,
    );
    // Rails.
    let w = (rect.width() * 0.1).max(1.6);
    for x in [rect.min.x + w * 0.5, rect.max.x - w * 0.5] {
        let seg = [pos2(x, rect.min.y + 1.0), pos2(x, rect.max.y - 1.0)];
        match rails {
            Some(c) => {
                glow_line(p, &seg, w, c);
                p.line_segment(seg, Stroke::new(w, mix(c, Color32::WHITE, 0.3)));
            }
            None => {
                p.line_segment(seg, Stroke::new(w, mix(sk.low, sk.hi, 0.18)));
            }
        }
    }
}

/// A short braided cable, `len` px long, coming down to `bottom` at `x`.
pub fn cable(p: &Painter, x: f32, top: f32, bottom: f32, width: f32, sk: &Plastic) {
    p.line_segment(
        [pos2(x, top), pos2(x, bottom)],
        Stroke::new(width, mix(sk.low, sk.mid, 0.6)),
    );
    let step = (width * 0.9).max(3.0);
    let mut y = top;
    while y < bottom {
        p.line_segment(
            [
                pos2(x - width * 0.5, y),
                pos2(x + width * 0.5, y + step * 0.7),
            ],
            Stroke::new(0.8, with_alpha(sk.high, 70)),
        );
        y += step;
    }
    p.line_segment(
        [pos2(x - width * 0.3, top), pos2(x - width * 0.3, bottom)],
        Stroke::new(1.0, with_alpha(sk.hi, 45)),
    );
}

/// A hexagon with its first corner at `start` radians.
pub fn hexagon(c: Pos2, r: f32, start: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            let a = start + i as f32 * PI / 3.0;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

/// A neutral mark for a logo spot: a hexagon ring round a dot, glowing in `lit` when the logo
/// zone is on, a quiet engraving otherwise. Radius `r` is the spot's size.
pub fn hex_mark(p: &Painter, c: Pos2, r: f32, lit: Option<Color32>, sk: &Plastic) {
    let mut ring = hexagon(c, r * 0.8, PI / 6.0);
    ring.push(ring[0]);
    let width = (r * 0.2).max(1.4);
    match lit {
        Some(col) => {
            glow_line(p, &ring, width, col);
            p.add(Shape::line(
                ring,
                PathStroke::new(width, mix(col, Color32::WHITE, 0.3)),
            ));
            glow(p, c, r * 0.14, mix(col, Color32::WHITE, 0.25));
        }
        None => {
            let ink = mix(sk.low, sk.hi, 0.22);
            p.add(Shape::line(ring, PathStroke::new(width, ink)));
            p.circle_filled(c, r * 0.14, ink);
        }
    }
}

/// A chevron (`^`, or `v` when `up` is false) of half-width `w`, centred on `c`.
pub fn chevron(p: &Painter, c: Pos2, w: f32, up: bool, color: Color32) {
    let k = if up { -1.0 } else { 1.0 };
    p.add(Shape::line(
        vec![
            c + vec2(-w, -k * w * 0.5),
            c + vec2(0.0, k * w * 0.5),
            c + vec2(w, -k * w * 0.5),
        ],
        PathStroke::new(1.4, color),
    ));
}

/// Is `q` inside the polygon `poly` (even-odd rule)?
pub fn inside(poly: &[Pos2], q: Pos2) -> bool {
    let mut hit = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > q.y) != (b.y > q.y) && q.x < (b.x - a.x) * (q.y - a.y) / (b.y - a.y) + a.x {
            hit = !hit;
        }
        j = i;
    }
    hit
}

/// A lattice of small diamonds (an embossed grip texture) over the part of `poly` that can hold
/// them. Skipped when the pitch is too small to read.
pub fn lattice(p: &Painter, poly: &[Pos2], pitch: f32, ink: Color32) {
    if pitch < 7.0 {
        return;
    }
    let bounds = Rect::from_points(poly);
    let r = pitch * 0.32;
    let mut y = bounds.min.y + r;
    let mut row = 0;
    while y < bounds.max.y {
        let mut x = bounds.min.x + r + if row % 2 == 0 { 0.0 } else { pitch * 0.5 };
        while x < bounds.max.x {
            let c = pos2(x, y);
            if [vec2(r, 0.0), vec2(-r, 0.0), vec2(0.0, r), vec2(0.0, -r)]
                .iter()
                .all(|d| inside(poly, c + *d))
            {
                p.add(Shape::closed_line(
                    vec![
                        c + vec2(0.0, -r),
                        c + vec2(r, 0.0),
                        c + vec2(0.0, r),
                        c + vec2(-r, 0.0),
                    ],
                    PathStroke::new(0.9, ink),
                ));
            }
            x += pitch;
        }
        y += pitch * 0.5;
        row += 1;
    }
}

/// Soft glow along a polyline in `color` with a bright core.
pub fn light_line(p: &Painter, pts: &[Pos2], width: f32, color: Color32) {
    glow_line(p, pts, width, color);
    p.add(Shape::line(
        pts.to_vec(),
        PathStroke::new(width, mix(color, Color32::WHITE, 0.3)),
    ));
}

/// A neutral mark for a logo spot: a diamond round a dot, glowing in `lit` when the logo zone is
/// on, a quiet engraving otherwise.
pub fn diamond_mark(p: &Painter, c: Pos2, r: f32, lit: Option<Color32>, sk: &Plastic) {
    let k = r * 0.78;
    let mut ring = vec![
        c + vec2(0.0, -k),
        c + vec2(k, 0.0),
        c + vec2(0.0, k),
        c + vec2(-k, 0.0),
    ];
    ring.push(ring[0]);
    let width = (r * 0.2).max(1.4);
    match lit {
        Some(col) => {
            glow_line(p, &ring, width, col);
            p.add(Shape::line(
                ring,
                PathStroke::new(width, mix(col, Color32::WHITE, 0.3)),
            ));
            glow(p, c, r * 0.14, mix(col, Color32::WHITE, 0.25));
        }
        None => {
            let ink = mix(sk.low, sk.hi, 0.22);
            p.add(Shape::line(ring, PathStroke::new(width, ink)));
            p.circle_filled(c, r * 0.14, ink);
        }
    }
}

/// A rounded diamond of half-diagonal `h` centred on `c`, as `N` points (an Aerox-style cut-out).
pub fn rounded_diamond<const N: usize>(
    c: (f32, f32),
    h: (f32, f32),
    roundness: f32,
) -> [(f32, f32); N] {
    let k = 2.0 / roundness;
    std::array::from_fn(|i| {
        let a = i as f32 * 2.0 * PI / N as f32;
        let (s, co) = a.sin_cos();
        (
            c.0 + h.0 * co.signum() * co.abs().powf(k),
            c.1 + h.1 * s.signum() * s.abs().powf(k),
        )
    })
}

/// Fill many small polygons (cut-outs) in one mesh. Each `holes` entry is a closed outline in
/// screen space; `color_at` colours every vertex, and the centre gets `centre_at`.
pub fn hole_mesh(
    p: &Painter,
    holes: &[Vec<Pos2>],
    color_at: impl Fn(Pos2) -> Color32,
    centre_at: impl Fn(Pos2) -> Color32,
) {
    use egui::epaint::{Mesh, Vertex, WHITE_UV};
    let mut mesh = Mesh::default();
    for h in holes {
        let c = h.iter().fold(Pos2::ZERO, |a, q| a + q.to_vec2()) / h.len() as f32;
        let base = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex {
            pos: c,
            uv: WHITE_UV,
            color: centre_at(c),
        });
        for q in h {
            mesh.vertices.push(Vertex {
                pos: *q,
                uv: WHITE_UV,
                color: color_at(*q),
            });
        }
        let n = h.len() as u32;
        for i in 0..n {
            mesh.add_triangle(base, base + 1 + i, base + 1 + (i + 1) % n);
        }
    }
    p.add(Shape::mesh(mesh));
}
