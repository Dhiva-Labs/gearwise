//! Keyboard artwork: a rounded case with bevelled keycaps (skirt, shaded top face, legend). Boards
//! like the K55 also get the media bar above the function row (brand-neutral mark, mode buttons,
//! indicator LEDs, volume keys) and the attached wrist rest.

use egui::epaint::PathStroke;
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2,
    vec2,
};
use gearwise_core::keys::{KeyCell, KeyboardLayout};

use super::shapes::{glow, rounded_poly, vertical_fan};
use crate::theme::{Palette, mix, with_alpha};

/// What a board has around its keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardStyle {
    /// Media bar above the function row.
    pub top_bar: bool,
    /// Wrist rest attached along the bottom.
    pub wrist_rest: bool,
}

impl BoardStyle {
    /// Boards with a macro-key column (id 0x100 upwards) or named K55 are the full-featured case.
    pub fn for_layout(layout: &KeyboardLayout) -> Self {
        let full = layout.name.to_ascii_lowercase().contains("k55") || layout.contains(0x100);
        Self {
            top_bar: full,
            wrist_rest: full,
        }
    }
}

/// Space around the keys inside the case, in key units.
const PAD: f32 = 0.55;
/// Height of the media bar, the gap under it, and of the wrist rest, in key units.
const BAR_H: f32 = 1.5;
const BAR_GAP: f32 = 0.2;
const REST_H: f32 = 2.3;
const REST_GAP: f32 = 0.05;

/// Where a keyboard layout lands inside an available rect.
#[derive(Clone, Copy)]
pub struct KbGeom {
    /// Top-left of the key area (layout coordinates 0, 0).
    pub origin: Pos2,
    pub unit: f32,
    /// The case (media bar and keys).
    pub chassis: Rect,
    /// Case plus wrist rest.
    pub total: Rect,
    pub rest: Option<Rect>,
    pub style: BoardStyle,
}

