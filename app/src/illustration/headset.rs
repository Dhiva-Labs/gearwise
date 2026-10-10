//! A headset drawn from a headband, two cups and a boom mic.

use egui::epaint::PathStroke;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use super::shapes::{bezier, glow, rounded_poly, vertical_fan};
use super::MouseLook;
use crate::theme::{Palette, mix, with_alpha};

pub fn paint_headset(p: &Painter, area: Rect, pal: &Palette, look: &MouseLook) {
    let s = area.width().min(area.height() * 1.15);
    let c = area.center();
    let at = |x: f32, y: f32| pos2(c.x + x * s, c.y + y * s);
    let led = look.logo.or(look.body);
    let band_dark = mix(pal.raised, Color32::BLACK, 0.45);
    let (cup_top, cup_bottom) = if pal.dark { (mix(pal.raised, pal.text, 0.14), mix(pal.raised, pal.window, 0.3)) } else { (Color32::WHITE, mix(pal.raised, pal.border, 0.9)) };

    // Headband: three layered strokes give it a rounded, lit look.
    let band: Vec<Pos2> = bezier(at(-0.32, 0.12), at(-0.32, -0.46), at(0.32, -0.46), at(0.32, 0.12), 40);
    p.add(Shape::line(band.clone(), PathStroke::new(s * 0.075, band_dark)));
    p.add(Shape::line(band.clone(), PathStroke::new(s * 0.052, mix(cup_top, cup_bottom, 0.5))));
    let hi: Vec<Pos2> = band.iter().map(|q| *q + vec2(0.0, -s * 0.012)).collect();
    p.add(Shape::line(hi, PathStroke::new(s * 0.014, with_alpha(Color32::WHITE, if pal.dark { 60 } else { 140 }))));

    for side in [-1.0f32, 1.0] {
        let cup = Rect::from_center_size(at(side * 0.33, 0.2), vec2(s * 0.2, s * 0.34));
        for i in 1..=4 {
            p.rect_filled(cup.translate(vec2(0.0, 5.0 + i as f32 * 2.0)).expand(i as f32 * 2.0), CornerRadius::same(22), Color32::from_black_alpha(if pal.dark { 14 } else { 6 }));
        }
        p.add(Shape::convex_polygon(rounded_poly(cup.expand(2.0), s * 0.07, 6), band_dark, Stroke::NONE));
        p.add(vertical_fan(&rounded_poly(cup, s * 0.065, 6), cup_top, cup_bottom));
        let disc = cup.center() + vec2(-side * s * 0.015, 0.0);
        match led {
            Some(col) => {
                glow(p, disc, s * 0.028, mix(col, Color32::WHITE, 0.25));
                p.circle_stroke(disc, s * 0.075, Stroke::new(1.6, with_alpha(col, 230)));
            }
            None => {
                p.circle_stroke(disc, s * 0.075, Stroke::new(1.0, with_alpha(pal.text, 60)));
                p.circle_stroke(disc, s * 0.03, Stroke::new(1.0, with_alpha(pal.text, 40)));
            }
        }
    }

    // Boom mic from the left cup.
    let boom = bezier(at(-0.36, 0.34), at(-0.38, 0.5), at(-0.26, 0.58), at(-0.12, 0.55), 20);
    p.add(Shape::line(boom.clone(), PathStroke::new(s * 0.024, band_dark)));
    let tip = *boom.last().unwrap_or(&at(-0.12, 0.55));
    p.rect_filled(Rect::from_center_size(tip, vec2(s * 0.07, s * 0.035)), CornerRadius::same(6), mix(cup_top, cup_bottom, 0.4));
}
