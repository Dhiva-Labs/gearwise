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

use super::g502_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, DOWN, Frame, LIGHT, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role, Side};
use crate::illustration::shapes::{
    edge_lights, fill_poly, glow, glow_line, linear, rot_rrect, rounded_poly,
};
use crate::theme::{Palette, mix, with_alpha};

/// The G502 HERO family: HERO, Proteus and Lightspeed share the shell (the X models, a newer
/// shell, have their own drawing).
pub struct G502;

impl ModelArt for G502 {
    fn id(&self) -> &'static str {
        "g502"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["g502"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "G502 HERO Gaming Mouse",
            "G502 LIGHTSPEED",
            "G502 Proteus Spectrum",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
    }

    fn overhang(&self) -> f32 {
        g::CABLE
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

/// Build the parts (for hit areas, tints and callout anchors) of the G502 shell in `rect`.
fn build(rect: Rect) -> Anatomy {
    let f = frame(rect);
    let s = f.s();
    let part = |role: Role, poly: Vec<Pos2>, anchor: (f32, f32)| {
        Part::new(role, poly).anchored(f.px(anchor))
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
        part(
            Role::Middle,
            rounded_poly(wheel, 7.0 * s, 4),
            g::ANCHOR_MIDDLE,
        ),
        part(Role::TiltLeft, around(g::ARROW_L, 20.0, 26.0), g::ARROW_L),
        part(Role::TiltRight, around(g::ARROW_R, 20.0, 26.0), g::ARROW_R),
        // G7 / G8 sit left of the left button, so their labels go in the left margin.
        part(Role::DpiUp, f.pts(g::G8), g::ANCHOR_G8).labelled(Side::Left),
        part(Role::DpiDown, f.pts(g::G7), g::ANCHOR_G7).labelled(Side::Left),
        part(
            Role::DpiCycle,
            rounded_poly(f.rect_px(g::CYCLE_KEY), 3.0 * s, 3),
            g::ANCHOR_CYCLE,
        ),
        part(
            Role::Profile,
            rounded_poly(f.rect_px(g::G9_KEY), 3.0 * s, 3),
            g::ANCHOR_G9,
        ),
        part(Role::Forward, f.pts(g::G5), g::ANCHOR_G5),
        part(Role::Back, f.pts(g::G4), g::ANCHOR_G4),
        part(Role::Sniper, f.pts(g::SNIPER), g::ANCHOR_SNIPER),
    ];
    Anatomy {
        parts,
        wheel,
        logo: (
            f.px((g::LOGO.0, g::LOGO.1)),
            g::LOGO.2 * rect.width() / 353.0,
        ),
    }
}

fn paint(art: &MouseArt, p: &Painter, pal: &Palette, look: &MouseLook, tint: &[(Role, f32)]) {
    let f = frame(art.rect);
    let sk = Plastic::black(pal);
    let s = f.s();
    let sh = shapes();
    let outline = f.norm(&sh.outline);

    kit::shadow(p, &outline, art.rect, pal);
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
    let swoosh = crate::illustration::shapes::chaikin(&f.pts(g::SWOOSH), 2);
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

    kit::paint_tint(p, art, pal, tint);
    paint_highlight(p, art, &f, pal, look);
}

fn facet(
    p: &Painter,
    pts: &[Pos2],
    dir: Vec2,
    from: Color32,
    to: Color32,
    sk: &Plastic,
    edge: f32,
) {
    kit::facet(p, pts, dir, from, to, sk.hi, edge);
}

/// Cable and its ribbed strain relief, entering the notch between the primary buttons.
fn paint_cable(p: &Painter, f: &Frame, sk: &Plastic) {
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
fn sheen(p: &Painter, f: &Frame, sk: &Plastic, pal: &Palette) {
    let c = f.px((478.0, 520.0));
    let radii = vec2(78.0 * f.s(), 86.0 * f.s());
    let ring: Vec<Pos2> = (0..32)
        .map(|i| {
            let a = i as f32 * PI / 16.0;
            c + vec2(a.cos() * radii.x, a.sin() * radii.y)
        })
        .collect();
    let peak = if pal.dark { 40.0 } else { 46.0 };
    p.add(crate::illustration::shapes::gradient_fan(&ring, |q| {
        let d = ((q - c) / radii).length().min(1.0);
        with_alpha(sk.hi, (peak * (1.0 - d * d)) as u8)
    }));
}

/// "G7", "G8" and "G9" engravings, shown when the render is big enough to read them.
fn facet_labels(p: &Painter, f: &Frame, sk: &Plastic) {
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

fn paint_wheel(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
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
fn paint_keys(p: &Painter, f: &Frame, sk: &Plastic) {
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
fn paint_leds(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
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

fn paint_logo(p: &Painter, art: &MouseArt, sk: &Plastic, look: &MouseLook) {
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
    kit::zone_ring(p, art, pal, look, |p, ring| {
        // The DPI strip: outline the panel that carries it.
        let outline = kit::grown(&f.pts(g::LED_PANEL), 6.0);
        p.extend(Shape::dashed_line(
            &[outline.as_slice(), &outline[..1]].concat(),
            ring,
            5.0,
            4.0,
        ));
    });
}
