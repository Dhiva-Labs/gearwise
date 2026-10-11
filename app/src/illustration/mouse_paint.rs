//! Painting of the mouse artwork: contact shadow, rim, domed shell, split buttons,
//! wheel, flank keys, logo LED, lighting glow and hover tint.

use std::f32::consts::PI;

use egui::epaint::{Mesh, PathStroke, Vertex, WHITE_UV};
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Shape, Stroke, StrokeKind, pos2, vec2,
};

use super::mouse::{Geo, Kind, MouseArt, MouseLook, Role, ZoneRole, seam_t};
use super::shapes::{
    centroid, convex_hull, glow, glow_line, gradient_fan, rounded_poly, seam, vertical_fan,
};
use crate::theme::{Palette, mix, with_alpha};

impl MouseArt {
    /// Paint the whole mouse. `tint` lists parts to light up (hover / selection).
    pub fn paint(&self, p: &Painter, pal: &Palette, look: &MouseLook, tint: &[(Role, f32)]) {
        if self.kind == Kind::G502 {
            return super::g502::paint(self, p, pal, look, tint);
        }
        let skin = Skin::new(pal);
        let g = &self.geo;
        let shape = g.outline();
        let closed = |pts: &[Pos2]| [pts, &pts[..1]].concat();

        // Contact shadow.
        for i in 1..=5 {
            let k = i as f32;
            let c = self.rect.center();
            let pts: Vec<Pos2> = shape
                .iter()
                .map(|q| pos2(q.x, q.y + 9.0) + (*q - c) * (0.012 * k))
                .collect();
            p.add(Shape::convex_polygon(
                pts,
                Color32::from_black_alpha(if pal.dark { 20 } else { 8 }),
                Stroke::NONE,
            ));
        }
        // Underglow from the body zone.
        if let Some(c) = look.body {
            glow_line(p, &closed(&shape), 5.0, c);
        }

        // Rim, then the domed shell inset inside it.
        p.add(Shape::convex_polygon(shape.clone(), skin.rim, Stroke::NONE));
        let shell = g.inset(3.5);
        p.add(self.dome(&shell, &skin));
        p.add(Shape::closed_line(
            shell.outline(),
            PathStroke::new(1.0, with_alpha(skin.hi, 70)),
        ));

        // Soft specular sheen on the palm.
        let sc = g.pt(0.56, 0.36);
        let radii = vec2(self.rect.width() * 0.2, self.rect.height() * 0.17);
        let sheen: Vec<Pos2> = (0..32)
            .map(|i| {
                sc + vec2(
                    (i as f32 * PI / 16.0).cos() * radii.x,
                    (i as f32 * PI / 16.0).sin() * radii.y,
                )
            })
            .collect();
        let sheen_alpha = if pal.dark { 40.0 } else { 90.0 };
        p.add(gradient_fan(&sheen, |q| {
            let d = ((q - sc) / radii).length().min(1.0);
            with_alpha(Color32::WHITE, (sheen_alpha * (1.0 - d * d)) as u8)
        }));

        // Palm panel groove and the primary buttons.
        for role in [Role::Left, Role::Right] {
            let Some(part) = self.part(role) else {
                continue;
            };
            p.add(vertical_fan(&part.poly, skin.btn_top, skin.btn_bottom));
            p.add(Shape::closed_line(
                part.poly.clone(),
                PathStroke::new(1.0, skin.groove),
            ));
        }
        let rear: Vec<Pos2> = (0..=24)
            .map(|i| i as f32 / 24.0)
            .map(|f| g.pt(seam_t(f), f))
            .collect();
        seam(p, &rear, skin.groove, with_alpha(skin.hi, 60));
        seam(
            p,
            &[g.pt(0.01, 0.5), g.pt(seam_t(0.5), 0.5)],
            skin.groove,
            with_alpha(skin.hi, 50),
        );

        // Thumb panel or grid backing.
        if self.kind == Kind::Mmo {
            let cells: Vec<Pos2> = self
                .parts
                .iter()
                .filter(|c| matches!(c.role, Role::Grid(_)))
                .flat_map(|c| c.poly.iter().copied())
                .collect();
            let mid = centroid(&cells);
            let padded: Vec<Pos2> = cells
                .iter()
                .map(|q| *q + (*q - mid).normalized() * 6.0)
                .collect();
            let panel = convex_hull(&padded);
            p.add(Shape::convex_polygon(
                panel.clone(),
                skin.groove,
                Stroke::NONE,
            ));
            p.add(Shape::closed_line(
                panel,
                PathStroke::new(1.0, with_alpha(skin.hi, 60)),
            ));
        }

        // Body-zone light strips along both flanks.
        if let Some(c) = look.body {
            for f in [0.035, 0.965] {
                let strip: Vec<Pos2> = (0..=24)
                    .map(|i| g.pt(0.2 + 0.68 * i as f32 / 24.0, f))
                    .collect();
                glow_line(p, &strip, 2.4, c);
            }
        }

        self.paint_wheel(p, &skin, look);
        for role in [Role::DpiUp, Role::DpiDown] {
            if let Some(part) = self.part(role) {
                self.key(p, &skin, &part.poly);
            }
        }
        if let Some(part) = self.part(Role::Profile) {
            self.key(p, &skin, &part.poly);
            p.circle_filled(
                part.anchor,
                self.rect.width() * 0.011,
                with_alpha(skin.hi, 120),
            );
        }
        for part in &self.parts {
            if matches!(
                part.role,
                Role::Sniper | Role::Back | Role::Forward | Role::Grid(_)
            ) {
                self.key(p, &skin, &part.poly);
            }
        }
        for part in &self.parts {
            if let Role::Grid(i) = part.role {
                p.text(
                    part.anchor,
                    Align2::CENTER_CENTER,
                    (i + 1).to_string(),
                    FontId::proportional(9.5),
                    with_alpha(skin.hi, 200),
                );
            }
        }
        if let Some(part) = self.part(Role::Sniper) {
            // A crosshair mark identifies the sniper button.
            let c = part.anchor;
            let s = self.rect.width() * 0.018;
            p.circle_stroke(c, s, Stroke::new(1.0, with_alpha(skin.hi, 150)));
            p.circle_filled(c, 1.3, with_alpha(skin.hi, 170));
        }
        self.paint_logo(p, &skin, look);

        // Rim light along the upper flanks.
        let hi_l: Vec<Pos2> = (0..=20)
            .map(|i| shell.pt(0.04 + 0.5 * i as f32 / 20.0, 0.0))
            .collect();
        let hi_r: Vec<Pos2> = (0..=20)
            .map(|i| shell.pt(0.04 + 0.4 * i as f32 / 20.0, 1.0))
            .collect();
        p.add(Shape::line(
            hi_l,
            PathStroke::new(1.6, with_alpha(skin.hi, 110)),
        ));
        p.add(Shape::line(
            hi_r,
            PathStroke::new(1.2, with_alpha(skin.hi, 55)),
        ));

        // Hover / selection tint.
        for (role, strength) in tint {
            if let Some(part) = self.part(*role) {
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

        self.paint_highlight(p, pal, look, &shape);
    }

    /// Shell body: a mesh with vertical shading and darker edges so it reads as domed.
    fn dome(&self, geo: &Geo, skin: &Skin) -> Shape {
        const ROWS: usize = 48;
        const COLS: [f32; 5] = [0.0, 0.1, 0.5, 0.9, 1.0];
        let mut mesh = Mesh::default();
        for t in Geo::depths(ROWS, 1.0) {
            let vertical = mix(skin.top, skin.bottom, t.powf(0.9));
            for f in COLS {
                let dome = (1.0 - (2.0 * f - 1.0).abs() * 1.0).clamp(0.0, 1.0);
                let lit = mix(vertical, skin.hi, 0.07 * (1.0 - f) * dome);
                let color = mix(skin.edge, lit, (dome * 3.2).clamp(0.0, 1.0));
                mesh.vertices.push(Vertex {
                    pos: geo.pt(t, f),
                    uv: WHITE_UV,
                    color,
                });
            }
        }
        let cols = COLS.len() as u32;
        for r in 0..ROWS as u32 {
            for c in 0..cols - 1 {
                let i = r * cols + c;
                mesh.add_triangle(i, i + 1, i + cols);
                mesh.add_triangle(i + 1, i + cols + 1, i + cols);
            }
        }
        Shape::mesh(mesh)
    }

    /// A small raised rubber/plastic key.
    fn key(&self, p: &Painter, skin: &Skin, poly: &[Pos2]) {
        let shifted: Vec<Pos2> = poly.iter().map(|q| *q + vec2(0.0, 1.5)).collect();
        p.add(Shape::convex_polygon(shifted, skin.groove, Stroke::NONE));
        p.add(vertical_fan(poly, skin.key_top, skin.key_bottom));
        p.add(Shape::closed_line(
            poly.to_vec(),
            PathStroke::new(1.0, with_alpha(skin.hi, 50)),
        ));
    }

    fn paint_wheel(&self, p: &Painter, skin: &Skin, look: &MouseLook) {
        let w = self.wheel;
        let slot = w.expand2(vec2(4.0, 5.0));
        p.rect_filled(
            slot,
            CornerRadius::same((slot.width() / 2.0) as u8),
            skin.groove,
        );
        p.rect_stroke(
            slot,
            CornerRadius::same((slot.width() / 2.0) as u8),
            Stroke::new(1.0, with_alpha(skin.hi, 45)),
            StrokeKind::Outside,
        );
        if let Some(c) = look.wheel {
            glow(p, w.center(), w.width() * 0.4, with_alpha(c, 60));
        }
        let poly = rounded_poly(w, w.width() / 2.0, 6);
        let cx = w.center().x;
        let half = w.width() / 2.0;
        let centre = look
            .wheel
            .map_or(skin.wheel_mid, |c| mix(skin.wheel_mid, c, 0.55));
        p.add(gradient_fan(&poly, |q| {
            mix(
                skin.wheel_edge,
                centre,
                (1.0 - ((q.x - cx).abs() / half).powi(2)).clamp(0.0, 1.0),
            )
        }));
        // Ridges.
        let n = (w.height() / 4.0) as i32;
        for i in 1..n {
            let y = w.min.y + w.height() * i as f32 / n as f32;
            p.hline(
                w.min.x + 2.0..=w.max.x - 2.0,
                y,
                Stroke::new(1.0, with_alpha(Color32::BLACK, 70)),
            );
            p.hline(
                w.min.x + 3.0..=w.max.x - 3.0,
                y + 1.0,
                Stroke::new(1.0, with_alpha(skin.hi, 40)),
            );
        }
        // Highlight stripe.
        p.vline(
            cx - half * 0.35,
            w.min.y + 4.0..=w.max.y - 4.0,
            Stroke::new(1.2, with_alpha(skin.hi, 70)),
        );
        p.rect_stroke(
            w,
            CornerRadius::same(half as u8),
            Stroke::new(1.0, with_alpha(Color32::BLACK, 120)),
            StrokeKind::Inside,
        );
    }

    fn paint_logo(&self, p: &Painter, skin: &Skin, look: &MouseLook) {
        let (c, r) = self.logo;
        match look.logo {
            Some(col) => {
                glow(p, c, r * 0.42, mix(col, Color32::WHITE, 0.25));
                p.circle_stroke(c, r, Stroke::new(1.6, with_alpha(col, 230)));
                p.circle_stroke(c, r * 0.72, Stroke::new(1.0, with_alpha(col, 110)));
            }
            None => {
                p.circle_filled(c, r, skin.groove);
                p.circle_stroke(c, r, Stroke::new(1.0, with_alpha(skin.hi, 70)));
                p.circle_stroke(c, r * 0.55, Stroke::new(1.0, with_alpha(skin.hi, 50)));
            }
        }
    }

    fn paint_highlight(&self, p: &Painter, pal: &Palette, look: &MouseLook, shape: &[Pos2]) {
        let Some(role) = look.highlight else { return };
        let ring = Stroke::new(1.5, pal.accent);
        match role {
            ZoneRole::Logo => {
                p.circle_stroke(self.logo.0, self.logo.1 * 1.9, ring);
            }
            ZoneRole::Wheel => {
                p.rect_stroke(
                    self.wheel.expand2(vec2(9.0, 10.0)),
                    CornerRadius::same(14),
                    ring,
                    StrokeKind::Outside,
                );
            }
            ZoneRole::Body => {
                p.extend(Shape::dashed_line(
                    &[shape, &shape[..1]].concat(),
                    ring,
                    6.0,
                    5.0,
                ));
            }
        }
    }
}

/// Palette-derived colours for the artwork.
struct Skin {
    rim: Color32,
    top: Color32,
    bottom: Color32,
    edge: Color32,
    hi: Color32,
    groove: Color32,
    btn_top: Color32,
    btn_bottom: Color32,
    key_top: Color32,
    key_bottom: Color32,
    wheel_mid: Color32,
    wheel_edge: Color32,
}

impl Skin {
    fn new(pal: &Palette) -> Self {
        if pal.dark {
            let top = mix(pal.raised, pal.text, 0.13);
            let bottom = mix(pal.raised, pal.window, 0.35);
            Self {
                rim: mix(pal.window, Color32::BLACK, 0.5),
                top,
                bottom,
                edge: mix(pal.card, Color32::BLACK, 0.35),
                hi: mix(pal.text, Color32::WHITE, 0.3),
                groove: mix(pal.window, Color32::BLACK, 0.45),
                btn_top: with_alpha(mix(top, pal.text, 0.06), 150),
                btn_bottom: with_alpha(mix(bottom, pal.text, 0.03), 120),
                key_top: mix(pal.raised, pal.text, 0.16),
                key_bottom: mix(pal.raised, pal.window, 0.2),
                wheel_mid: mix(pal.raised, pal.text, 0.25),
                wheel_edge: mix(pal.window, Color32::BLACK, 0.3),
            }
        } else {
            let top = Color32::WHITE;
            let bottom = mix(pal.raised, pal.border, 0.8);
            Self {
                rim: mix(pal.border, pal.text, 0.45),
                top,
                bottom,
                edge: mix(pal.border, pal.text, 0.12),
                hi: Color32::WHITE,
                groove: mix(pal.border, pal.text, 0.35),
                btn_top: with_alpha(Color32::WHITE, 170),
                btn_bottom: with_alpha(mix(bottom, Color32::WHITE, 0.5), 130),
                key_top: Color32::WHITE,
                key_bottom: mix(pal.raised, pal.border, 0.6),
                wheel_mid: mix(pal.border, Color32::WHITE, 0.35),
                wheel_edge: mix(pal.border, pal.text, 0.35),
            }
        }
    }
}
