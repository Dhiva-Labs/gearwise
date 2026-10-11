//! Logitech demo devices and tour steps for the model-accurate art (see `super`).

use gearwise_core::model::{
    Battery, ButtonAction, Connection, DeviceKind, DpiCaps, DpiStage, DpiState, Modifiers,
    MouseButton, Vendor, media,
};

use crate::screenshot::Step;
use crate::screenshot_demo::{Spec, btn, mouse_like};
use crate::state::Tab;
use crate::worker::DeviceSnapshot;

const MX_MASTER: &str = "logitech:b034";

pub(super) fn devices() -> Vec<DeviceSnapshot> {
    vec![mx_master_3s()]
}

pub(super) fn steps() -> Vec<Step> {
    vec![
        Step::device(
            "30-mx-master-assignments",
            MX_MASTER,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(0xd7),
        ),
        Step::device(
            "31-light-mx-master-assignments",
            MX_MASTER,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(0xc3),
        )
        .light(),
    ]
}

/// The MX Master 3S as the Logitech driver lists it over a Bolt receiver: only the controls
/// that can be diverted to software (no left / right click), no lighting, one DPI value and no
/// report-rate choice, but a battery.
fn mx_master_3s() -> DeviceSnapshot {
    let ctrl = |usage| ButtonAction::Key {
        usage,
        mods: Modifiers(Modifiers::LCTRL),
    };
    let mouse = ButtonAction::Mouse;
    let buttons = [
        (
            0x52,
            "Middle click",
            mouse(MouseButton::Middle),
            Some((0.50, 0.17)),
        ),
        (0x53, "Back", mouse(MouseButton::Back), Some((0.10, 0.50))),
        (
            0x56,
            "Forward",
            mouse(MouseButton::Forward),
            Some((0.11, 0.40)),
        ),
        (0xc3, "Gesture button", ButtonAction::Default, None),
        (0xc4, "Smart shift", ButtonAction::Default, None),
        (0xd7, "Thumb wheel", ButtonAction::Default, None),
    ]
    .into_iter()
    .map(|(id, name, action, hotspot)| btn(id, name, action, hotspot))
    .collect();
    let mut snap = mouse_like(Spec {
        name: "MX Master 3S",
        vendor: Vendor::Logitech,
        kind: DeviceKind::Mouse,
        pid: 0xb034,
        buttons,
        zones: Vec::new(),
        lighting: Vec::new(),
        custom: vec![
            (0x53, ctrl(0x1d)),
            (0x56, ctrl(0x1c)),
            (0xc3, ButtonAction::Media(media::PLAY_PAUSE)),
        ],
    });
    snap.descriptor.connection = Connection::Wireless;
    snap.descriptor.firmware = Some("RBM21 25.00.0013".into());
    snap.capabilities.dpi = Some(DpiCaps {
        min: 200,
        max: 8000,
        step: 50,
        max_stages: 1,
        separate_xy: false,
    });
    snap.capabilities.report_rates.clear();
    snap.capabilities.onboard_profiles = 0;
    snap.capabilities.battery = true;
    snap.state.dpi = Some(DpiState {
        stages: vec![DpiStage::uniform(1000)],
        active: 0,
    });
    snap.state.report_rate = None;
    snap.state.onboard_profile = None;
    snap.state.battery = Some(Battery {
        percent: Some(84),
        charging: false,
    });
    snap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::illustration::{Design, MouseArt};

    #[test]
    fn mx_master_3s_lists_only_the_divertable_controls() {
        let mx = mx_master_3s();
        assert_eq!(mx.descriptor.model_key, MX_MASTER);
        let ids: Vec<u16> = mx.capabilities.buttons.iter().map(|b| b.id).collect();
        assert_eq!(ids, [0x52, 0x53, 0x56, 0xc3, 0xc4, 0xd7]);
        assert!(mx.capabilities.lighting.is_empty());
        assert!(mx.capabilities.battery && mx.state.battery.is_some());
        assert!(mx.capabilities.report_rates.is_empty());
        // Every control has a part on the drawing, without falling back to a hotspot.
        let design = Design::for_name(&mx.descriptor.name);
        let rect =
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 300.0 / design.aspect()));
        let placed = MouseArt::new(rect, design).place(&mx.capabilities.buttons);
        assert_eq!(placed.len(), 6);
        assert!(placed.iter().all(|p| p.role.is_some()));
    }

    #[test]
    fn steps_only_name_devices_this_file_creates() {
        for step in steps() {
            assert!(step.targets(&devices()), "{}", step.name());
        }
    }
}
