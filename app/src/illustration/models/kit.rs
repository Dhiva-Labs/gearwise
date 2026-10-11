//! Building blocks shared by the model drawings: the reference-pixel frame, a plastic colour
//! ramp, flat-shaded facets and the generic parts of a paint pass (shadow, hover tint, zone
//! rings). Models are free to ignore any of it; the existing ones use all of it.

use egui::epaint::{Mesh, PathStroke, Vertex, WHITE_UV};
use egui::{
    Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::illustration::mouse::MouseArt;
use crate::illustration::roles::{MouseLook, Role, ZoneRole};
use crate::illustration::shapes::{edge_lights, fill_poly, linear, triangulate};
use crate::theme::{Palette, with_alpha};

/// A point: normalised (0-1 across the art box) or in reference pixels, as the call says.
pub type P = (f32, f32);

/// Direction towards the light, in screen space (up and slightly left).
pub const LIGHT: Vec2 = vec2(-0.55, -0.83);
/// The usual gradient direction for a facet: lit at the top, shaded towards the tail.
pub const DOWN: Vec2 = vec2(0.28, 0.96);

/// Maps the 0-1 art box and a model's reference pixel grid (the pixels of the product photo it
/// was measured from) onto a screen rect.
#[derive(Clone, Copy)]
pub struct Frame {
    pub rect: Rect,
    origin: P,
    size: P,
}

impl Frame {
    /// `origin` and `size` are the shell's bounding box in reference pixels.
    pub fn new(rect: Rect, origin: P, size: P) -> Self {
        Self { rect, origin, size }
    }

    /// A point of the 0-1 art box.
    pub fn pt(&self, q: P) -> Pos2 {
        pos2(
            self.rect.min.x + q.0 * self.rect.width(),
            self.rect.min.y + q.1 * self.rect.height(),
        )
    }

    /// A point given in reference pixels.
    pub fn px(&self, q: P) -> Pos2 {
        self.pt((
            (q.0 - self.origin.0) / self.size.0,
            (q.1 - self.origin.1) / self.size.1,
        ))
    }

    pub fn pts(&self, q: &[P]) -> Vec<Pos2> {
        q.iter().map(|p| self.pt(*p)).collect()
    }

    /// Normalised positions (a `Pos2` per point) to screen.
    pub fn norm(&self, q: &[Pos2]) -> Vec<Pos2> {
        q.iter().map(|p| self.pt((p.x, p.y))).collect()
    }

    /// A rect given by two corners in reference pixels.
    pub fn rect_px(&self, r: (P, P)) -> Rect {
        Rect::from_two_pos(self.px(r.0), self.px(r.1))
    }

    /// Screen pixels per reference pixel.
    pub fn s(&self) -> f32 {
        self.rect.height() / self.size.1
    }
}

/// A plastic in five steps from the darkest gap to the brightest facet, plus the edge light.
/// Dark and light themes get slightly different steps so the shell reads on either card.
#[derive(Clone, Copy)]
pub struct Plastic {
    pub black: Color32,
    pub deep: Color32,
    pub low: Color32,
    pub mid: Color32,
    pub high: Color32,
    pub hi: Color32,
}

impl Plastic {
    fn ramp(steps: [[u8; 3]; 6]) -> Self {
        let c = |[r, g, b]: [u8; 3]| Color32::from_rgb(r, g, b);
        Self {
            black: c(steps[0]),
            deep: c(steps[1]),
            low: c(steps[2]),
            mid: c(steps[3]),
            high: c(steps[4]),
            hi: c(steps[5]),
        }
    }

    /// Gloss-less black with a blue cast (G502, most gaming mice).
    pub fn black(pal: &Palette) -> Self {
        Self::ramp(if pal.dark {
            [
                [0x06, 0x08, 0x0b],
                [0x0f, 0x12, 0x17],
                [0x18, 0x1c, 0x24],
                [0x25, 0x2a, 0x35],
                [0x38, 0x3f, 0x4d],
                [0xd6, 0xe2, 0xf4],
            ]
        } else {
            [
                [0x0d, 0x0f, 0x13],
                [0x15, 0x18, 0x1e],
                [0x20, 0x24, 0x2c],
                [0x2e, 0x34, 0x3f],
                [0x44, 0x4b, 0x5a],
                [0xf4, 0xf7, 0xfc],
            ]
        })
    }

    /// Soft-touch graphite, a little lighter and more neutral than black (MX series).
    pub fn graphite(pal: &Palette) -> Self {
        Self::ramp(if pal.dark {
            [
                [0x0a, 0x0b, 0x0d],
                [0x1f, 0x21, 0x25],
                [0x2e, 0x31, 0x36],
                [0x3e, 0x42, 0x47],
                [0x54, 0x58, 0x5e],
                [0xe4, 0xe8, 0xee],
            ]
        } else {
            [
                [0x10, 0x11, 0x14],
                [0x26, 0x28, 0x2c],
                [0x35, 0x38, 0x3d],
                [0x47, 0x4b, 0x51],
                [0x5e, 0x63, 0x6a],
                [0xff, 0xff, 0xff],
            ]
        })
    }

    /// White plastic: the "black" step is the seam colour, `high` is nearly paper white.
    #[allow(dead_code)] // for the white colourways added with the brand drawings
    pub fn white(pal: &Palette) -> Self {
        Self::ramp(if pal.dark {
            [
                [0x4a, 0x50, 0x5b],
                [0x9a, 0xa1, 0xad],
                [0xbc, 0xc2, 0xcc],
                [0xd9, 0xde, 0xe5],
                [0xee, 0xf1, 0xf6],
                [0xff, 0xff, 0xff],
            ]
        } else {
            [
                [0x55, 0x5b, 0x66],
                [0x8a, 0x91, 0x9d],
                [0xad, 0xb4, 0xbf],
                [0xcb, 0xd1, 0xda],
                [0xe3, 0xe7, 0xee],
                [0xff, 0xff, 0xff],
            ]
        })
    }
}

/// A flat-shaded facet: a linear gradient along `dir` from `from` to `to`, with the edges that
/// face the light picked out in `hi` (`edge` scales how strongly; 0 for none).
pub fn facet(
    p: &Painter,
    pts: &[Pos2],
    dir: Vec2,
    from: Color32,
    to: Color32,
    hi: Color32,
    edge: f32,
) {
    fill_poly(p, pts, linear(pts, dir, from, to));
    if edge > 0.0 {
        edge_lights(p, pts, LIGHT, hi, edge);
    }
}

/// Fill `rect` with a colour computed per vertex on a `cols` x `rows` grid. Unlike [`fill_poly`],
/// which only colours the corners of a shape, this can show shading that peaks in the middle
/// (a metal cylinder, a dome).
pub fn shaded_rect(
    p: &Painter,
    rect: Rect,
    cols: usize,
    rows: usize,
    color_at: impl Fn(Pos2) -> Color32,
) {
    let (cols, rows) = (cols.max(2), rows.max(2));
    let mut mesh = Mesh::default();
    for r in 0..rows {
        for c in 0..cols {
            let q = pos2(
                rect.min.x + rect.width() * c as f32 / (cols - 1) as f32,
                rect.min.y + rect.height() * r as f32 / (rows - 1) as f32,
            );
            mesh.vertices.push(Vertex {
                pos: q,
                uv: WHITE_UV,
                color: color_at(q),
            });
        }
    }
    let w = cols as u32;
    for r in 0..rows as u32 - 1 {
        for c in 0..w - 1 {
            let i = r * w + c;
            mesh.add_triangle(i, i + 1, i + w);
            mesh.add_triangle(i + 1, i + w + 1, i + w);
        }
    }
    p.add(Shape::mesh(mesh));
}

/// The soft contact shadow under a shell.
pub fn shadow(p: &Painter, outline: &[Pos2], rect: Rect, pal: &Palette) {
    let c = rect.center();
    let alpha = if pal.dark { 22 } else { 10 };
    for i in 1..=5 {
        let k = i as f32;
        let pts: Vec<Pos2> = outline
            .iter()
            .map(|q| pos2(q.x, q.y + 9.0) + (*q - c) * (0.012 * k))
            .collect();
        fill_poly(p, &pts, |_| Color32::from_black_alpha(alpha));
    }
}

/// Is the polygon convex (either winding)?
fn is_convex(pts: &[Pos2]) -> bool {
    let turn = |i: usize| {
        let (a, b, c) = (pts[i], pts[(i + 1) % pts.len()], pts[(i + 2) % pts.len()]);
        (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
    };
    let signs: Vec<f32> = (0..pts.len())
        .map(turn)
        .filter(|t| t.abs() > 1e-3)
        .collect();
    signs.iter().all(|t| *t > 0.0) || signs.iter().all(|t| *t < 0.0)
}

/// A flat fill for any simple polygon: egui's convex fill when it is convex, else triangulated.
pub fn fill_flat(p: &Painter, pts: &[Pos2], color: Color32) {
    if is_convex(pts) {
        p.add(Shape::convex_polygon(pts.to_vec(), color, Stroke::NONE));
    } else {
        let mut mesh = Mesh::default();
        for q in pts {
            mesh.vertices.push(Vertex {
                pos: *q,
                uv: WHITE_UV,
                color,
            });
        }
        for t in triangulate(pts) {
            mesh.add_triangle(t[0], t[1], t[2]);
        }
        p.add(Shape::mesh(mesh));
    }
}

/// Hover / selection tint over the parts named in `tint` (role and strength 0-1).
pub fn paint_tint(p: &Painter, art: &MouseArt, pal: &Palette, tint: &[(Role, f32)]) {
    for (role, strength) in tint {
        if let Some(part) = art.part(*role) {
            fill_flat(
                p,
                &part.poly,
                with_alpha(pal.accent, (84.0 * strength) as u8),
            );
            p.add(Shape::closed_line(
                part.poly.clone(),
                PathStroke::new(1.0 + strength, with_alpha(pal.accent, 230)),
            ));
        }
    }
}

/// The accent ring that marks the lighting zone selected in the UI. The logo and wheel rings are
/// the same everywhere; what "body" means differs per model, so `body` draws it.
pub fn zone_ring(
    p: &Painter,
    art: &MouseArt,
    pal: &Palette,
    look: &MouseLook,
    body: impl FnOnce(&Painter, Stroke),
) {
    let Some(role) = look.highlight else { return };
    let ring = Stroke::new(1.5, pal.accent);
    match role {
        ZoneRole::Logo => {
            p.circle_stroke(art.logo.0, art.logo.1 * 1.7, ring);
        }
        ZoneRole::Wheel => {
            p.rect_stroke(
                art.wheel.expand2(vec2(10.0, 12.0)),
                CornerRadius::same(14),
                ring,
                StrokeKind::Outside,
            );
        }
        ZoneRole::Body => body(p, ring),
    }
}

/// A part's polygon grown by `px` pixels around its centre, for outlines that must clear it.
pub fn grown(poly: &[Pos2], px: f32) -> Vec<Pos2> {
    let c = crate::illustration::shapes::centroid(poly);
    poly.iter()
        .map(|q| *q + (*q - c).normalized() * px)
        .collect()
}
