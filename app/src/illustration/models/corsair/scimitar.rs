//! The Scimitar family (RGB Elite, PRO RGB, Elite Wireless): a tall MMO shell with two front
//! buttons either side of a slot that holds the wheel between two light rails, a pair of DPI
//! keys behind it, a rubber grip on the right and, on the thumb side, the sliding panel of twelve
//! numbered keys. Seen from above that panel is drawn flat against the left flank, three columns
//! by four rows, numbered row by row. The logo sits on the palm cover below a long diagonal
//! seam. Geometry lives in `scimitar_shapes`.

use egui::epaint::PathStroke;
use egui::{Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, vec2};

use super::common::{
    self, chevron, groove, hex_mark, key, lattice, light_line, rim, rubber_wheel, sheen,
};
use super::scimitar_shapes::{self as g};
use crate::illustration::models::kit::{self, DOWN, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.1;
const COLS: usize = 3;
const ROWS: usize = 4;
/// Padding inside the key panel and the gap between keys, in reference px.
const PAD: f32 = 7.0;
const GAP: f32 = 3.5;

pub struct Scimitar;

impl ModelArt for Scimitar {
    fn id(&self) -> &'static str {
        "scimitar"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["scimitar"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Corsair Scimitar RGB Elite",
            "Corsair Scimitar PRO RGB",
            "Corsair Scimitar RGB Elite (demo)",
            "Corsair Scimitar Elite Wireless SE",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn overhang(&self) -> f32 {
        CABLE
    }

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
        let s = f.s();
        let part = |role, poly: Vec<Pos2>, anchor| Part::new(role, poly).anchored(f.px(anchor));
        let wheel = f.rect_px(g::WHEEL);
        let mut parts = vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 6.0 * s, 4),
                (220.0, 138.0),
            ),
            part(Role::DpiUp, f.pts(g::DPI_UP), (218.0, 280.0)),
            part(Role::DpiDown, f.pts(g::DPI_DOWN), (218.0, 356.0)),
        ];
        for i in 0..COLS * ROWS {
            let cell = cell(i);
            let poly = rounded_poly(f.rect_px(cell), 4.0 * s, 3);
            let c = ((cell.0.0 + cell.1.0) / 2.0, (cell.0.1 + cell.1.1) / 2.0);
            parts.push(part(Role::Grid(i as u8), poly, c).labelled(Side::Left));
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
        let s = f.s();
        let sk = Plastic::black(pal);
        let outline = f.pts(g::OUTLINE);
        let panel = rounded_poly(f.rect_px(g::PANEL), 12.0 * s, 5);

        kit::shadow(p, &outline, art.rect, pal);
        kit::shadow(p, &panel, art.rect, pal);
        let x = f.px((212.0, 0.0)).x;
        common::cable(
            p,
            x,
            art.rect.min.y - CABLE * art.rect.height(),
            f.px((0.0, 54.0)).y,
            (8.0 * s).max(4.5),
            &sk,
        );

        paint_panel(p, &f, &sk, look, &panel);

        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((230.0, 480.0)),
            vec2(150.0, 190.0) * s,
            &sk,
            pal,
            30.0,
        );
        let grip = f.pts(g::RIGHT_GRIP);
        kit::facet(p, &grip, vec2(-1.0, 0.2), sk.mid, sk.low, sk.hi, 0.6);
        lattice(p, &grip, 13.0 * s, with_alpha(sk.hi, 38));
        groove(p, &f.pts(g::SEAM), 2.2 * s, &sk);

        paint_slot(p, &f, &sk, look);
        hex_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let mut poly = kit::grown(&panel, 5.0);
            poly.push(poly[0]);
            p.extend(Shape::dashed_line(&poly, ring, 5.0, 4.0));
        });
    }
}

