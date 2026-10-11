//! The M65 family (RGB Ultra, Elite, PRO): an angular FPS shell with a flat, faceted top split
//! into two big wedge buttons over a V-shaped seam, a rubber wheel with light rails, a glossy DPI
//! rocker behind it, a textured thumb flange on the left with two side keys and the red sniper
//! key, and a grip panel on the right. The logo sits on the palm and the DPI indicator is the
//! bar between the two DPI keys. Geometry lives in `m65_shapes`.

use egui::epaint::PathStroke;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::common::{self, chevron, groove, hex_mark, key, rim, rubber_wheel, sheen};
use super::m65_shapes::{self as g};
use crate::illustration::models::kit::{self, DOWN, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// How far the cable reaches above the shell, as a fraction of its height.
const CABLE: f32 = 0.11;

pub struct M65;

impl ModelArt for M65 {
    fn id(&self) -> &'static str {
        "m65"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["m65"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Corsair M65 RGB Ultra",
            "Corsair M65 RGB Ultra (demo)",
            "Corsair M65 RGB Ultra Wireless",
            "Corsair M65 PRO RGB",
            "Corsair M65 RGB Elite",
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
        Anatomy {
            parts: vec![
                part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
                part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
                part(
                    Role::Middle,
                    rounded_poly(wheel, 4.0 * s, 3),
                    (243.0, 133.0),
                ),
                part(Role::DpiUp, f.pts(g::DPI_UP), (244.0, 228.0)),
                part(Role::DpiDown, f.pts(g::DPI_DOWN), (245.0, 300.0)),
                part(Role::Forward, f.pts(g::FORWARD), (44.0, 280.0)),
                part(Role::Back, f.pts(g::BACK), (66.0, 396.0)),
                part(Role::Sniper, f.pts(g::SNIPER), (30.0, 280.0)),
            ],
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
        let sk = Plastic::graphite(pal);
        let outline = f.pts(g::OUTLINE);

        kit::shadow(p, &outline, art.rect, pal);
        // The notch the cable runs through, then the cable.
        let x = f.px((247.0, 0.0)).x;
        common::cable(
            p,
            x,
            art.rect.min.y - CABLE * art.rect.height(),
            f.px((0.0, 70.0)).y,
            9.0 * s,
            &sk,
        );

        kit::facet(p, &outline, DOWN, sk.mid, sk.low, sk.hi, 0.0);
        sheen(
            p,
            f.px((240.0, 470.0)),
            vec2(150.0, 170.0) * s,
            &sk,
            pal,
            34.0,
        );

        paint_flanks(p, &f, &sk);
        paint_top(p, &f, &sk);
        paint_wheel(p, &f, &sk, look);
        paint_dpi(p, &f, &sk, look);
        paint_sniper(p, &f);
        hex_mark(p, art.logo.0, art.logo.1, look.logo, &sk);

        rim(p, &outline, &sk);
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| {
            let r = Rect::from_min_max(f.px((206.0, 196.0)), f.px((284.0, 378.0))).expand(7.0);
            let poly = rounded_poly(r, 12.0, 4);
            p.extend(Shape::dashed_line(
                &[poly.as_slice(), &poly[..1]].concat(),
                ring,
                5.0,
                4.0,
            ));
        });
    }
}

/// The thumb flange with its side keys, and the grip panel on the right.
fn paint_flanks(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    let left = f.pts(g::LEFT_PANEL);
    kit::facet(p, &left, vec2(1.0, 0.35), sk.deep, sk.mid, sk.hi, 0.5);
    texture(p, &left, 15.0 * s, sk);
    let right = f.pts(g::RIGHT_PANEL);
    kit::facet(p, &right, vec2(-1.0, 0.1), sk.high, sk.low, sk.hi, 0.7);
    texture(p, &right, 15.0 * s, sk);
    // The bright line where the flange's inner edge catches the light.
    let edge = f.pts(g::EDGE_L);
    p.add(Shape::line(
        edge,
        PathStroke::new((2.2 * s).max(1.2), with_alpha(Color32::WHITE, 200)),
    ));
    for poly in [g::FORWARD, g::BACK] {
        let pts = f.pts(poly);
        key(
            p,
            &pts,
            sk,
            (1.6 * s).max(1.0),
            vec2(0.9, 0.4),
            mix(sk.high, sk.hi, 0.5),
            sk.high,
            0.9,
        );
    }
}

/// Small embossed triangles over a polygon, as on the grip panels. Only drawn when big enough
/// to read.
fn texture(p: &Painter, poly: &[Pos2], pitch: f32, sk: &Plastic) {
    if pitch < 6.0 {
        return;
    }
    let bounds = Rect::from_points(poly);
    let ink = with_alpha(sk.hi, 38);
    let r = pitch * 0.2;
    let mut y = bounds.min.y + pitch;
    let mut odd = false;
    while y < bounds.max.y - pitch * 0.5 {
        let mut x = bounds.min.x + pitch * if odd { 1.0 } else { 0.5 };
        while x < bounds.max.x {
            let c = pos2(x, y);
            if common::inside(poly, c + vec2(-r, -r)) && common::inside(poly, c + vec2(r, r)) {
                p.add(Shape::convex_polygon(
                    vec![c + vec2(0.0, -r), c + vec2(r, r), c + vec2(-r, r)],
                    ink,
                    Stroke::NONE,
                ));
            }
            x += pitch;
        }
        y += pitch * 0.9;
        odd = !odd;
    }
}

/// The two wedge buttons over their V-shaped seam.
fn paint_top(p: &Painter, f: &Frame, sk: &Plastic) {
    let s = f.s();
    for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
        let pts = f.pts(poly);
        kit::facet(
            p,
            &pts,
            vec2(0.35, 0.95),
            mix(sk.high, sk.mid, 0.35),
            sk.mid,
            sk.hi,
            0.8,
        );
    }
    groove(p, &f.pts(g::SEAM_L), 2.2 * s, sk);
    groove(p, &f.pts(g::SEAM_R), 2.2 * s, sk);
    // The gap between the right button and its grip panel.
    let gap = f.pts(
        &[
            (410.0, 84.0),
            (418.0, 150.0),
            (424.0, 200.0),
            (428.0, 260.0),
            (431.0, 330.0),
        ]
        .map(|(x, y)| g::n(x, y)),
    );
    groove(p, &gap, 2.0 * s, sk);
}

