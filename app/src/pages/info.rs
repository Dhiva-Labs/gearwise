//! Info tab: what the device is and what the app can do with it.

use egui::Ui;
use gearwise_core::model::ButtonAction;

use crate::state::DeviceView;
use crate::widgets;

pub fn show(ui: &mut Ui, dev: &DeviceView) {
    let d = &dev.desc;
    widgets::card_columns(
        ui,
        0.5,
        |ui| {
            widgets::subtitle(ui, "Device");
            ui.add_space(4.0);
            widgets::kv(ui, "Name", &d.name);
            widgets::kv(ui, "Vendor", d.vendor.name());
            widgets::kv(ui, "Type", d.kind.name());
            widgets::kv(
                ui,
                "USB ID",
                &format!("{:04x}:{:04x}", d.vendor_id, d.product_id),
            );
            widgets::kv(ui, "Model key", &d.model_key);
            widgets::kv(ui, "Serial", d.serial.as_deref().unwrap_or("Not reported"));
            widgets::kv(
                ui,
                "Firmware",
                d.firmware.as_deref().unwrap_or("Not reported"),
            );
            widgets::kv(ui, "Connection", d.connection.name());
            if d.simulated {
                widgets::kv(ui, "Source", "Simulated (demo)");
            }
        },
        |ui| {
            widgets::subtitle(ui, "Capabilities");
            ui.add_space(4.0);
            for (k, v) in capability_rows(dev) {
                widgets::kv(ui, k, &v);
            }
        },
    );
}

/// One line per feature group, "None" where the device lacks it.
pub fn capability_rows(dev: &DeviceView) -> Vec<(&'static str, String)> {
    let c = &dev.caps;
    let yes_no = |b: bool| if b { "Yes" } else { "No" }.to_string();
    let dpi = c.dpi.as_ref().map_or("None".to_string(), |d| {
        format!(
            "{}–{} DPI, step {}, up to {} stages{}",
            d.min,
            d.max,
            d.step,
            d.max_stages,
            if d.separate_xy { ", separate X/Y" } else { "" }
        )
    });
    let rates = if c.report_rates.is_empty() {
        "None".to_string()
    } else {
        c.report_rates
            .iter()
            .map(|r| format!("{r}"))
            .collect::<Vec<_>>()
            .join(", ")
            + " Hz"
    };
    let lighting = if c.lighting.is_empty() {
        "None".to_string()
    } else {
        c.lighting
            .iter()
            .map(|z| z.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let per_key = c.per_key.as_ref().map_or("No".to_string(), |l| {
        format!("{} ({} keys)", l.name, l.keys.len())
    });
    let remappable = c.buttons.iter().filter(|b| b.remappable).count();
    let customised = dev
        .state
        .buttons
        .values()
        .filter(|a| **a != ButtonAction::Default)
        .count();
    vec![
        ("DPI", dpi),
        ("Report rates", rates),
        ("Lighting zones", lighting),
        ("Per-key RGB", per_key),
        (
            "Buttons",
            format!(
                "{} ({remappable} remappable, {customised} customised)",
                c.buttons.len()
            ),
        ),
        (
            "Onboard profiles",
            if c.onboard_profiles == 0 {
                "None".to_string()
            } else {
                c.onboard_profiles.to_string()
            },
        ),
        ("Battery", yes_no(c.battery)),
        (
            "Lift-off distance",
            if c.lift_off.is_empty() {
                "No".to_string()
            } else {
                c.lift_off.join(", ")
            },
        ),
        ("Angle snapping", yes_no(c.angle_snapping)),
        ("Game mode", yes_no(c.game_mode)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker::DeviceSnapshot;

    fn view(i: usize) -> DeviceView {
        let mut d = gearwise_core::demo::devices().remove(i);
        let state = d.read_state().unwrap();
        DeviceView::new(DeviceSnapshot {
            descriptor: d.descriptor().clone(),
            capabilities: d.capabilities().clone(),
            state,
        })
    }

    #[test]
    fn summary_reflects_capabilities() {
        let mouse = capability_rows(&view(0));
        let get = |rows: &[(&str, String)], k: &str| {
            rows.iter().find(|(n, _)| *n == k).unwrap().1.clone()
        };
        assert!(get(&mouse, "DPI").starts_with("100–25600 DPI"));
        assert_eq!(get(&mouse, "Per-key RGB"), "No");
        assert_eq!(get(&mouse, "Battery"), "Yes");
        let kb = capability_rows(&view(1));
        assert!(get(&kb, "Per-key RGB").contains("104 keys"));
        assert_eq!(get(&kb, "DPI"), "None");
        assert_eq!(get(&kb, "Game mode"), "Yes");
    }
}