/// Reference-px rect of key `i` (row by row, front to back) in the side panel.
fn cell(i: usize) -> ((f32, f32), (f32, f32)) {
    let (p0, p1) = g::PANEL;
    let w = (p1.0 - p0.0 - 2.0 * PAD - (COLS - 1) as f32 * GAP) / COLS as f32;
    let h = (p1.1 - p0.1 - 2.0 * PAD - (ROWS - 1) as f32 * GAP) / ROWS as f32;
    let (r, c) = ((i / COLS) as f32, (i % COLS) as f32);
    let x0 = p0.0 + PAD + c * (w + GAP);
    let y0 = p0.1 + PAD + r * (h + GAP);
    ((x0, y0), (x0 + w, y0 + h))
}

/// The side panel: a dark frame lit from behind by the thumb zone, holding twelve grey keys.
fn paint_panel(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook, panel: &[Pos2]) {
    let s = f.s();
    kit::fill_flat(p, panel, sk.black);
    kit::facet(
        p,
        &kit::grown(panel, -(1.6 * s).max(1.0)),
        vec2(1.0, 0.1),
        sk.deep,
        sk.black,
        sk.hi,
        0.6,
    );
    let rect = f.rect_px(g::PANEL);
    if let Some(c) = look.body {
        // The backlight shows between the keys and round the frame.
        glow(
            p,
            rect.center() + vec2(-rect.width() * 0.3, 0.0),
            rect.height() * 0.2,
            with_alpha(c, 26),
        );
        let edge = [
            rect.left_top() + vec2(4.0 * s, 10.0 * s),
            rect.left_bottom() + vec2(4.0 * s, -10.0 * s),
        ];
        light_line(p, &edge, 2.6 * s, c);
    }
    let font = FontId::proportional(9.5 * s);
    let amber = Color32::from_rgb(0xe0, 0xc4, 0x3c);
    for i in 0..COLS * ROWS {
        let r = f.rect_px(cell(i));
        let poly = rounded_poly(r, 4.0 * s, 3);
        let glow_c = look.body.map(|c| mix(sk.high, c, 0.35));
        key(
            p,
            &poly,
            sk,
            (1.0 * s).max(0.8),
            vec2(0.3, 1.0),
            glow_c.unwrap_or_else(|| mix(sk.high, sk.hi, 0.32)),
            glow_c.map_or(mix(sk.mid, sk.high, 0.5), |c| mix(sk.mid, c, 0.3)),
            0.9,
        );
        if 9.5 * s >= 5.6 {
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                (i + 1).to_string(),
                font.clone(),
                with_alpha(amber, 230),
            );
        }
    }
}

/// The wheel slot with its light rails, the wheel and the two DPI keys behind it.
fn paint_slot(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let slot = f.pts(g::SLOT);
    kit::fill_flat(p, &slot, sk.black);
    let wheel = f.rect_px(g::WHEEL);
    let rail = |x0: f32, x1: f32| f.rect_px(((x0, 58.0), (x1, 222.0)));
    for r in [rail(190.0, 199.0), rail(240.0, 249.0)] {
        match look.wheel {
            Some(c) => {
                glow(p, r.center(), r.width() * 0.6, with_alpha(c, 30));
                p.rect_filled(r, CornerRadius::same(2), mix(c, Color32::WHITE, 0.4));
            }
            None => {
                p.rect_filled(r, CornerRadius::same(2), mix(sk.low, sk.hi, 0.5));
            }
        }
    }
    rubber_wheel(p, wheel, sk, None, s);

    for (poly, up) in [(g::DPI_UP, true), (g::DPI_DOWN, false)] {
        let pts = f.pts(poly);
        key(
            p,
            &pts,
            sk,
            (1.4 * s).max(1.0),
            DOWN,
            sk.deep,
            sk.black,
            0.9,
        );
        let c = f.px((218.0, if up { 281.0 } else { 358.0 }));
        chevron(p, c, 8.0 * s, up, with_alpha(sk.hi, 140));
    }
    let mut frame = f.pts(g::SLOT);
    frame.push(frame[0]);
    p.add(Shape::line(
        frame,
        PathStroke::new(1.0, with_alpha(sk.hi, 40)),
    ));
}
