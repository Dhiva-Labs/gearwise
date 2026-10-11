//! Geometry and painting helpers shared by the device art: béziers, rounded and
//! rotated polygons, gradient meshes and soft glows.

use egui::epaint::{Mesh, PathStroke, Vertex, WHITE_UV};
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2, pos2, vec2};

use crate::theme::with_alpha;

/// Flatten a cubic bézier into `n + 1` points.
pub fn bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            pos2(
                w[0] * p0.x + w[1] * p1.x + w[2] * p2.x + w[3] * p3.x,
                w[0] * p0.y + w[1] * p1.y + w[2] * p2.y + w[3] * p3.y,
            )
        })
        .collect()
}

/// Corner arcs of a rounded rectangle, clockwise from the top-left.
pub fn rounded_poly(rect: Rect, radius: f32, seg: usize) -> Vec<Pos2> {
    let r = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let corners = [
        (pos2(rect.min.x + r, rect.min.y + r), 180.0f32),
        (pos2(rect.max.x - r, rect.min.y + r), 270.0),
        (pos2(rect.max.x - r, rect.max.y - r), 0.0),
        (pos2(rect.min.x + r, rect.max.y - r), 90.0),
    ];
    let mut out = Vec::with_capacity(4 * (seg + 1));
    for (c, start) in corners {
        for i in 0..=seg {
            let a = (start + 90.0 * i as f32 / seg as f32).to_radians();
            out.push(c + vec2(a.cos(), a.sin()) * r);
        }
    }
    out
}

/// A rounded rectangle of `w` x `h` rotated by `angle` radians around `center`.
/// Local +y runs along the rotated "length" axis (so `angle = 0` is upright).
pub fn rot_rrect(center: Pos2, w: f32, h: f32, angle: f32, radius: f32) -> Vec<Pos2> {
    let (s, c) = angle.sin_cos();
    rounded_poly(Rect::from_center_size(Pos2::ZERO, vec2(w, h)), radius, 4)
        .into_iter()
        .map(|p| center + vec2(p.x * c - p.y * s, p.x * s + p.y * c))
        .collect()
}

pub fn centroid(points: &[Pos2]) -> Pos2 {
    let sum = points.iter().fold(Pos2::ZERO, |a, p| a + p.to_vec2());
    sum / points.len().max(1) as f32
}

/// Convex hull (Andrew's monotone chain), counter-clockwise in screen space.
pub fn convex_hull(points: &[Pos2]) -> Vec<Pos2> {
    let mut pts: Vec<Pos2> = points.to_vec();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: Pos2, a: Pos2, b: Pos2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut hull: Vec<Pos2> = Vec::with_capacity(pts.len() * 2);
    for pass in 0..2 {
        let start = hull.len();
        let iter: Box<dyn Iterator<Item = &Pos2>> = if pass == 0 {
            Box::new(pts.iter())
        } else {
            Box::new(pts.iter().rev())
        };
        for p in iter {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], *p) <= 0.0
            {
                hull.pop();
            }
            hull.push(*p);
        }
        hull.pop();
    }
    hull
}

/// Fill a star-shaped polygon with a colour computed per vertex (a triangle fan
/// from its centroid), which gives smooth gradients without textures.
pub fn gradient_fan(points: &[Pos2], color_at: impl Fn(Pos2) -> Color32) -> Shape {
    let centre = centroid(points);
    let mut mesh = Mesh::default();
    mesh.vertices.push(Vertex {
        pos: centre,
        uv: WHITE_UV,
        color: color_at(centre),
    });
    for p in points {
        mesh.vertices.push(Vertex {
            pos: *p,
            uv: WHITE_UV,
            color: color_at(*p),
        });
    }
    let n = points.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    Shape::mesh(mesh)
}

/// Vertical two-colour gradient fan over `points`.
pub fn vertical_fan(points: &[Pos2], top: Color32, bottom: Color32) -> Shape {
    let (y0, y1) = points
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
    gradient_fan(points, |p| {
        crate::theme::mix(
            top,
            bottom,
            ((p.y - y0) / (y1 - y0).max(1.0)).clamp(0.0, 1.0),
        )
    })
}

/// Chaikin corner cutting on an open polyline: rounds the corners but keeps both end points.
pub fn chaikin(pts: &[Pos2], rounds: usize) -> Vec<Pos2> {
    let mut cur = pts.to_vec();
    for _ in 0..rounds {
        if cur.len() < 3 {
            break;
        }
        let mut next = Vec::with_capacity(cur.len() * 2);
        next.push(cur[0]);
        for w in cur.windows(2) {
            next.push(w[0] + (w[1] - w[0]) * 0.25);
            next.push(w[0] + (w[1] - w[0]) * 0.75);
        }
        next.push(cur[cur.len() - 1]);
        cur = next;
    }
    cur
}

