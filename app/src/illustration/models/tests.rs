use std::collections::HashSet;

use egui::{Color32, Pos2, RawInput, vec2};

use super::*;
use crate::illustration::mouse::{Design, art_rect};
use crate::illustration::roles::ZoneRole;
use crate::theme::apply;
use gearwise_core::profile::Theme;

/// A drawing that only has a name, for exercising the lookup rules.
struct Named {
    id: &'static str,
    word: &'static str,
    priority: i32,
}

impl ModelArt for Named {
    fn id(&self) -> &'static str {
        self.id
    }
    fn matches(&self, name: &str) -> bool {
        name.contains(self.word)
    }
    fn priority(&self) -> i32 {
        self.priority
    }
    fn examples(&self) -> &'static [&'static str] {
        &[]
    }
    fn aspect(&self) -> f32 {
        0.6
    }
    fn build(&self, _: Rect) -> Anatomy {
        unreachable!()
    }
    fn paint(&self, _: &MouseArt, _: &Painter, _: &Palette, _: &MouseLook, _: &[(Role, f32)]) {
        unreachable!()
    }
}

#[test]
fn the_most_specific_match_wins_and_ties_go_to_the_first_registered() {
    let family = Named {
        id: "family",
        word: "viper",
        priority: 0,
    };
    let variant = Named {
        id: "variant",
        word: "viper v3",
        priority: 10,
    };
    let twin = Named {
        id: "twin",
        word: "viper",
        priority: 0,
    };
    let all = || [&family as &dyn ModelArt, &variant, &twin].into_iter();
    let id = |name: &str| best_match(all(), name).map(|m| m.id());
    assert_eq!(id("razer viper v3 pro"), Some("variant"));
    assert_eq!(
        id("razer viper ultimate"),
        Some("family"),
        "first of two equals"
    );
    assert_eq!(id("razer deathadder"), None);
    // Registration order matters only on ties.
    let reversed = [&twin as &dyn ModelArt, &family].into_iter();
    assert_eq!(best_match(reversed, "viper").map(|m| m.id()), Some("twin"));
}

#[test]
fn lookup_ignores_case_and_unknown_names() {
    assert_eq!(lookup("MX MASTER 3S").map(|m| m.id()), Some("mx-master"));
    assert!(lookup("Totally Unknown Mouse").is_none());
}

#[test]
fn ids_are_unique_and_every_example_name_resolves_to_its_own_model() {
    let mut ids = HashSet::new();
    for m in all() {
        assert!(ids.insert(m.id()), "id {} is registered twice", m.id());
        assert!(
            !m.examples().is_empty(),
            "{} lists no example names",
            m.id()
        );
        for name in m.examples() {
            let found = lookup(name).map(|f| f.id());
            assert_eq!(found, Some(m.id()), "{name} should be drawn by {}", m.id());
        }
    }
}

fn all_designs() -> Vec<Design> {
    all().map(Design::Model).collect()
}

/// The parts of every registered model, at several sizes: inside the box, sensibly shaped, one
/// per role, with the anchor on the part and the wheel and logo zones in the box.
#[test]
fn every_model_builds_sane_parts_at_every_size() {
    for design in all_designs() {
        let Design::Model(m) = design else {
            unreachable!()
        };
        assert!((0.25..2.0).contains(&m.aspect()), "{} aspect", m.id());
        assert!((0.0..0.5).contains(&m.overhang()), "{} overhang", m.id());
        for h in [260.0f32, 430.0, 600.0] {
            let rect = art_rect(
                Rect::from_min_size(Pos2::new(40.0, 30.0), vec2(h * m.aspect() * 1.5, h * 1.2)),
                design,
            );
            let art = MouseArt::new(rect, design);
            assert!(!art.parts.is_empty(), "{} has no parts", m.id());
            let mut roles = HashSet::new();
            for part in &art.parts {
                let who = format!("{} {:?} at {h}", m.id(), part.role);
                assert!(roles.insert(part.role), "{who}: role placed twice");
                assert!(part.poly.len() >= 3, "{who}: degenerate polygon");
                assert!(
                    part.poly
                        .iter()
                        .all(|q| q.is_finite() && rect.expand(3.0).contains(*q)),
                    "{who}: leaves the box"
                );
                let bounds = Rect::from_points(&part.poly).expand(4.0);
                assert!(
                    bounds.contains(part.anchor),
                    "{who}: anchor is off the part"
                );
            }
            assert!(
                rect.expand(3.0).contains_rect(art.wheel),
                "{} wheel",
                m.id()
            );
            assert!(rect.expand(3.0).contains(art.logo.0), "{} logo", m.id());
        }
    }
}

/// Paint every model with every part tinted and every lighting zone selected, in both themes.
#[test]
fn models_paint_with_every_tint_and_highlight_in_both_themes() {
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 700.0));
    for theme in [Theme::Dark, Theme::Light] {
        let ctx = egui::Context::default();
        let pal = Palette::new(theme, Color32::from_rgb(0, 212, 170));
        apply(&ctx, &pal);
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                for design in all_designs() {
                    let rect = art_rect(screen.shrink(60.0), design);
                    let art = MouseArt::new(rect, design);
                    let tint: Vec<(Role, f32)> = art.parts.iter().map(|p| (p.role, 0.8)).collect();
                    for highlight in [
                        None,
                        Some(ZoneRole::Logo),
                        Some(ZoneRole::Wheel),
                        Some(ZoneRole::Body),
                    ] {
                        let look = MouseLook {
                            logo: Some(Color32::RED),
                            wheel: Some(Color32::GREEN),
                            body: Some(Color32::BLUE),
                            highlight,
                            ..Default::default()
                        };
                        art.paint(ui.painter(), &pal, &look, &tint);
                    }
                    art.paint(ui.painter(), &pal, &MouseLook::default(), &[]);
                }
            },
        );
        out.textures_delta.clear();
        assert!(out.shapes.len() > 50, "{theme:?} produced no art");
    }
}