pub fn keyboard_geom(avail: Rect, layout: &KeyboardLayout) -> KbGeom {
    let style = BoardStyle::for_layout(layout);
    let bar = if style.top_bar { BAR_H + BAR_GAP } else { PAD };
    let rest = if style.wrist_rest {
        REST_GAP + REST_H
    } else {
        0.0
    };
    let (w, case_h) = (layout.width() + PAD * 2.0, bar + layout.height() + PAD);
    let h = case_h + rest;
    let unit = (avail.width() / w).min(avail.height() / h).clamp(6.0, 62.0);
    let total = Rect::from_center_size(avail.center(), vec2(w * unit, h * unit));
    let chassis = Rect::from_min_size(total.min, vec2(w * unit, case_h * unit));
    let rest = style.wrist_rest.then(|| {
        Rect::from_min_max(
            pos2(chassis.min.x + unit * 0.2, chassis.max.y + unit * REST_GAP),
            pos2(chassis.max.x - unit * 0.2, total.max.y),
        )
    });
    KbGeom {
        origin: chassis.min + vec2(PAD * unit, bar * unit),
        unit,
        chassis,
        total,
        rest,
        style,
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

/// The case colour under the keys (flat, so zone outlines can be cut out of it).
pub fn plate_color(pal: &Palette) -> Color32 {
    if pal.dark {
        mix(pal.window, Color32::BLACK, 0.35)
    } else {
        // A black board with lit keys reads best on a light card too.
        mix(pal.text, Color32::BLACK, 0.2)
    }
}

/// Centres of the three LED indicator dots in the media bar, if the board has one.
pub fn indicator_dots(g: &KbGeom) -> Option<[Pos2; 3]> {
    g.style.top_bar.then(|| {
        let y = g.chassis.min.y + g.unit * BAR_H * 0.38;
        [0.80, 0.822, 0.844].map(|f| pos2(g.chassis.min.x + f * g.chassis.width(), y))
    })
}

/// Paint the case, the media bar and the wrist rest. `indicator` lights the three LED dots.
pub fn paint_chassis(p: &Painter, g: &KbGeom, pal: &Palette, indicator: Option<Color32>) {
    let u = g.unit;
    let r = CornerRadius::same((u * 0.45) as u8);
    let alpha = if pal.dark { 18 } else { 7 };
    for i in 1..=4 {
        p.rect_filled(
            g.total
                .translate(vec2(0.0, 3.0 + 2.0 * i as f32))
                .expand(i as f32),
            r,
            Color32::from_black_alpha(alpha),
        );
    }
    let plate = plate_color(pal);
    if let Some(rest) = g.rest {
        paint_rest(p, rest, u, pal, plate);
    }
    p.rect(
        g.chassis,
        r,
        plate,
        Stroke::new(1.0, mix(pal.border, pal.text, 0.2)),
        StrokeKind::Inside,
    );
    // Soft top edge highlight on the case.
    p.hline(
        g.chassis.min.x + u * 0.5..=g.chassis.max.x - u * 0.5,
        g.chassis.min.y + 1.5,
        Stroke::new(
            1.0,
            with_alpha(Color32::WHITE, if pal.dark { 22 } else { 70 }),
        ),
    );
    if g.style.top_bar {
        paint_bar(p, g, pal, indicator);
    }
}

fn paint_rest(p: &Painter, rest: Rect, u: f32, pal: &Palette, plate: Color32) {
    let top = (u * 0.25) as u8;
    let bottom = (u * 0.7) as u8;
    let radius = CornerRadius {
        nw: top,
        ne: top,
        sw: bottom,
        se: bottom,
    };
    let base = mix(plate, Color32::BLACK, if pal.dark { 0.25 } else { 0.1 });
    p.rect(
        rest,
        radius,
        base,
        Stroke::new(1.0, mix(pal.border, pal.text, 0.15)),
        StrokeKind::Inside,
    );
    // Fine ribbing, like the real pad's texture.
    let mut y = rest.min.y + u * 0.35;
    while y < rest.max.y - u * 0.5 {
        p.hline(
            rest.min.x + u * 0.5..=rest.max.x - u * 0.5,
            y,
            Stroke::new(1.0, with_alpha(pal.text, if pal.dark { 7 } else { 14 })),
        );
        y += u * 0.3;
    }
    // A small neutral mark centred near the bottom edge.
    let c = pos2(rest.center().x, rest.max.y - u * 0.55);
    let d = u * 0.17;
    p.add(Shape::closed_line(
        vec![
            c + vec2(0.0, -d),
            c + vec2(d, 0.0),
            c + vec2(0.0, d),
            c + vec2(-d, 0.0),
        ],
        PathStroke::new(1.0, with_alpha(pal.text, 60)),
    ));
}

/// Media bar: a small brand-neutral mark, three round mode buttons with their LEDs, volume keys.
fn paint_bar(p: &Painter, g: &KbGeom, pal: &Palette, indicator: Option<Color32>) {
    let u = g.unit;
    let ch = g.chassis;
    let y = ch.min.y + u * BAR_H * 0.55;
    let ink = with_alpha(pal.text, if pal.dark { 120 } else { 150 });
    let well = if pal.dark {
        mix(plate_color(pal), Color32::BLACK, 0.4)
    } else {
        mix(plate_color(pal), Color32::BLACK, 0.25)
    };

    // Mark: a hollow rhombus around a dot.
    let c = pos2(ch.center().x, y);
    let (w, h) = (u * 0.52, u * 0.34);
    p.add(Shape::closed_line(
        vec![
            c + vec2(0.0, -h),
            c + vec2(w, 0.0),
            c + vec2(0.0, h),
            c + vec2(-w, 0.0),
        ],
        PathStroke::new(1.3, ink),
    ));
    p.circle_filled(c, (u * 0.07).max(1.0), ink);

    // Mode buttons.
    for f in [0.669, 0.708, 0.749] {
        let c = pos2(ch.min.x + f * ch.width(), y);
        let rad = u * 0.3;
        p.circle_filled(c, rad, well);
        p.circle_stroke(c, rad, Stroke::new(1.0, with_alpha(pal.text, 40)));
        p.circle_stroke(c, rad * 0.55, Stroke::new(1.0, with_alpha(pal.text, 28)));
    }

    // Indicator LEDs.
    if let Some(dots) = indicator_dots(g) {
        for d in dots {
            match indicator {
                Some(col) => glow(p, d, (u * 0.08).max(1.2), col),
                None => {
                    p.circle_filled(d, (u * 0.08).max(1.2), with_alpha(pal.text, 40));
                }
            }
        }
    }

    // Volume keys: mute, down, up.
    let (x0, x1) = (ch.min.x + 0.868 * ch.width(), ch.min.x + 0.975 * ch.width());
    let kw = (x1 - x0) / 3.0;
    let rail = Rect::from_min_max(pos2(x0, y - u * 0.3), pos2(x1, y + u * 0.3));
    p.rect_filled(rail, CornerRadius::same((u * 0.15) as u8), well);
    for i in 0..3 {
        let kc = pos2(x0 + kw * (i as f32 + 0.5), y);
        if i > 0 {
            p.vline(
                x0 + kw * i as f32,
                rail.min.y + 2.0..=rail.max.y - 2.0,
                Stroke::new(1.0, with_alpha(pal.text, 30)),
            );
        }
        volume_glyph(p, kc, u, i, ink);
    }
}

/// A speaker with nothing (mute), a minus or a plus.
fn volume_glyph(p: &Painter, c: Pos2, u: f32, kind: usize, ink: Color32) {
    let s = u * 0.16;
    let x = c.x - s * 0.9;
    p.add(Shape::convex_polygon(
        vec![
            pos2(x - s * 0.5, c.y - s * 0.5),
            pos2(x + s * 0.2, c.y - s * 0.5),
            pos2(x + s * 1.1, c.y - s * 1.1),
            pos2(x + s * 1.1, c.y + s * 1.1),
            pos2(x + s * 0.2, c.y + s * 0.5),
            pos2(x - s * 0.5, c.y + s * 0.5),
        ],
        ink,
        Stroke::NONE,
    ));
    let sign = pos2(c.x + s * 1.9, c.y);
    let st = Stroke::new(1.2, ink);
    match kind {
        0 => {
            p.line_segment(
                [
                    sign + vec2(-s * 0.6, -s * 0.6),
                    sign + vec2(s * 0.6, s * 0.6),
                ],
                st,
            );
            p.line_segment(
                [
                    sign + vec2(-s * 0.6, s * 0.6),
                    sign + vec2(s * 0.6, -s * 0.6),
                ],
                st,
            );
        }
        1 => {
            p.line_segment([sign + vec2(-s * 0.7, 0.0), sign + vec2(s * 0.7, 0.0)], st);
        }
        _ => {
            p.line_segment([sign + vec2(-s * 0.7, 0.0), sign + vec2(s * 0.7, 0.0)], st);
            p.line_segment([sign + vec2(0.0, -s * 0.7), sign + vec2(0.0, s * 0.7)], st);
        }
    }
}

/// Outline exactly the keys in `rects` (their union, not a bounding box): an accent band is
/// painted under the keys around every cap, then the case colour fills the gaps between
/// neighbours so only the outer edge of the group keeps the band.
pub fn paint_zone_outline(p: &Painter, g: &KbGeom, rects: &[Rect], pal: &Palette) {
    let gap = (g.unit * 0.06).max(1.0);
    let inner = gap + 1.0;
    let band = 2.2;
    let radius = |extra: f32| CornerRadius::same(((g.unit * 0.14).clamp(2.0, 6.0) + extra) as u8);
    for r in rects {
        p.rect_filled(
            r.expand(inner + band + 3.0),
            radius(inner + band + 3.0),
            with_alpha(pal.accent, 34),
        );
    }
    for r in rects {
        p.rect_filled(r.expand(inner + band), radius(inner + band), pal.accent);
    }
    for r in rects {
        p.rect_filled(r.expand(inner), radius(inner), plate_color(pal));
    }
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
    // The bundled font has no arrow or media glyphs, so draw them.
    if paint_glyph(p, face, label, paint.label) {
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

/// Draw the glyph for an arrow or media key label; false when `label` is ordinary text.
fn paint_glyph(p: &Painter, face: Rect, label: &str, color: Color32) -> bool {
    let c = face.center();
    let s = (face.height() * 0.22).clamp(2.5, 8.0);
    let tri = |tip: Pos2, dir: f32, size: f32| {
        p.add(Shape::convex_polygon(
            vec![
                tip,
                tip + vec2(-dir * size * 1.5, -size),
                tip + vec2(-dir * size * 1.5, size),
            ],
            color,
            Stroke::NONE,
        ));
    };
    let bar = |x: f32, half: f32| {
        p.add(Shape::rect_filled(
            Rect::from_center_size(pos2(x, c.y), vec2((s * 0.45).max(1.2), half * 2.0)),
            CornerRadius::ZERO,
            color,
        ));
    };
    match label {
        "↑" | "↓" | "←" | "→" => {
            let dir = match label {
                "↑" => vec2(0.0, -1.0),
                "↓" => vec2(0.0, 1.0),
                "←" => vec2(-1.0, 0.0),
                _ => vec2(1.0, 0.0),
            };
            let perp = vec2(-dir.y, dir.x);
            let s = (face.height() * 0.28).clamp(3.0, 8.0);
            p.add(Shape::convex_polygon(
                vec![
                    c + dir * s,
                    c - dir * s * 0.7 + perp * s * 0.9,
                    c - dir * s * 0.7 - perp * s * 0.9,
                ],
                color,
                Stroke::NONE,
            ));
        }
        "Stop" => {
            p.add(Shape::rect_filled(
                Rect::from_center_size(c, vec2(s * 1.5, s * 1.5)),
                CornerRadius::same(1),
                color,
            ));
        }
        "Prev" => {
            bar(c.x - s * 1.55, s);
            tri(pos2(c.x - s * 0.1, c.y), -1.0, s);
            tri(pos2(c.x + s * 1.45, c.y), -1.0, s);
        }
        "Next" => {
            bar(c.x + s * 1.55, s);
            tri(pos2(c.x + s * 0.1, c.y), 1.0, s);
            tri(pos2(c.x - s * 1.45, c.y), 1.0, s);
        }
        "Play" => {
            tri(pos2(c.x - s * 0.1, c.y), 1.0, s);
            bar(c.x + s * 0.7, s);
            bar(c.x + s * 1.5, s);
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k55_like() -> KeyboardLayout {
        let mut l = gearwise_core::keys::ansi_full();
        l.name = "K55 RGB PRO".into();
        l
    }

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

    #[test]
    fn k55_style_boards_get_a_bar_and_a_wrist_rest_inside_the_space() {
        let layout = k55_like();
        assert!(BoardStyle::for_layout(&layout).top_bar);
        assert!(!BoardStyle::for_layout(&gearwise_core::keys::ansi_full()).wrist_rest);
        let avail = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 520.0));
        let g = keyboard_geom(avail, &layout);
        assert!(avail.contains_rect(g.total.shrink(0.5)));
        let rest = g.rest.expect("wrist rest");
        assert!(rest.min.y >= g.chassis.max.y && rest.max.y <= g.total.max.y + 0.01);
        // Keys start below the bar, and the indicator dots sit inside it.
        let first = layout
            .keys
            .iter()
            .map(|k| key_rect(&g, k).min.y)
            .fold(f32::MAX, f32::min);
        assert!(first > g.chassis.min.y + g.unit * BAR_H);
        let dots = indicator_dots(&g).unwrap();
        assert!(dots.iter().all(|d| g.chassis.contains(*d) && d.y < first));
    }

    #[test]
    fn glyph_keys_have_no_text_fallback() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let face = Rect::from_min_size(pos2(10.0, 10.0), vec2(30.0, 24.0));
            for l in ["Stop", "Prev", "Play", "Next", "↑", "←"] {
                assert!(paint_glyph(ui.painter(), face, l, Color32::WHITE), "{l}");
            }
            assert!(!paint_glyph(ui.painter(), face, "Esc", Color32::WHITE));
        });
        out.textures_delta.clear();
    }
}