/// The wheel in its well, with light rails and the two side tabs.
fn paint_wheel(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    kit::fill_flat(p, &f.pts(g::WELL), sk.black);
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        glow(p, wheel.center(), wheel.width() * 0.5, with_alpha(c, 40));
    }
    rubber_wheel(p, wheel, sk, look.wheel, s);
    for tab in [g::TAB_L, g::TAB_R] {
        let r = f.rect_px(tab);
        match look.wheel {
            Some(c) => {
                glow(p, r.center(), r.height() * 0.3, with_alpha(c, 70));
                p.rect_filled(r, CornerRadius::same(1), mix(c, Color32::WHITE, 0.25));
            }
            None => {
                p.rect_filled(r, CornerRadius::same(1), mix(sk.low, sk.hi, 0.18));
            }
        }
    }
}

/// The glossy DPI rocker: two black keys with white edge lines either side of the silver bar
/// that doubles as the DPI indicator.
fn paint_dpi(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    for (poly, up) in [(g::DPI_UP, true), (g::DPI_DOWN, false)] {
        let pts = f.pts(poly);
        kit::fill_flat(p, &pts, sk.black);
        let face = kit::grown(&pts, -(1.6 * s).max(1.0));
        kit::facet(p, &face, DOWN, sk.deep, sk.black, sk.hi, 0.0);
        let mut line = kit::grown(&pts, -(2.6 * s).max(1.4));
        line.push(line[0]);
        p.add(Shape::line(
            line,
            PathStroke::new(1.0, with_alpha(Color32::WHITE, 120)),
        ));
        let c = f.px(if up { (244.0, 232.0) } else { (245.0, 290.0) });
        chevron(p, c, 8.0 * s, up, with_alpha(sk.hi, 200));
    }
    let bar = f.rect_px(g::BAR);
    let (a, b) = match look.body {
        Some(c) => {
            glow(p, bar.center(), bar.width() * 0.25, with_alpha(c, 60));
            (mix(c, Color32::WHITE, 0.55), mix(c, Color32::WHITE, 0.1))
        }
        None => (
            Color32::from_rgb(0xf2, 0xf4, 0xf8),
            Color32::from_rgb(0x8d, 0x93, 0x9c),
        ),
    };
    kit::shaded_rect(p, bar, 2, 4, |q| {
        mix(a, b, ((q.y - bar.min.y) / bar.height()).clamp(0.0, 1.0))
    });
    p.rect_stroke(
        bar,
        CornerRadius::same(1),
        Stroke::new(1.0, with_alpha(Color32::BLACK, 120)),
        egui::StrokeKind::Inside,
    );
}

/// The red sniper key on the flange, with a crosshair when the render is big enough.
fn paint_sniper(p: &Painter, f: &Frame) {
    let s = f.s();
    let pts = f.pts(g::SNIPER);
    kit::fill_flat(p, &pts, Color32::from_rgb(0x2a, 0x08, 0x0a));
    let face = kit::grown(&pts, -(1.4 * s).max(1.0));
    kit::facet(
        p,
        &face,
        vec2(0.2, 1.0),
        Color32::from_rgb(0xe0, 0x32, 0x34),
        Color32::from_rgb(0x8e, 0x12, 0x16),
        Color32::from_rgb(0xff, 0xb0, 0xb0),
        0.8,
    );
    let c = f.px((30.0, 280.0));
    let r = 5.0 * s;
    if r >= 2.5 {
        let ink = with_alpha(Color32::WHITE, 210);
        p.circle_stroke(c, r, Stroke::new(1.0, ink));
        p.line_segment(
            [c - vec2(r * 1.5, 0.0), c + vec2(r * 1.5, 0.0)],
            Stroke::new(1.0, ink),
        );
        p.line_segment(
            [c - vec2(0.0, r * 1.5), c + vec2(0.0, r * 1.5)],
            Stroke::new(1.0, ink),
        );
    }
}