/// Twice the signed area (positive when the polygon runs clockwise on screen).
fn area2(pts: &[Pos2]) -> f32 {
    (0..pts.len())
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            a.x * b.y - b.x * a.y
        })
        .sum()
}

/// Ear-clipping triangulation of a simple polygon of either winding (indices into `pts`).
pub fn triangulate(pts: &[Pos2]) -> Vec<[u32; 3]> {
    let n = pts.len();
    if n < 3 {
        return Vec::new();
    }
    let mut idx: Vec<usize> = (0..n).collect();
    if area2(pts) < 0.0 {
        idx.reverse();
    }
    let cross = |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    // Tolerances relative to the polygon's size, so tiny (normalised) polygons work too.
    let (lo, hi) = pts.iter().fold((pts[0], pts[0]), |(lo, hi), p| {
        (
            pos2(lo.x.min(p.x), lo.y.min(p.y)),
            pos2(hi.x.max(p.x), hi.y.max(p.y)),
        )
    });
    let eps = (hi - lo).length_sq() * 1e-9;
    // Inclusive of the boundary, so a vertex touching an ear's far edge blocks it; vertices
    // that merely coincide with a corner do not.
    let inside = |a: Pos2, b: Pos2, c: Pos2, p: Pos2| {
        let same = |q: Pos2| (p - q).length_sq() <= eps;
        !(same(a) || same(b) || same(c))
            && cross(a, b, p) >= -eps
            && cross(b, c, p) >= -eps
            && cross(c, a, p) >= -eps
    };
    let mut out = Vec::with_capacity(n - 2);
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let (ia, ib, ic) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            let turn = cross(a, b, c);
            // Reflex corners cannot be ears; flat ones are clipped as zero-area triangles.
            if turn < -eps {
                continue;
            }
            let blocked = turn > eps
                && idx
                    .iter()
                    .any(|&k| k != ia && k != ib && k != ic && inside(a, b, c, pts[k]));
            if !blocked {
                out.push([ia as u32, ib as u32, ic as u32]);
                idx.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            // Degenerate leftovers: drop a vertex rather than loop forever.
            idx.remove(0);
        }
    }
    if idx.len() == 3 {
        out.push([idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    }
    out
}

/// Fill a simple (possibly concave) polygon, colouring each vertex with `color_at`. An affine
/// `color_at` (a linear gradient) is reproduced exactly. A hairline in the average colour
/// anti-aliases the edge, which a bare mesh does not get.
pub fn fill_poly(p: &Painter, pts: &[Pos2], color_at: impl Fn(Pos2) -> Color32) {
    if pts.len() < 3 {
        return;
    }
    let mut mesh = Mesh {
        vertices: pts
            .iter()
            .map(|q| Vertex {
                pos: *q,
                uv: WHITE_UV,
                color: color_at(*q),
            })
            .collect(),
        ..Default::default()
    };
    for t in triangulate(pts) {
        mesh.add_triangle(t[0], t[1], t[2]);
    }
    let avg = color_at(centroid(pts));
    p.add(Shape::mesh(mesh));
    p.add(Shape::closed_line(pts.to_vec(), PathStroke::new(0.9, avg)));
}

/// Linear gradient along `dir` between the bounding box corners of `pts`.
pub fn linear(pts: &[Pos2], dir: Vec2, from: Color32, to: Color32) -> impl Fn(Pos2) -> Color32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for q in pts {
        let d = q.to_vec2().dot(dir);
        lo = lo.min(d);
        hi = hi.max(d);
    }
    move |q: Pos2| {
        crate::theme::mix(
            from,
            to,
            ((q.to_vec2().dot(dir) - lo) / (hi - lo).max(1.0)).clamp(0.0, 1.0),
        )
    }
}

/// Crisp highlight along the edges of `pts` that face `light` (a vector pointing at the
/// light), and a faint shade on the opposite ones, so flat facets read as bevelled.
pub fn edge_lights(p: &Painter, pts: &[Pos2], light: Vec2, hi: Color32, strength: f32) {
    let sign = if area2(pts) >= 0.0 { 1.0 } else { -1.0 };
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let d = b - a;
        if d.length() < 2.0 {
            continue;
        }
        let normal = vec2(d.y, -d.x).normalized() * sign;
        let k = normal.dot(light);
        if k > 0.3 {
            let alpha = (255.0 * strength * ((k - 0.3) / 0.7).clamp(0.0, 1.0)) as u8;
            p.line_segment([a, b], Stroke::new(1.1, with_alpha(hi, alpha)));
        } else if k < -0.5 {
            p.line_segment(
                [a, b],
                Stroke::new(1.0, Color32::from_black_alpha((60.0 * strength) as u8)),
            );
        }
    }
}

/// Soft round glow: translucent discs growing outwards around a solid core.
pub fn glow(p: &Painter, center: Pos2, radius: f32, color: Color32) {
    for i in (1..=7).rev() {
        let k = i as f32 / 7.0;
        p.circle_filled(
            center,
            radius * (1.0 + 2.0 * k),
            with_alpha(color, (20.0 * (1.0 - k) + 4.0) as u8),
        );
    }
    p.circle_filled(center, radius, color);
}

