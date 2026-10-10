//! Data shared between the app shell, the sidebar and the pages.

use gearwise_core::device::{Capabilities, DeviceDescriptor, DeviceState, Setting};

use crate::worker::DeviceSnapshot;

/// A device as the UI sees it: the last known state, kept optimistically ahead of the hardware.
pub struct DeviceView {
    pub desc: DeviceDescriptor,
    pub caps: Capabilities,
    pub state: DeviceState,
    /// False while a wireless device is asleep or out of range.
    pub online: bool,
    /// Commands sent to the worker that have not been answered yet.
    pub pending: u32,
    /// A re-read arrived while commands were in flight; fetch again when idle.
    pub stale: bool,
}

impl DeviceView {
    pub fn new(s: DeviceSnapshot) -> Self {
        Self {
            desc: s.descriptor,
            caps: s.capabilities,
            state: s.state,
            online: true,
            pending: 0,
            stale: false,
        }
    }

    /// Name without the " (demo)" suffix simulated devices carry (a Demo badge says so).
    pub fn display_name(&self) -> &str {
        self.desc
            .name
            .strip_suffix(" (demo)")
            .unwrap_or(&self.desc.name)
    }

    pub fn tabs(&self) -> Vec<Tab> {
        let c = &self.caps;
        let mut t = Vec::new();
        if !c.buttons.is_empty() {
            t.push(Tab::Assignments);
        }
        if c.dpi.is_some() {
            t.push(Tab::Sensitivity);
        }
        if !c.lighting.is_empty() || c.per_key.is_some() {
            t.push(Tab::Lighting);
        }
        if !c.report_rates.is_empty() || !c.lift_off.is_empty() || c.angle_snapping || c.game_mode {
            t.push(Tab::Performance);
        }
        t.push(Tab::Info);
        t
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Device,
    Profiles,
    Macros,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tab {
    Assignments,
    Sensitivity,
    Lighting,
    Performance,
    Info,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Assignments => "Assignments",
            Tab::Sensitivity => "Sensitivity",
            Tab::Lighting => "Lighting",
            Tab::Performance => "Performance",
            Tab::Info => "Info",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

/// What a page asks the app shell to do. Pages never touch devices or the worker.
pub enum Action {
    Apply {
        dev: String,
        setting: Setting,
        debounce: bool,
    },
    ApplyMany {
        dev: String,
        settings: Vec<Setting>,
    },
    SaveOnboard(String),
    SaveToProfile(String),
    /// Make a profile the active one and apply it to connected devices.
    ActivateProfile(String),
    /// Re-apply a profile to connected devices without changing the active one.
    ApplyProfile(String),
    SelectDevice(String),
    Navigate(Route),
    SetDemo(bool),
    Rescan,
    CopyText(String),
    Toast(ToastKind, String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingKey {
    Dpi,
    DpiActive,
    ReportRate,
    Lighting(u8),
    PerKey,
    Button(u16),
    OnboardProfile,
    LiftOff,
    AngleSnapping,
    GameMode,
}

pub fn setting_key(s: &Setting) -> SettingKey {
    match s {
        Setting::Dpi(_) => SettingKey::Dpi,
        Setting::DpiActiveStage(_) => SettingKey::DpiActive,
        Setting::ReportRate(_) => SettingKey::ReportRate,
        Setting::Lighting { zone, .. } => SettingKey::Lighting(*zone),
        Setting::PerKey(_) => SettingKey::PerKey,
        Setting::Button { id, .. } => SettingKey::Button(*id),
        Setting::OnboardProfile(_) => SettingKey::OnboardProfile,
        Setting::LiftOff(_) => SettingKey::LiftOff,
        Setting::AngleSnapping(_) => SettingKey::AngleSnapping,
        Setting::GameMode(_) => SettingKey::GameMode,
    }
}

/// Fold a queued setting into a newer one for the same key: per-key colours accumulate, anything else is replaced.
pub fn merge_settings(old: Setting, new: Setting) -> Setting {
    match (old, new) {
        (Setting::PerKey(mut a), Setting::PerKey(b)) => {
            a.extend(b);
            Setting::PerKey(a)
        }
        (_, new) => new,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gearwise_core::model::{ButtonAction, Rgb};

    #[test]
    fn keys_distinguish_zones_and_buttons() {
        let z = |zone| Setting::Lighting {
            zone,
            state: gearwise_core::model::ZoneState {
                effect: gearwise_core::model::LightingEffect::Off,
                brightness: 0,
            },
        };
        assert_ne!(setting_key(&z(0)), setting_key(&z(1)));
        let b = |id| Setting::Button {
            id,
            action: ButtonAction::Disabled,
        };
        assert_ne!(setting_key(&b(1)), setting_key(&b(2)));
        assert_eq!(
            setting_key(&Setting::ReportRate(500)),
            setting_key(&Setting::ReportRate(1000))
        );
    }

    #[test]
    fn per_key_updates_accumulate_but_others_replace() {
        let a = Setting::PerKey([(4, Rgb::new(1, 0, 0))].into());
        let b = Setting::PerKey([(5, Rgb::new(0, 1, 0)), (4, Rgb::new(9, 9, 9))].into());
        match merge_settings(a, b) {
            Setting::PerKey(m) => {
                assert_eq!(m.len(), 2);
                assert_eq!(m[&4], Rgb::new(9, 9, 9));
            }
            _ => panic!("expected per-key"),
        }
        assert_eq!(
            merge_settings(Setting::ReportRate(125), Setting::ReportRate(1000)),
            Setting::ReportRate(1000)
        );
    }

    #[test]
    fn demo_suffix_is_dropped_from_display_names() {
        let mut d = gearwise_core::demo::devices().remove(0);
        let state = d.read_state().unwrap();
        let v = DeviceView::new(DeviceSnapshot {
            descriptor: d.descriptor().clone(),
            capabilities: d.capabilities().clone(),
            state,
        });
        assert!(v.desc.name.ends_with("(demo)"));
        assert_eq!(v.display_name(), "G502 X Plus");
    }

    #[test]
    fn tabs_follow_capabilities() {
        let snap = |caps: Capabilities| {
            DeviceView::new(DeviceSnapshot {
                descriptor: gearwise_core::demo::devices()[0].descriptor().clone(),
                capabilities: caps,
                state: DeviceState::default(),
            })
        };
        assert_eq!(snap(Capabilities::default()).tabs(), vec![Tab::Info]);
        let demo = gearwise_core::demo::devices();
        let tabs = |i: usize| snap(demo[i].capabilities().clone()).tabs();
        assert_eq!(
            tabs(0),
            vec![
                Tab::Assignments,
                Tab::Sensitivity,
                Tab::Lighting,
                Tab::Performance,
                Tab::Info
            ]
        );
        assert_eq!(
            tabs(1),
            vec![Tab::Assignments, Tab::Lighting, Tab::Performance, Tab::Info]
        );
    }
}
