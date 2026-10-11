//! The Cobra and Cobra Pro: a compact right-handed shell with two broad primary buttons that
//! end in a V-shaped trench above the palm cover, a spine down the middle that carries the
//! wheel (with a Chroma strip either side) and a pill-shaped DPI key behind it, thumb buttons
//! on the left flank, rubber grips on the lower flanks, and an underglow strip round the rear.
//! Geometry lives in `cobra_pro_shapes`.

use egui::epaint::PathStroke;
use egui::{CornerRadius, Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::cobra_pro_shapes::{self as g, shapes};
use super::shared;
use crate::illustration::models::kit::{self, Frame, P, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::illustration::shapes::{fill_poly, glow, rounded_poly};
use crate::theme::{Palette, mix, with_alpha};

pub struct CobraPro;

impl ModelArt for CobraPro {
    fn id(&self) -> &'static str {
        "razer-cobra-pro"
    }

    fn matches(&self, name: &str) -> bool {
        contains_any(name, &["cobra"])
    }

    fn examples(&self) -> &'static [&'static str] {
        &[
            "Razer Cobra",
            "Razer Cobra Pro (Wired)",
            "Razer Cobra Pro (Wireless)",
            "Cobra Pro (demo)",
        ]
    }

    fn aspect(&self) -> f32 {
        g::ASPECT
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

fn build(rect: Rect) -> Anatomy {
    let f = frame(rect);
    let s = f.s();
    let part =
        |role: Role, poly: Vec<Pos2>, anchor: P| Part::new(role, poly).anchored(f.px(anchor));
    let wheel = f.rect_px(g::WHEEL);
    Anatomy {
        parts: vec![
            part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
            part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
            part(
                Role::Middle,
                rounded_poly(wheel, 4.0 * s, 3),
                g::ANCHOR_MIDDLE,
            ),
            part(
                Role::DpiCycle,
                rounded_poly(f.rect_px(g::PILL), 5.0 * s, 4),
                g::ANCHOR_PILL,
            ),
            part(Role::Forward, f.pts(g::FORWARD), g::ANCHOR_FORWARD),
            part(Role::Back, f.pts(g::BACK), g::ANCHOR_BACK),
        ],
        wheel,
        logo: (f.px(g::LOGO), g::LOGO_RADIUS * s),
    }
}

fn paint(art: &MouseArt, p: &Painter, pal: &Palette, look: &MouseLook, tint: &[(Role, f32)]) {
    let f = frame(art.rect);
    let s = f.s();
    let sk = Plastic::black(pal);
    let sh = shapes();
    let outline = f.norm(&sh.outline);
    let under = f.norm(&sh.underglow);

    kit::shadow(p, &outline, art.rect, pal);
    // The underglow strip sits under the shell's rim, so only its light shows beyond it.
    shared::underglow(p, &under, 4.0 * s, &sk, look.body);
    kit::facet(p, &outline, vec2(0.1, 1.0), sk.mid, sk.low, sk.hi, 0.0);
    shared::sheen(p, f.px((225.0, 470.0)), vec2(76.0 * s, 100.0 * s), &sk, pal);

    // Rubber grips on the lower flanks.
    shared::grip(p, &f.pts(g::LEFT_GRIP), &sk, s, vec2(1.0, 0.3));
    shared::grip(p, &f.pts(g::RIGHT_GRIP), &sk, s, vec2(-1.0, 0.3));

    // Primary buttons, the V-shaped trench under them and the palm cover's lit front edge.
    let gap = (1.3 * s).max(1.0);
    let face = mix(sk.mid, sk.high, 0.55);
    for poly in [g::LEFT_BTN, g::RIGHT_BTN] {
        shared::button(p, &f.pts(poly), gap, face, sk.mid, &sk, 0.8);
    }
    fill_poly(p, &f.pts(g::BAND), |_| sk.black);
    p.add(Shape::line(
        f.pts(g::BEVEL),
        PathStroke::new(1.3, with_alpha(sk.hi, 64)),
    ));

    paint_spine(p, &f, &sk, look);

    for poly in [g::FORWARD, g::BACK] {
        shared::button(p, &f.pts(poly), (1.0 * s).max(1.0), face, sk.mid, &sk, 0.9);
    }

    shared::volume(p, &outline, &sk, pal);
    shared::logo(p, art, &sk, look);
    shared::rim(p, &outline, &sk);

    kit::paint_tint(p, art, pal, tint);
    kit::zone_ring(p, art, pal, look, |p, ring| {
        shared::dashed_run(p, &kit::grown(&under, 6.0), ring);
    });
}

/// The spine between the primary buttons: lighter plastic holding the wheel window and the pill.
fn paint_spine(p: &Painter, f: &Frame, sk: &Plastic, look: &MouseLook) {
    let s = f.s();
    let well = f.rect_px(g::CHANNEL);
    p.rect_filled(well, CornerRadius::same((1.5 * s) as u8), sk.black);
    let spine = Rect::from_min_max(well.min + vec2(1.4 * s, 0.0), well.max - vec2(1.4 * s, 0.0));
    let poly = rounded_poly(spine, 1.5 * s, 2);
    kit::facet(p, &poly, kit::DOWN, sk.mid, sk.low, sk.hi, 0.5);

    let win = f.rect_px(g::WHEEL_SLOT);
    shared::window(p, win, 3.0 * s, sk);
    let wheel = f.rect_px(g::WHEEL);
    if let Some(c) = look.wheel {
        glow(p, wheel.center(), wheel.width() * 0.5, with_alpha(c, 50));
    }
    shared::wheel(p, wheel, sk, look.wheel, s, 0.35);
    // The Chroma strips, between the wheel and the window.
    for x in [win.min.x + 3.0 * s, win.max.x - 3.0 * s] {
        let (a, b) = (
            Pos2::new(x, wheel.min.y + 1.0),
            Pos2::new(x, wheel.max.y - 1.0),
        );
        match look.wheel {
            Some(c) => shared::led_bar(p, a, b, 2.0 * s, c),
            None => {
                p.line_segment([a, b], Stroke::new(2.0 * s, mix(sk.low, sk.hi, 0.2)));
            }
        }
    }

    // The DPI key: a pill split into three by two fine grooves.
    let pill = rounded_poly(f.rect_px(g::PILL), 5.0 * s, 4);
    shared::button(p, &pill, (1.2 * s).max(1.0), sk.high, sk.mid, sk, 0.9);
    let r = f.rect_px(g::PILL);
    for t in [0.42, 0.56] {
        let y = r.min.y + r.height() * t;
        p.hline(
            r.min.x + 2.0..=r.max.x - 2.0,
            y,
            Stroke::new(1.0, with_alpha(sk.black, 200)),
        );
    }
}
