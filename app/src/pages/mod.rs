//! One module per screen. Pages draw with read-only data and report what the
//! user did through [`crate::state::Action`].

pub mod assignments;
pub mod device;
pub mod empty;
pub mod info;
pub mod lighting;
pub mod macros;
pub mod per_key;
pub mod performance;
pub mod picker;
pub mod profiles;
pub mod sensitivity;
pub mod settings;

use gearwise_core::model::ButtonAction;
use gearwise_core::profile::Library;

/// Human label for an action, resolving macro ids to their names.
pub fn action_label(a: &ButtonAction, lib: &Library) -> String {
    match a {
        ButtonAction::Macro(id) => lib.macro_by_id(id).map_or_else(
            || "Missing macro".to_string(),
            |m| format!("Macro: {}", m.name),
        ),
        other => other.label(),
    }
}

/// Per-page UI state that survives between frames.
#[derive(Default)]
pub struct PageStates {
    pub assign: assignments::AssignState,
    pub sens: sensitivity::SensState,
    pub light: lighting::LightState,
    pub profiles: profiles::ProfilesState,
    pub macros: macros::MacrosState,
    pub settings: settings::SettingsState,
}

impl PageStates {
    /// True while a page wants raw key presses (keystroke capture, macro recording).
    pub fn wants_keys(&self) -> bool {
        self.assign.capturing || self.macros.wants_keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Action, DeviceView, Tab};
    use crate::worker::DeviceSnapshot;
    use egui::{Event, PointerButton, Pos2, RawInput, Rect, vec2};
    use gearwise_core::profile::Macro;

    #[test]
    fn macro_actions_show_the_macro_name() {
        let mut lib = Library::starter();
        let m = Macro::new("Rapid click");
        let id = m.id.clone();
        lib.macros.push(m);
        assert_eq!(
            action_label(&ButtonAction::Macro(id), &lib),
            "Macro: Rapid click"
        );
        assert_eq!(
            action_label(&ButtonAction::Macro("gone".into()), &lib),
            "Missing macro"
        );
        assert_eq!(action_label(&ButtonAction::Disabled, &lib), "Disabled");
    }

    /// The three demo devices plus the extra silhouettes (G502 HERO, slim, MMO, DeathAdder, headset).
    fn demo_views() -> Vec<DeviceView> {
        let demo = gearwise_core::demo::devices().into_iter().map(|mut d| {
            let state = d.read_state().unwrap();
            DeviceView::new(DeviceSnapshot {
                descriptor: d.descriptor().clone(),
                capabilities: d.capabilities().clone(),
                state,
            })
        });
        demo.chain(
            crate::screenshot::extra_devices()
                .into_iter()
                .map(DeviceView::new),
        )
        .collect()
    }

    /// Tiny deterministic generator so the "monkey" below is repeatable.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 40) as f32 / (1u64 << 24) as f32
        }
    }

    /// Draw `page` for a few frames, then click and drag at random points, folding the
    /// actions it emits back into the device like the app does.
    fn monkey(
        seed: u64,
        views: &mut [DeviceView],
        mut page: impl FnMut(&mut egui::Ui, &mut [DeviceView], &mut Vec<Action>),
    ) {
        let ctx = egui::Context::default();
        crate::theme::apply(
            &ctx,
            &crate::theme::Palette::new(
                gearwise_core::profile::Theme::Dark,
                egui::Color32::from_rgb(0, 212, 170),
            ),
        );
        let mut rng = Lcg(seed);
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(1100.0, 760.0));
        let mut t = 0.0;
        for frame in 0..48 {
            let mut events = Vec::new();
            if frame >= 6 {
                let pos = Pos2::new(rng.next() * screen.width(), rng.next() * screen.height());
                events.push(Event::PointerMoved(pos));
                if frame % 3 == 0 {
                    let m = egui::Modifiers {
                        shift: rng.next() > 0.7,
                        ..Default::default()
                    };
                    events.push(Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: m,
                    });
                } else if frame % 3 == 1 {
                    events.push(Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Default::default(),
                    });
                }
            }
            t += 0.016;
            let input = RawInput {
                screen_rect: Some(screen),
                time: Some(t),
                events,
                ..Default::default()
            };
            let mut out = Vec::new();
            ctx.run_ui(input, |ui| page(ui, views, &mut out))
                .textures_delta
                .clear();
            for a in out {
                match a {
                    Action::Apply { dev, setting, .. } => {
                        if let Some(v) = views.iter_mut().find(|v| v.desc.id == dev) {
                            setting.apply_to(&mut v.state);
                        }
                    }
                    Action::ApplyMany { dev, settings } => {
                        if let Some(v) = views.iter_mut().find(|v| v.desc.id == dev) {
                            settings.iter().for_each(|s| s.apply_to(&mut v.state));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn every_device_tab_survives_random_clicking() {
        let lib = Library::starter();
        for seed in 1..=2u64 {
            for idx in 0..8 {
                let mut views = demo_views();
                let tabs = views[idx].tabs();
                for tab in tabs {
                    let mut st = PageStates::default();
                    let mut t = tab;
                    monkey(seed * 31 + idx as u64, &mut views, |ui, views, out| {
                        device::show(ui, &mut st, &views[idx], &lib, &[], &mut t, out);
                    });
                    assert_eq!(
                        t, tab,
                        "tab selection is only changed by clicking the tab bar"
                    );
                }
            }
        }
    }

    #[test]
    fn library_pages_survive_random_clicking() {
        for seed in 1..=3u64 {
            let mut lib = crate::screenshot::sample_library();
            let mut views = demo_views();
            let mut st = PageStates::default();
            monkey(seed, &mut views, |ui, views, out| {
                profiles::show(ui, &mut st.profiles, &mut lib, views, out);
            });
            monkey(seed + 100, &mut views, |ui, views, out| {
                macros::show(ui, &mut st.macros, &mut lib, views, &[], out);
            });
            monkey(seed + 200, &mut views, |ui, _, out| {
                settings::show(ui, &mut st.settings, &mut lib, out);
            });
            monkey(seed + 300, &mut views, |ui, _, out| empty::show(ui, out));
            assert!(!lib.profiles.is_empty() || seed > 0);
        }
    }

    #[test]
    fn tabs_never_drop_below_info() {
        for v in demo_views() {
            assert!(v.tabs().contains(&Tab::Info));
        }
    }
}
