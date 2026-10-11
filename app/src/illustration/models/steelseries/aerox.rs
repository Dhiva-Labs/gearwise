//! The Aerox family (3, 5, 9): ultralight shells perforated with a lattice of rounded diamond
//! cut-outs, so the RGB inside shows through. All three share one painter and differ in outline,
//! buttons and where the perforations run: the 3 has the plain thumb pair, the 5 adds a stack of
//! flick keys on the left flank and the 9 a sliding panel of twelve keys, drawn flat against the
//! left flank like the Scimitar's. The three zones (Front, Middle, Rear) share one colour in the
//! preview, so the cut-outs glow in it, brightest at the rear. Geometry lives in
//! `aerox3_shapes`, `aerox5_shapes` and `aerox9_shapes`.

use std::sync::OnceLock;

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, pos2, vec2};

use super::{aerox3_shapes as a3, aerox5_shapes as a5, aerox9_shapes as a9};
use crate::illustration::models::corsair::common::{
    diamond_mark, groove, hole_mesh, inside, key, rim, rounded_diamond, rubber_wheel, sheen,
};
use crate::illustration::models::kit::{self, DOWN, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

type Poly = &'static [P];
type Box2 = (P, P);

/// Everything that makes one Aerox different from another.
struct Aerox {
    origin: P,
    size: P,
    outline: Poly,
    left: Poly,
    right: Poly,
    well: Poly,
    dpi: Poly,
    seam: Poly,
    /// Regions that carry cut-outs (normalised polygons).
    perf: &'static [Poly],
    /// Thumb keys on the left flank; `None` for a key the drivers do not expose.
    flank: &'static [(Option<Role>, Poly)],
    wheel: Box2,
    logo: (f32, f32, f32),
    anchors: (P, P),
    /// Distance between cut-out centres along each axis, and their half-diagonal (ref px).
    pitch: f32,
    half: f32,
    /// The Aerox 9's panel of twelve keys, 3 columns by 4 rows.
    panel: Option<Box2>,
    holes: OnceLock<Vec<Vec<Pos2>>>,
}

const COLS: usize = 3;
const ROWS: usize = 4;
const PAD: f32 = 6.0;
const GAP: f32 = 3.0;

static AEROX_3: Aerox = Aerox {
    origin: a3::ORIGIN,
    size: a3::SIZE,
    outline: a3::OUTLINE,
    left: a3::LEFT_BTN,
    right: a3::RIGHT_BTN,
    well: a3::WELL,
    dpi: a3::DPI_KEY,
    seam: a3::SEAM,
    perf: &[a3::PERF_L, a3::PERF_R, a3::PERF_BODY],
    flank: &[
        (Some(Role::Forward), a3::FORWARD),
        (Some(Role::Back), a3::BACK),
    ],
    wheel: a3::WHEEL,
    logo: a3::LOGO,
    anchors: (a3::ANCHOR_LEFT, a3::ANCHOR_RIGHT),
    pitch: 19.7,
    half: 14.8,
    panel: None,
    holes: OnceLock::new(),
};

static AEROX_5: Aerox = Aerox {
    origin: a5::ORIGIN,
    size: a5::SIZE,
    outline: a5::OUTLINE,
    left: a5::LEFT_BTN,
    right: a5::RIGHT_BTN,
    well: a5::WELL,
    dpi: a5::DPI_KEY,
    seam: a5::SEAM,
    perf: &[a5::PERF_L, a5::PERF_R, a5::PERF_BODY],
    flank: &[
        (Some(Role::Forward), a5::FORWARD),
        (Some(Role::Back), a5::BACK),
        (None, a5::FLICK),
    ],
    wheel: a5::WHEEL,
    logo: a5::LOGO,
    anchors: (a5::ANCHOR_LEFT, a5::ANCHOR_RIGHT),
    pitch: 19.7,
    half: 14.8,
    panel: None,
    holes: OnceLock::new(),
};

static AEROX_9: Aerox = Aerox {
    origin: a9::ORIGIN,
    size: a9::SIZE,
    outline: a9::OUTLINE,
    left: a9::LEFT_BTN,
    right: a9::RIGHT_BTN,
    well: a9::WELL,
    dpi: a9::DPI_KEY,
    seam: a9::SEAM,
    perf: &[a9::PERF_L, a9::PERF_R, a9::PERF_BODY],
    flank: &[],
    wheel: a9::WHEEL,
    logo: a9::LOGO,
    anchors: (a9::ANCHOR_LEFT, a9::ANCHOR_RIGHT),
    pitch: 20.4,
    half: 15.4,
    panel: Some(a9::PANEL),
    holes: OnceLock::new(),
};

macro_rules! variant {
    ($name:ident, $data:ident, $id:literal, $priority:literal, |$n:ident| $matches:expr, $examples:expr) => {
        pub struct $name;

        impl ModelArt for $name {
            fn id(&self) -> &'static str {
                $id
            }

            fn matches(&self, $n: &str) -> bool {
                $matches
            }

            fn priority(&self) -> i32 {
                $priority
            }

            fn examples(&self) -> &'static [&'static str] {
                $examples
            }

            fn aspect(&self) -> f32 {
                $data.size.0 / $data.size.1
            }

            fn build(&self, rect: Rect) -> Anatomy {
                $data.build(rect)
            }

            fn paint(
                &self,
                art: &MouseArt,
                p: &Painter,
                pal: &Palette,
                look: &MouseLook,
                tint: &[(Role, f32)],
            ) {
                $data.paint(art, p, pal, look, tint);
            }
        }
    };
}

