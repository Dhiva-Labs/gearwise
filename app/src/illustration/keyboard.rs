//! Keyboard artwork: a chassis with bevelled keycaps (skirt, shaded top face, legend).

use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, vec2,
};
use gearwise_core::keys::{KeyCell, KeyboardLayout};

use super::shapes::{rounded_poly, vertical_fan};
use crate::theme::{Palette, mix, with_alpha};

/// Where a keyboard layout lands inside an available rect.
#[derive(Clone, Copy)]
pub struct KbGeom {
    pub origin: Pos2,
    pub unit: f32,
    pub chassis: Rect,
}

pub fn keyboard_geom(avail: Rect, layout: &KeyboardLayout) -> KbGeom {
    let pad = 0.55;
    let (w, h) = (layout.width() + pad * 2.0, layout.height() + pad * 2.0);
    let unit = (avail.width() / w).min(avail.height() / h).clamp(6.0, 62.0);
    let chassis = Rect::from_center_size(avail.center(), vec2(w * unit, h * unit));
    KbGeom {
        origin: chassis.min + vec2(pad * unit, pad * unit),
        unit,
        chassis,
    }
}

pub fn key_rect(g: &KbGeom, c: &KeyCell) -> Rect {
    let gap = (g.unit * 0.06).max(1.0);
    Rect::from_min_size(
        g.origin + vec2(c.x * g.unit, c.y * g.unit),
        vec2(c.w * g.unit, c.h * g.unit),
    )
    .shrink(gap)
}

pub fn paint_chassis(p: &Painter, g: &KbGeom, pal: &Palette) {
    let r = CornerRadius::same((g.unit * 0.35) as u8);
    for i in 1..=4 {
        p.rect_filled(
            g.chassis
                .translate(vec2(0.0, 3.0 + 2.0 * i as f32))
                .expand(i as f32),
            r,
            Color32::from_black_alpha(if pal.dark { 18 } else { 7 }),
        );
    }
    let plate = if pal.dark {
        mix(pal.window, Color32::BLACK, 0.35)
    } else {
        mix(pal.border, pal.text, 0.3)
    };
    p.rect(
        g.chassis,
        r,
        plate,
        Stroke::new(1.0, mix(pal.border, pal.text, 0.2)),
        StrokeKind::Inside,
    );
    // Soft top edge highlight on the case.
    p.hline(
        g.chassis.min.x + g.unit * 0.4..=g.chassis.max.x - g.unit * 0.4,
        g.chassis.min.y + 1.5,
        Stroke::new(
            1.0,
            with_alpha(Color32::WHITE, if pal.dark { 22 } else { 70 }),
        ),
    );
}

#[derive(Clone, Copy)]
pub struct KeyPaint {
    /// Colour of the keycap top.
    pub fill: Color32,
    pub label: Color32,
    /// Outline of the keycap skirt (selection and hover rings).
    pub stroke: Stroke,
    /// LED colour spilling out from under the cap.
    pub glow: Option<Color32>,
}

pub fn paint_key(p: &Painter, rect: Rect, label: &str, paint: KeyPaint, unit: f32) {
    let r = (unit * 0.14).clamp(2.0, 6.0);
    if let Some(g) = paint.glow {
        p.rect_filled(
            rect.expand(unit * 0.26),
            CornerRadius::same((r + 6.0) as u8),
            with_alpha(g, 22),
        );
        p.rect_filled(
            rect.expand(unit * 0.12),
            CornerRadius::same((r + 3.0) as u8),
            with_alpha(g, 52),
        );
    }
    // Skirt, then the shaded top face.
    p.rect(
        rect,
        CornerRadius::same(r as u8),
        mix(paint.fill, Color32::BLACK, 0.5),
        paint.stroke,
        StrokeKind::Inside,
    );
    let inset = (unit * 0.07).max(1.0);
    let face = Rect::from_min_max(
        rect.min + vec2(inset, inset * 0.6),
        rect.max - vec2(inset, inset * 1.7),
    );
    if unit >= 14.0 {
        p.add(vertical_fan(
            &rounded_poly(face, r * 0.8, 3),
            mix(paint.fill, Color32::WHITE, 0.16),
            mix(paint.fill, Color32::BLACK, 0.12),
        ));
        p.hline(
            face.min.x + r..=face.max.x - r,
            face.min.y + 0.7,
            Stroke::new(1.0, with_alpha(Color32::WHITE, 46)),
        );
    } else {
        p.add(Shape::rect_filled(face, CornerRadius::same(2), paint.fill));
        return;
    }
    if label.is_empty() {
        return;
    }
    // The bundled font has no arrow glyphs, so draw them.
    let dir = match label {
        "↑" => Some(vec2(0.0, -1.0)),
        "↓" => Some(vec2(0.0, 1.0)),
        "←" => Some(vec2(-1.0, 0.0)),
        "→" => Some(vec2(1.0, 0.0)),
        _ => None,
    };
    if let Some(dir) = dir {
        let s = (unit * 0.2).clamp(3.0, 8.0);
        let perp = vec2(-dir.y, dir.x);
        let c = face.center();
        p.add(Shape::convex_polygon(
            vec![
                c + dir * s,
                c - dir * s * 0.7 + perp * s * 0.9,
                c - dir * s * 0.7 - perp * s * 0.9,
            ],
            paint.label,
            Stroke::NONE,
        ));
        return;
    }
    let size = (unit * 0.3).clamp(8.0, 13.0);
    let galley = p.layout_no_wrap(label.to_string(), FontId::proportional(size), paint.label);
    let font = if galley.size().x > face.width() - 3.0 {
        FontId::proportional(size * 0.78)
    } else {
        FontId::proportional(size)
    };
    crate::widgets::text_in(
        p,
        face.shrink(1.0),
        Align2::CENTER_CENTER,
        label,
        font,
        paint.label,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn keyboard_fits_inside_available_space() {
        let layout = gearwise_core::keys::ansi_full();
        let avail = Rect::from_min_size(pos2(10.0, 20.0), vec2(900.0, 400.0));
        let g = keyboard_geom(avail, &layout);
        assert!(avail.contains_rect(g.chassis.shrink(0.5)));
        let esc = layout.keys.iter().find(|k| k.id == 0x29).unwrap();
        assert!(g.chassis.contains_rect(key_rect(&g, esc)));
    }

    #[test]
    fn tiny_windows_still_get_a_usable_unit() {
        let layout = gearwise_core::keys::ansi_tkl();
        let g = keyboard_geom(Rect::from_min_size(Pos2::ZERO, vec2(50.0, 20.0)), &layout);
        assert_eq!(g.unit, 6.0);
    }
}
