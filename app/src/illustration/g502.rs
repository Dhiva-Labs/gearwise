//! The G502-style mouse: a faceted, angular shell (split nose with a deep centre channel,
//! flared thumb rest, stacked G7/G8 wedges, wheel with tilt marks, two keys behind it, a
//! three-segment DPI strip and a palm logo). Geometry lives in `g502_shapes`; this module turns
//! it into parts for the callouts and paints the flat-shaded, edge-lit render.

use std::f32::consts::PI;

use egui::epaint::PathStroke;
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
    pos2, vec2,
};

use super::g502_shapes::{self as g, P, n, shapes};
use super::mouse::{Geo, Kind, MouseArt, MouseLook, Part, Role, ZoneRole};
use super::shapes::{edge_lights, fill_poly, glow, glow_line, linear, rot_rrect, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

/// Maps the 0-1 art box and the reference pixel grid onto a screen rect.
struct Frame {
    rect: Rect,
}

impl Frame {
    fn pt(&self, q: P) -> Pos2 {
        pos2(
            self.rect.min.x + q.0 * self.rect.width(),
            self.rect.min.y + q.1 * self.rect.height(),
        )
    }

    /// A point given in reference pixels.
    fn px(&self, q: (f32, f32)) -> Pos2 {
        self.pt(n(q.0, q.1))
    }

    fn pts(&self, q: &[P]) -> Vec<Pos2> {
        q.iter().map(|p| self.pt(*p)).collect()
    }

    fn norm(&self, q: &[Pos2]) -> Vec<Pos2> {
        q.iter().map(|p| self.pt((p.x, p.y))).collect()
    }

    fn rect_px(&self, r: ((f32, f32), (f32, f32))) -> Rect {
        Rect::from_two_pos(self.px(r.0), self.px(r.1))
    }

    /// Screen pixels per reference pixel.
    fn s(&self) -> f32 {
        self.rect.height() / 609.0
    }
}

/// Build the parts (for hit areas, tints and callout anchors) of the G502 shell in `rect`.
pub(super) fn build(rect: Rect, geo: Geo) -> MouseArt {
    let f = Frame { rect };
    let s = f.s();
    let part = |role: Role, poly: Vec<Pos2>, anchor: (f32, f32)| Part {
        role,
        poly,
        anchor: f.px(anchor),
    };
    let around = |c: (f32, f32), w: f32, h: f32| {
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
        Part {
            role: Role::Middle,
            poly: rounded_poly(wheel, 7.0 * s, 4),
            anchor: f.px(g::ANCHOR_MIDDLE),
        },
        part(Role::TiltLeft, around(g::ARROW_L, 20.0, 26.0), g::ARROW_L),
        part(Role::TiltRight, around(g::ARROW_R, 20.0, 26.0), g::ARROW_R),
        part(Role::DpiUp, f.pts(g::G8), g::ANCHOR_G8),
        part(Role::DpiDown, f.pts(g::G7), g::ANCHOR_G7),
        Part {
            role: Role::DpiCycle,
            poly: rounded_poly(f.rect_px(g::CYCLE_KEY), 3.0 * s, 3),
            anchor: f.px(g::ANCHOR_CYCLE),
        },
        Part {
            role: Role::Profile,
            poly: rounded_poly(f.rect_px(g::G9_KEY), 3.0 * s, 3),
            anchor: f.px(g::ANCHOR_G9),
        },
        part(Role::Forward, f.pts(g::G5), g::ANCHOR_G5),
        part(Role::Back, f.pts(g::G4), g::ANCHOR_G4),
        part(Role::Sniper, f.pts(g::SNIPER), g::ANCHOR_SNIPER),
    ];
    let logo = (
        f.px((g::LOGO.0, g::LOGO.1)),
        g::LOGO.2 * rect.width() / 353.0,
    );
    MouseArt {
        rect,
        kind: Kind::G502,
        geo,
        parts,
        wheel,
        logo,
    }
}

/// Black plastic in five steps, plus the edge light.
struct Skin {
    black: Color32,
    deep: Color32,
    low: Color32,
    mid: Color32,
    high: Color32,
    hi: Color32,
}

impl Skin {
    fn new(pal: &Palette) -> Self {
        let c = Color32::from_rgb;
        if pal.dark {
            Self {
                black: c(0x06, 0x08, 0x0b),
                deep: c(0x0f, 0x12, 0x17),
                low: c(0x18, 0x1c, 0x24),
                mid: c(0x25, 0x2a, 0x35),
                high: c(0x38, 0x3f, 0x4d),
                hi: c(0xd6, 0xe2, 0xf4),
            }
        } else {
            Self {
                black: c(0x0d, 0x0f, 0x13),
                deep: c(0x15, 0x18, 0x1e),
                low: c(0x20, 0x24, 0x2c),
                mid: c(0x2e, 0x34, 0x3f),
                high: c(0x44, 0x4b, 0x5a),
                hi: c(0xf4, 0xf7, 0xfc),
            }
        }
    }
}

const LIGHT: Vec2 = vec2(-0.55, -0.83);
const DOWN: Vec2 = vec2(0.28, 0.96);

pub(super) fn paint(
    art: &MouseArt,
    p: &Painter,
    pal: &Palette,
    look: &MouseLook,
    tint: &[(Role, f32)],
) {
    let f = Frame { rect: art.rect };
    let sk = Skin::new(pal);
    let s = f.s();
    let sh = shapes();
    let outline = f.norm(&sh.outline);

    paint_shadow(p, &outline, art.rect, pal);
    paint_cable(p, &f, &sk);
    fill_poly(p, &outline, |_| sk.black);

    // Lower body: thumb-rest lobe, palm cover, right grip wall.
    let lobe = f.norm(&sh.lobe);
    facet(p, &lobe, DOWN, sk.high, sk.deep, &sk, 0.55);
    let palm = f.norm(&sh.palm);
    facet(p, &palm, DOWN, sk.mid, sk.low, &sk, 0.45);
    sheen(p, &f, &sk, pal);
    let wall = f.norm(&sh.wall);
    facet(p, &wall, vec2(1.0, 0.2), sk.low, sk.deep, &sk, 0.4);
    facet(p, &f.pts(g::POCKET), DOWN, sk.black, sk.black, &sk, 0.0);
    facet(p, &f.pts(g::PALM_WEDGE), DOWN, sk.high, sk.mid, &sk, 0.55);

    // Dark seams: the tail groove, swoosh and the gap under the primary buttons.
    facet(
        p,
        &f.pts(g::TAIL_GROOVE),
        DOWN,
        sk.black,
        sk.black,
        &sk,
        0.0,
    );
    facet(p, &f.pts(g::TAIL_STRIP), DOWN, sk.mid, sk.low, &sk, 0.3);
    let swoosh = super::shapes::chaikin(&f.pts(g::SWOOSH), 2);
    p.add(Shape::line(
        swoosh.clone(),
        PathStroke::new(6.5 * s, sk.black),
    ));
    let shifted: Vec<Pos2> = swoosh.iter().map(|q| *q + vec2(2.2 * s, 2.0 * s)).collect();
    p.add(Shape::line(
        shifted,
        PathStroke::new(1.1, with_alpha(sk.hi, 70)),
    ));
    facet(p, &f.pts(g::GROOVE), DOWN, sk.deep, sk.black, &sk, 0.2);

    // Primary buttons, the channel and the wedges beside it.
    facet(p, &f.pts(g::RAIL), DOWN, sk.deep, sk.black, &sk, 0.25);
    facet(p, &f.pts(g::LEFT_BTN), DOWN, sk.high, sk.mid, &sk, 0.95);
    facet(p, &f.pts(g::RIGHT_BTN), DOWN, sk.high, sk.mid, &sk, 0.75);
    facet(p, &f.pts(g::G8), vec2(0.5, 0.86), sk.mid, sk.low, &sk, 0.9);
    facet(p, &f.pts(g::G7), vec2(0.5, 0.86), sk.mid, sk.low, &sk, 0.9);
    facet(
        p,
        &f.pts(g::LED_PANEL),
        vec2(0.3, 0.95),
        sk.mid,
        sk.deep,
        &sk,
        0.8,
    );
    facet(p, &f.pts(g::G5), DOWN, sk.deep, sk.black, &sk, 1.0);
    facet(p, &f.pts(g::G4), DOWN, sk.deep, sk.black, &sk, 1.0);
    facet(
        p,
        &f.pts(g::SNIPER),
        vec2(0.6, 0.8),
        sk.high,
        sk.mid,
        &sk,
        1.0,
    );
    facet_labels(p, &f, &sk);

    paint_wheel(p, &f, &sk, look);
    paint_keys(p, &f, &sk);
    paint_leds(p, &f, &sk, look);
    paint_logo(p, art, &sk, look);

    // A faint rim all round keeps the dark shell readable on a dark card, then a crisp
    // highlight along the lit upper-left of the silhouette.
    p.add(Shape::closed_line(
        outline.clone(),
        PathStroke::new(1.0, with_alpha(sk.hi, 34)),
    ));
    edge_lights(p, &outline, LIGHT, sk.hi, 0.55);

    for (role, strength) in tint {
        if let Some(part) = art.part(*role) {
            p.add(Shape::convex_polygon(
                part.poly.clone(),
                with_alpha(pal.accent, (84.0 * strength) as u8),
                Stroke::NONE,
            ));
            p.add(Shape::closed_line(
                part.poly.clone(),
                PathStroke::new(1.0 + strength, with_alpha(pal.accent, 230)),
            ));
        }
    }
    paint_highlight(p, art, &f, pal, look);
}

fn facet(p: &Painter, pts: &[Pos2], dir: Vec2, from: Color32, to: Color32, sk: &Skin, edge: f32) {
    fill_poly(p, pts, linear(pts, dir, from, to));
    if edge > 0.0 {
        edge_lights(p, pts, LIGHT, sk.hi, edge);
    }
}

fn paint_shadow(p: &Painter, outline: &[Pos2], rect: Rect, pal: &Palette) {
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

/// Cable and its ribbed strain relief, entering the notch between the primary buttons.
fn paint_cable(p: &Painter, f: &Frame, sk: &Skin) {
    let s = f.s();
    let top = f.rect.min.y - g::CABLE * f.rect.height();
    let x = f.px((g::CABLE_X, 0.0)).x;
    let boot_top = f.px((0.0, 97.0)).y;
    let boot_bottom = f.px((0.0, 150.0)).y;
    p.line_segment(
        [pos2(x, top), pos2(x, boot_top + 2.0)],
        Stroke::new(9.0 * s, sk.deep),
    );
    p.line_segment(
        [pos2(x - 1.5 * s, top), pos2(x - 1.5 * s, boot_top)],
        Stroke::new(1.3, with_alpha(sk.hi, 60)),
    );
    let boot = Rect::from_min_max(
        pos2(x - 13.0 * s, boot_top),
        pos2(x + 13.0 * s, boot_bottom),
    );
    p.rect_filled(boot, CornerRadius::same((3.0 * s) as u8), sk.black);
    let ribs = 4;
    for i in 0..ribs {
        let y0 = boot.min.y + boot.height() * i as f32 / ribs as f32;
        let r = Rect::from_min_max(
            pos2(boot.min.x, y0 + 1.0),
            pos2(boot.max.x, y0 + boot.height() / ribs as f32 - 2.0),
        );
        p.rect_filled(r, CornerRadius::same((2.5 * s) as u8), sk.low);
        p.hline(
            r.min.x + 2.0..=r.max.x - 2.0,
            r.min.y + 0.8,
            Stroke::new(1.0, with_alpha(sk.hi, 70)),
        );
    }
}

/// A soft specular bloom on the palm cover.
fn sheen(p: &Painter, f: &Frame, sk: &Skin, pal: &Palette) {
    let c = f.px((478.0, 520.0));
    let radii = vec2(78.0 * f.s(), 86.0 * f.s());
    let ring: Vec<Pos2> = (0..32)
        .map(|i| {
            let a = i as f32 * PI / 16.0;
            c + vec2(a.cos() * radii.x, a.sin() * radii.y)
        })
        .collect();
    let peak = if pal.dark { 40.0 } else { 46.0 };
    p.add(super::shapes::gradient_fan(&ring, |q| {
        let d = ((q - c) / radii).length().min(1.0);
        with_alpha(sk.hi, (peak * (1.0 - d * d)) as u8)
    }));
}

/// "G7", "G8" and "G9" engravings, shown when the render is big enough to read them.
fn facet_labels(p: &Painter, f: &Frame, sk: &Skin) {
    let size = 8.5 * f.s();
    if size < 5.6 {
        return;
    }
    let font = FontId::proportional(size);
    let ink = with_alpha(sk.hi, 150);
    p.text(
        f.px((340.0, 218.0)),
        Align2::CENTER_CENTER,
        "G8",
        font.clone(),
        ink,
    );
    p.text(f.px((342.0, 270.0)), Align2::CENTER_CENTER, "G7", font, ink);
}

fn paint_wheel(p: &Painter, f: &Frame, sk: &Skin, look: &MouseLook) {
    let s = f.s();
    let slot = f.rect_px(g::WHEEL_SLOT);
    let r = CornerRadius::same((9.0 * s) as u8);
    p.rect_filled(slot, r, sk.black);
    p.rect_stroke(
        slot,
        r,
        Stroke::new(1.0, with_alpha(sk.hi, 55)),
        StrokeKind::Inside,
    );
    let w = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        glow(p, w.center(), w.width() * 0.4, with_alpha(c, 60));
    }
    // The tyre: lighter ribs separated by dark gaps, shaded towards the sides.
    let poly = rounded_poly(w, 6.0 * s, 4);
    let cx = w.center().x;
    let half = w.width() / 2.0;
    let mid = look.wheel.map_or(sk.high, |c| mix(sk.high, c, 0.5));
    fill_poly(p, &poly, |q| {
        mix(
            sk.low,
            mid,
            (1.0 - ((q.x - cx).abs() / half).powi(2)).clamp(0.0, 1.0),
        )
    });
    let bands = 9;
    let step = w.height() / bands as f32;
    for i in 0..bands {
        let y = w.min.y + step * i as f32;
        let gap = Rect::from_min_max(
            pos2(w.min.x + 1.0, y + step * 0.62),
            pos2(w.max.x - 1.0, y + step),
        );
        p.rect_filled(gap, CornerRadius::ZERO, with_alpha(sk.black, 190));
        p.hline(
            w.min.x + 2.0..=w.max.x - 2.0,
            y + 0.7,
            Stroke::new(1.0, with_alpha(sk.hi, 80)),
        );
    }
    p.rect_stroke(
        w,
        CornerRadius::same((6.0 * s) as u8),
        Stroke::new(1.0, with_alpha(Color32::BLACK, 150)),
        StrokeKind::Inside,
    );
    // Tilt marks.
    let (aw, ah) = (7.0 * s, 9.0 * s);
    for (c, dir) in [(g::ARROW_L, -1.0), (g::ARROW_R, 1.0)] {
        let c = f.px(c);
        p.add(Shape::convex_polygon(
            vec![
                c + vec2(dir * aw, 0.0),
                c + vec2(-dir * aw * 0.6, -ah),
                c + vec2(-dir * aw * 0.6, ah),
            ],
            with_alpha(sk.hi, 190),
            Stroke::NONE,
        ));
    }
}

/// The two keys behind the wheel: the DPI cycle key and the G9 key.
fn paint_keys(p: &Painter, f: &Frame, sk: &Skin) {
    let s = f.s();
    let r = CornerRadius::same((3.5 * s) as u8);
    let frames = [
        ((455.0, 310.0), (499.0, 347.0)),
        ((457.0, 355.0), (501.0, 402.0)),
    ];
    for fr in frames {
        p.rect_filled(f.rect_px(fr), r, sk.black);
    }
    for key in [g::CYCLE_KEY, g::G9_KEY] {
        let rect = f.rect_px(key);
        let poly = rounded_poly(rect, 3.0 * s, 3);
        fill_poly(p, &poly, linear(&poly, vec2(0.0, 1.0), sk.high, sk.mid));
        edge_lights(p, &poly, LIGHT, sk.hi, 1.0);
        p.rect_stroke(
            rect,
            r,
            Stroke::new(1.0, with_alpha(sk.hi, 70)),
            StrokeKind::Inside,
        );
    }
    // Cycle icon: an open ring with an arrow head.
    let c = f.rect_px(g::CYCLE_KEY).center();
    let rad = 6.0 * s;
    let ring: Vec<Pos2> = (0..=18)
        .map(|i| {
            let a = (40.0 + 270.0 * i as f32 / 18.0).to_radians();
            c + vec2(a.cos(), a.sin()) * rad
        })
        .collect();
    p.add(Shape::line(
        ring,
        PathStroke::new(1.2, with_alpha(sk.hi, 200)),
    ));
    let tip = c + vec2(40f32.to_radians().cos(), 40f32.to_radians().sin()) * rad;
    p.add(Shape::convex_polygon(
        vec![
            tip + vec2(2.8 * s, -1.6 * s),
            tip + vec2(0.2 * s, 3.2 * s),
            tip + vec2(-3.0 * s, -1.6 * s),
        ],
        with_alpha(sk.hi, 200),
        Stroke::NONE,
    ));
    let size = 8.5 * s;
    if size >= 5.6 {
        p.text(
            f.rect_px(g::G9_KEY).center(),
            Align2::CENTER_CENTER,
            "G9",
            FontId::proportional(size),
            with_alpha(sk.hi, 190),
        );
    }
}

/// The DPI strip: three slanted segments on the left flank.
fn paint_leds(p: &Painter, f: &Frame, sk: &Skin, look: &MouseLook) {
    let s = f.s();
    for c in g::LEDS {
        let c = f.pt(c);
        let pts = rot_rrect(c, 5.2 * s, 13.0 * s, -0.42, 1.6 * s);
        match look.body {
            Some(col) => {
                glow(p, c, 3.2 * s, with_alpha(col, 70));
                p.add(Shape::convex_polygon(
                    pts,
                    mix(col, Color32::WHITE, 0.2),
                    Stroke::NONE,
                ));
            }
            None => {
                p.add(Shape::convex_polygon(
                    pts,
                    mix(sk.deep, sk.hi, 0.12),
                    Stroke::new(0.8, with_alpha(sk.hi, 40)),
                ));
            }
        }
    }
}

/// The palm mark: an open ring and a dot, glowing when lit.
fn logo_arc(c: Pos2, r: f32) -> Vec<Pos2> {
    (0..=24)
        .map(|i| {
            let a = (14.0 + 284.0 * i as f32 / 24.0).to_radians();
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

fn paint_logo(p: &Painter, art: &MouseArt, sk: &Skin, look: &MouseLook) {
    let (c, r) = art.logo;
    let ring = logo_arc(c, r * 0.78);
    let width = r * 0.34;
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
            p.add(Shape::line(
                ring,
                PathStroke::new(width, mix(sk.low, sk.hi, 0.2)),
            ));
            p.circle_filled(c, r * 0.16, mix(sk.low, sk.hi, 0.2));
        }
    }
}

fn paint_highlight(p: &Painter, art: &MouseArt, f: &Frame, pal: &Palette, look: &MouseLook) {
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
        ZoneRole::Body => {
            // The DPI strip: outline the panel that carries it.
            let panel = f.pts(g::LED_PANEL);
            let c = super::shapes::centroid(&panel);
            let grown: Vec<Pos2> = panel
                .iter()
                .map(|q| *q + (*q - c).normalized() * 6.0)
                .collect();
            p.extend(Shape::dashed_line(
                &[grown.as_slice(), &grown[..1]].concat(),
                ring,
                5.0,
                4.0,
            ));
        }
    }
}