/// Soft glow along a polyline: wide faint strokes under narrower brighter ones.
pub fn glow_line(p: &Painter, pts: &[Pos2], width: f32, color: Color32) {
    for (k, a) in [(3.2, 14u8), (2.2, 28), (1.5, 54), (1.0, 120)] {
        p.add(Shape::line(
            pts.to_vec(),
            PathStroke::new(width * k, with_alpha(color, a)),
        ));
    }
}

/// Light then dark hairline pair that reads as a seam or groove.
pub fn seam(p: &Painter, pts: &[Pos2], dark: Color32, light: Color32) {
    p.add(Shape::line(pts.to_vec(), PathStroke::new(1.3, dark)));
    let lit: Vec<Pos2> = pts.iter().map(|q| *q + vec2(0.0, 1.2)).collect();
    p.add(Shape::line(lit, PathStroke::new(1.0, light)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_hits_its_end_points_and_stays_in_the_hull() {
        let pts = bezier(
            pos2(0.0, 0.0),
            pos2(0.0, 10.0),
            pos2(10.0, 10.0),
            pos2(10.0, 0.0),
            16,
        );
        assert_eq!(pts.len(), 17);
        assert_eq!(pts[0], pos2(0.0, 0.0));
        assert!((pts[16] - pos2(10.0, 0.0)).length() < 1e-4);
        assert!(
            pts.iter()
                .all(|p| (0.0..=10.0).contains(&p.x) && (0.0..=10.0).contains(&p.y))
        );
    }

    #[test]
    fn rounded_polygon_stays_inside_its_rect() {
        let r = Rect::from_min_size(pos2(10.0, 20.0), vec2(40.0, 24.0));
        let pts = rounded_poly(r, 8.0, 4);
        assert_eq!(pts.len(), 20);
        assert!(pts.iter().all(|p| r.expand(0.01).contains(*p)));
        // A radius larger than half the short side is clamped, not inverted.
        assert!(
            rounded_poly(r, 100.0, 4)
                .iter()
                .all(|p| r.expand(0.01).contains(*p))
        );
    }

    #[test]
    fn hull_drops_interior_points() {
        let square = [
            pos2(0.0, 0.0),
            pos2(10.0, 0.0),
            pos2(10.0, 10.0),
            pos2(0.0, 10.0),
            pos2(5.0, 5.0),
            pos2(2.0, 8.0),
        ];
        let hull = convex_hull(&square);
        assert_eq!(hull.len(), 4);
        assert!(!hull.contains(&pos2(5.0, 5.0)));
        assert_eq!(convex_hull(&[pos2(1.0, 1.0)]).len(), 1);
    }

    #[test]
    fn triangulation_covers_concave_polygons_of_either_winding() {
        let l = [
            pos2(0.0, 0.0),
            pos2(2.0, 0.0),
            pos2(2.0, 1.0),
            pos2(1.0, 1.0),
            pos2(1.0, 2.0),
            pos2(0.0, 2.0),
        ];
        let area = |pts: &[Pos2]| -> f32 {
            triangulate(pts)
                .iter()
                .map(|[a, b, c]| {
                    let (a, b, c) = (pts[*a as usize], pts[*b as usize], pts[*c as usize]);
                    (((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)) / 2.0).abs()
                })
                .sum()
        };
        assert!((area(&l) - 3.0).abs() < 1e-4);
        let mut rev = l.to_vec();
        rev.reverse();
        assert!((area(&rev) - 3.0).abs() < 1e-4);
        // Tiny polygons with collinear points still triangulate fully.
        let tiny: Vec<Pos2> = [(0.0, 0.0), (0.5, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
            .iter()
            .map(|(x, y)| pos2(x * 1e-3, y * 1e-3))
            .collect();
        assert_eq!(triangulate(&tiny).len(), 3);
    }

    #[test]
    fn chaikin_keeps_end_points_and_rounds_corners() {
        let run = [pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0)];
        let s = chaikin(&run, 2);
        assert_eq!(s.first(), Some(&run[0]));
        assert_eq!(s.last(), Some(&run[2]));
        assert!(s.len() > 3);
        assert!(!s.contains(&pos2(10.0, 0.0)), "the corner is cut");
    }

    #[test]
    fn rotation_keeps_the_centre() {
        let pts = rot_rrect(pos2(5.0, 5.0), 10.0, 20.0, 0.7, 3.0);
        let c = centroid(&pts);
        assert!((c - pos2(5.0, 5.0)).length() < 0.5);
        let upright = rot_rrect(pos2(0.0, 0.0), 10.0, 20.0, 0.0, 0.0);
        assert!(
            upright
                .iter()
                .all(|p| p.x.abs() <= 5.01 && p.y.abs() <= 10.01)
        );
    }
}
