//! Painter-drawn device art: mice, keyboards and headsets, plus the callouts that
//! tie buttons to the picture.

pub mod callouts;
pub mod headset;
pub mod keyboard;
pub mod mouse;
mod mouse_paint;
pub mod shapes;

use egui::{Pos2, Rect, pos2};

pub use keyboard::{KbGeom, KeyPaint, key_rect, keyboard_geom, paint_chassis, paint_key};
pub use mouse::{Kind, MouseArt, MouseLook, Role, ZoneRole, art_rect, zone_role};

/// Map a normalised device coordinate (0-1, x left to right, y front to back) onto `rect`.
pub fn at(rect: Rect, p: (f32, f32)) -> Pos2 {
    pos2(rect.min.x + p.0 * rect.width(), rect.min.y + p.1 * rect.height())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Palette, apply};
    use egui::{Color32, RawInput, vec2};
    use periph_core::profile::Theme;

    #[test]
    fn hotspots_map_into_the_rect() {
        let r = Rect::from_min_size(pos2(100.0, 50.0), vec2(200.0, 400.0));
        assert_eq!(at(r, (0.5, 0.25)), pos2(200.0, 150.0));
        let m = art_rect(Rect::from_min_size(Pos2::ZERO, vec2(1000.0, 500.0)), Kind::Ambidextrous);
        assert!((m.width() / m.height() - Kind::Ambidextrous.aspect()).abs() < 1e-3);
    }

    /// Paint every silhouette, lit and unlit, tinted and plain, in both themes.
    #[test]
    fn every_silhouette_paints_in_both_themes() {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 700.0));
        for theme in [Theme::Dark, Theme::Light] {
            let ctx = egui::Context::default();
            let pal = Palette::new(theme, Color32::from_rgb(0, 212, 170));
            apply(&ctx, &pal);
            let mut out = ctx.run_ui(RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| {
                for kind in [Kind::Ergonomic, Kind::ErgoLite, Kind::Ambidextrous, Kind::Mmo] {
                    let rect = art_rect(Rect::from_min_size(pos2(20.0, 20.0), vec2(500.0, 440.0)), kind);
                    let art = MouseArt::new(rect, kind);
                    let lit = MouseLook { logo: Some(Color32::RED), wheel: Some(Color32::GREEN), body: Some(Color32::BLUE), highlight: Some(ZoneRole::Body) };
                    art.paint(ui.painter(), &pal, &MouseLook::default(), &[(Role::Left, 1.0), (Role::Grid(3), 0.7), (Role::Sniper, 1.0)]);
                    art.paint(ui.painter(), &pal, &lit, &[]);
                }
                headset::paint_headset(ui.painter(), screen.shrink(100.0), &pal, &MouseLook { logo: Some(Color32::YELLOW), ..Default::default() });
                headset::paint_headset(ui.painter(), screen.shrink(100.0), &pal, &MouseLook::default());
            });
            out.textures_delta.clear();
            assert!(out.shapes.len() > 10, "{theme:?} produced no art");
        }
    }
}
