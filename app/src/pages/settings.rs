//! Settings page: appearance, devices, permissions help and About.

use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{FontId, RichText, TextEdit, Ui, vec2};
use periph_core::profile::{Library, Theme};

use crate::icons::Icon;
use crate::state::Action;
use crate::theme::{self, ACCENTS, palette, parse_hex, to_hex};
use crate::widgets::{self, Seg};

pub const UDEV_COMMAND: &str = "sudo packaging/linux/install-udev-rules.sh";

#[derive(Default)]
pub struct SettingsState {
    hex: String,
}

pub fn show(ui: &mut Ui, st: &mut SettingsState, lib: &mut Library, out: &mut Vec<Action>) {
    let pal = palette(ui.ctx());
    ui.add_space(4.0);
    widgets::title(ui, "Settings");
    ui.add_space(14.0);

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::subtitle(ui, "Appearance");
        ui.add_space(8.0);
        widgets::section(ui, "Theme");
        let items = [
            Seg::new(Theme::Dark, "Dark").icon(Icon::Moon),
            Seg::new(Theme::Light, "Light").icon(Icon::Sun),
        ];
        if let Some(t) = widgets::segmented(ui, lib.prefs.theme, &items) {
            lib.prefs.theme = t;
        }
        ui.add_space(12.0);
        widgets::section(ui, "Accent colour");
        let current = theme::parse_hex(&lib.prefs.accent).unwrap_or(pal.accent);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
            for (name, hex) in ACCENTS {
                let c = parse_hex(hex).unwrap_or(pal.accent);
                if widgets::swatch(ui, c, 32.0, c == current)
                    .on_hover_text(name)
                    .clicked()
                {
                    lib.prefs.accent = hex.to_string();
                }
            }
            ui.add_space(8.0);
            let mut custom = current;
            if color_edit_button_srgba(ui, &mut custom, Alpha::Opaque)
                .on_hover_text("Custom colour")
                .changed()
            {
                lib.prefs.accent = to_hex(custom);
            }
            let editing = ui.memory(|m| m.has_focus(ui.id().with("accent-hex")));
            if !editing {
                st.hex = to_hex(current);
            }
            let r = ui.add(
                TextEdit::singleline(&mut st.hex)
                    .id(ui.id().with("accent-hex"))
                    .desired_width(82.0)
                    .font(FontId::monospace(13.0)),
            );
            if r.changed()
                && let Some(c) = parse_hex(&st.hex)
            {
                lib.prefs.accent = to_hex(c);
            }
        });
    });
    ui.add_space(14.0);

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::subtitle(ui, "Devices");
        ui.add_space(6.0);
        let mut demo = lib.prefs.demo_devices;
        widgets::setting_row(
            ui,
            "Show demo devices",
            "Simulated devices to explore the app without hardware. They never touch real hardware.",
            |ui| {
                if widgets::toggle(ui, &mut demo) {
                    out.push(Action::SetDemo(demo));
                }
            },
        );
    });
    ui.add_space(14.0);

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::subtitle(ui, "Permissions");
        ui.add_space(4.0);
        let linux = cfg!(target_os = "linux");
        ui.label(
            RichText::new(if linux {
                "Linux"
            } else {
                "Linux (for reference)"
            })
            .color(pal.text)
            .strong(),
        );
        widgets::muted(
            ui,
            "Devices need a udev rule so your user can talk to them without root. Run this once, then unplug and reconnect the device:",
        );
        ui.horizontal(|ui| {
            code_box(ui, UDEV_COMMAND);
            if widgets::secondary(ui, Some(Icon::Copy), "Copy").clicked() {
                out.push(Action::CopyText(UDEV_COMMAND.to_string()));
                out.push(Action::Toast(
                    crate::state::ToastKind::Success,
                    "Command copied".into(),
                ));
            }
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new(if cfg!(windows) {
                "Windows"
            } else {
                "Windows (for reference)"
            })
            .color(pal.text)
            .strong(),
        );
        widgets::muted(
            ui,
            "Close Logitech G HUB, Razer Synapse and Corsair iCUE. They hold exclusive access to the device while they run.",
        );
    });
    ui.add_space(14.0);

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::subtitle(ui, "About");
        ui.add_space(4.0);
        widgets::kv(ui, "Version", env!("CARGO_PKG_VERSION"));
        widgets::kv(ui, "License", "MIT");
        widgets::kv(
            ui,
            "Library file",
            &periph_core::profile::default_library_path()
                .display()
                .to_string(),
        );
        ui.add_space(4.0);
        widgets::muted(
            ui,
            "Periph Studio is an open, lightweight way to configure Logitech, Razer and Corsair mice and keyboards.",
        );
    });
    ui.add_space(24.0);
}

pub fn code_box(ui: &mut Ui, text: &str) {
    let pal = palette(ui.ctx());
    egui::Frame::new()
        .fill(pal.input)
        .stroke(egui::Stroke::new(1.0, pal.border))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 7))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .font(FontId::monospace(13.0))
                    .color(pal.text),
            );
        });
}