variant!(
    Aerox3,
    AEROX_3,
    "aerox-3",
    0,
    |n| contains_any(n, &["aerox"]),
    &[
        "SteelSeries Aerox 3",
        "SteelSeries Aerox 3 Wireless",
        "SteelSeries Aerox 3 (demo)",
        "SteelSeries Aerox",
    ]
);
variant!(
    Aerox5,
    AEROX_5,
    "aerox-5",
    10,
    |n| contains_any(n, &["aerox 5"]),
    &[
        "SteelSeries Aerox 5",
        "SteelSeries Aerox 5 Wireless",
        "SteelSeries Aerox 5 (demo)",
    ]
);
variant!(
    Aerox9,
    AEROX_9,
    "aerox-9",
    10,
    |n| contains_any(n, &["aerox 9"]),
    &[
        "SteelSeries Aerox 9 Wireless",
        "SteelSeries Aerox 9 Wireless (demo)",
    ]
);

impl Aerox {
    fn frame(&self, rect: Rect) -> Frame {
        Frame::new(rect, self.origin, self.size)
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = self.frame(rect);
        let s = f.s();
        let part = |role, poly: Vec<Pos2>, anchor: Pos2| Part::new(role, poly).anchored(anchor);
        let wheel = f.rect_px(self.wheel);
        let centre = |poly: Poly| {
            let pts = f.pts(poly);
            crate::illustration::shapes::centroid(&pts)
        };
        let mut parts = vec![
            part(Role::Left, f.pts(self.left), f.px(self.anchors.0)),
            part(Role::Right, f.pts(self.right), f.px(self.anchors.1)),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                wheel.center(),
            ),
            part(Role::DpiCycle, f.pts(self.dpi), centre(self.dpi)),
        ];
        for (role, poly) in self.flank {
            if let Some(role) = role {
                parts.push(part(*role, f.pts(poly), centre(poly)));
            }
        }
        if self.panel.is_some() {
            for i in 0..COLS * ROWS {
                let cell = self.cell(i);
                let poly = rounded_poly(f.rect_px(cell), 4.0 * s, 3);
                let c = f.px(((cell.0.0 + cell.1.0) / 2.0, (cell.0.1 + cell.1.1) / 2.0));
                parts.push(part(Role::Grid(i as u8), poly, c).labelled(Side::Left));
            }
        }
        Anatomy {
            parts,
            wheel,
            logo: (f.px((self.logo.0, self.logo.1)), self.logo.2 * s),
        }
    }

    /// Reference-px rect of panel key `i`, row by row.
    fn cell(&self, i: usize) -> Box2 {
        let (p0, p1) = self.panel.unwrap_or(((0.0, 0.0), (1.0, 1.0)));
        let w = (p1.0 - p0.0 - 2.0 * PAD - (COLS - 1) as f32 * GAP) / COLS as f32;
        let h = (p1.1 - p0.1 - 2.0 * PAD - (ROWS - 1) as f32 * GAP) / ROWS as f32;
        let (r, c) = ((i / COLS) as f32, (i % COLS) as f32);
        let x0 = p0.0 + PAD + c * (w + GAP);
        let y0 = p0.1 + PAD + r * (h + GAP);
        ((x0, y0), (x0 + w, y0 + h))
    }

    /// The cut-outs, as closed outlines in the 0-1 art box, found once: every diamond of the
    /// lattice that fits inside a perforated region and clear of the wheel well and DPI key.
    fn holes(&self) -> &[Vec<Pos2>] {
        self.holes.get_or_init(|| {
            let (w, h) = self.size;
            let to_px =
                |poly: Poly| -> Vec<Pos2> { poly.iter().map(|q| pos2(q.0 * w, q.1 * h)).collect() };
            let regions: Vec<Vec<Pos2>> = self.perf.iter().map(|r| to_px(r)).collect();
            let blocked = [to_px(self.well), to_px(self.dpi)];
            let p = self.pitch;
            let mut out = Vec::new();
            let (rows, cols) = ((h / p) as i32 + 2, (w / p) as i32 + 2);
            for j in 0..rows {
                for i in -cols..cols {
                    if (i + j) % 2 != 0 {
                        continue;
                    }
                    let c = (w * 0.5 + i as f32 * p, j as f32 * p);
                    let pts = rounded_diamond::<12>(c, (self.half, self.half), 1.4);
                    let pts: Vec<Pos2> = pts.iter().map(|q| pos2(q.0, q.1)).collect();
                    let fits = regions.iter().any(|r| pts.iter().all(|q| inside(r, *q)));
                    let clear = blocked
                        .iter()
                        .all(|b| !pts.iter().any(|q| inside(b, *q)) && !inside(b, pos2(c.0, c.1)));
                    if fits && clear {
                        out.push(pts.iter().map(|q| pos2(q.x / w, q.y / h)).collect());
                    }
                }
            }
            out
        })
    }

    fn paint(
        &self,
        art: &MouseArt,
        p: &Painter,
        pal: &Palette,
        look: &MouseLook,
        tint: &[(Role, f32)],
    ) {
        let f = self.frame(art.rect);
        let s = f.s();
        let sk = Plastic::black(pal);
        let outline = f.pts(self.outline);

        kit::shadow(p, &outline, art.rect, pal);
        if self.panel.is_some() {
            self.paint_panel(p, &f, &sk, look, pal);
        }
        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.pt((0.5, 0.5)),
            vec2(0.34, 0.3) * art.rect.size(),
            &sk,
            pal,
            36.0,
        );
        for poly in [self.left, self.right] {
            let pts = f.pts(poly);
            kit::facet(
                p,
                &pts,
                vec2(0.2, 1.0),
                mix(sk.high, sk.mid, 0.4),
                sk.mid,
                sk.hi,
                0.7,
            );
        }

        // The wheel well, the wheel and the DPI key.
        let well = f.pts(self.well);
        kit::fill_flat(p, &well, sk.black);
        let wheel = f.rect_px(self.wheel);
        rubber_wheel(p, wheel, &sk, None, s);
        let dpi = f.pts(self.dpi);
        key(p, &dpi, &sk, (1.3 * s).max(1.0), DOWN, sk.mid, sk.deep, 0.9);

        self.paint_holes(p, &f, &sk, look);
        groove(p, &f.pts(self.seam), 2.0 * s, &sk);

        for (_, poly) in self.flank {
            let pts = f.pts(poly);
            key(p, &pts, &sk, 0.8, vec2(1.0, 0.0), sk.high, sk.mid, 0.6);
        }
        diamond_mark(
            p,
            f.px((self.logo.0, self.logo.1)),
            self.logo.2 * s,
            None,
            &sk,
        );

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        let body = self.perf[self.perf.len() - 1];
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let mut poly = kit::grown(&f.pts(body), 4.0);
            poly.push(poly[0]);
            p.extend(Shape::dashed_line(&poly, ring, 5.0, 4.0));
        });
    }

    /// The cut-outs: dark when unlit, glowing in the body colour (strongest at the rear) when lit.
    fn paint_holes(&self, p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
        let holes: Vec<Vec<Pos2>> = self
            .holes()
            .iter()
            .map(|h| h.iter().map(|q| f.pt((q.x, q.y))).collect())
            .collect();
        let rect = f.rect;
        let dark = with_alpha(sk.black, 245);
        // 0 at the front of the perforated body, 1 at the tail; brighter along the axis.
        let level = move |q: Pos2| {
            let t = ((q.y - rect.min.y) / rect.height() - 0.42) / 0.5;
            let axis = 1.0 - ((q.x - rect.center().x).abs() / (rect.width() * 0.5)).min(1.0);
            (0.3 + 0.7 * t.clamp(0.0, 1.0).powf(1.2) * (0.65 + 0.35 * axis)).clamp(0.0, 1.0)
        };
        match look.body {
            Some(c) => hole_mesh(
                p,
                &holes,
                |q| mix(dark, mix(c, Color32::WHITE, 0.1), level(q)),
                |q| mix(dark, mix(c, Color32::WHITE, 0.55), level(q).powf(0.8)),
            ),
            None => hole_mesh(p, &holes, |_| dark, |_| mix(dark, sk.low, 0.4)),
        }
        if let Some(c) = look.body {
            // A soft halo under the tail, where the light spills over the edge.
            let tail = f.pt((0.5, 0.95));
            glow(p, tail, rect.width() * 0.1, with_alpha(c, 14));
        }
    }

    /// The Aerox 9's key panel: a dark frame holding twelve numbered keys.
    fn paint_panel(&self, p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook, pal: &Palette) {
        let s = f.s();
        let rect = f.rect_px(self.panel.unwrap_or_default());
        let frame = rounded_poly(rect, 12.0 * s, 5);
        kit::shadow(p, &frame, f.rect, pal);
        kit::fill_flat(p, &frame, sk.black);
        kit::facet(
            p,
            &kit::grown(&frame, -(1.6 * s).max(1.0)),
            vec2(1.0, 0.1),
            sk.deep,
            sk.black,
            sk.hi,
            0.6,
        );
        let font = FontId::proportional(11.0 * s);
        for i in 0..COLS * ROWS {
            let r = f.rect_px(self.cell(i));
            let poly = rounded_poly(r, 4.0 * s, 3);
            let lit = look.body.map(|c| mix(sk.high, c, 0.35));
            key(
                p,
                &poly,
                sk,
                (1.0 * s).max(0.8),
                vec2(0.3, 1.0),
                lit.unwrap_or_else(|| mix(sk.high, sk.hi, 0.2)),
                lit.map_or(sk.high, |c| mix(sk.mid, c, 0.3)),
                0.9,
            );
            if 11.0 * s >= 5.6 {
                p.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    (i + 1).to_string(),
                    font.clone(),
                    with_alpha(sk.hi, 190),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cut-outs are laid out once: a reasonable number, inside the art box, and none over the
    /// wheel well or the DPI key.
    #[test]
    fn perforations_stay_off_the_controls_and_keep_the_triangle_count_down() {
        for (name, a) in [("3", &AEROX_3), ("5", &AEROX_5), ("9", &AEROX_9)] {
            let holes = a.holes();
            assert!(
                (60..260).contains(&holes.len()),
                "Aerox {name}: {} holes",
                holes.len()
            );
            let (w, h) = a.size;
            let px =
                |poly: Poly| -> Vec<Pos2> { poly.iter().map(|q| pos2(q.0 * w, q.1 * h)).collect() };
            let (well, dpi, outline) = (px(a.well), px(a.dpi), px(a.outline));
            for hole in holes {
                assert!(
                    hole.iter()
                        .all(|q| (0.0..=1.0).contains(&q.x) && (0.0..=1.0).contains(&q.y))
                );
                for q in hole {
                    let q = pos2(q.x * w, q.y * h);
                    assert!(
                        !inside(&well, q) && !inside(&dpi, q),
                        "Aerox {name}: a hole over a control"
                    );
                    assert!(
                        inside(&outline, q),
                        "Aerox {name}: a hole outside the shell"
                    );
                }
            }
        }
    }
}
