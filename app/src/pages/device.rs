//! Device page: header, onboard profile slots and the capability-driven tabs.

use egui::{Align, Layout, RichText, ScrollArea, Ui, vec2};
use gearwise_core::profile::Library;

use super::PageStates;
use crate::icons::{self, Icon};
use crate::keymap::KeyEv;
use crate::state::{Action, DeviceView, Tab};
use crate::theme::{self, palette, vendor_color, with_alpha};
use crate::widgets::{self, Seg};
use gearwise_core::device::Setting;

pub fn show(
    ui: &mut Ui,
    st: &mut PageStates,
    dev: &DeviceView,
    lib: &Library,
    keys: &[KeyEv],
    tab: &mut Tab,
    out: &mut Vec<Action>,
) {
    let tabs = dev.tabs();
    if !tabs.contains(tab) {
        *tab = tabs[0];
    }
    ui.add_space(4.0);
    header(ui, dev, out);
    ui.add_space(14.0);
    // The slot picker shares the tab row only when there is room for both.
    let roomy = ui.available_width() >= 900.0;
    ui.horizontal(|ui| {
        let items: Vec<Seg<Tab>> = tabs
            .iter()
            .map(|t| {
                Seg::new(*t, t.label()).icon(match t {
                    Tab::Assignments => Icon::Cursor,
                    Tab::Sensitivity => Icon::Gauge,
                    Tab::Lighting => Icon::Sun,
                    Tab::Performance => Icon::Bolt,
                    Tab::Info => Icon::Info,
                })
            })
            .collect();
        if let Some(t) = widgets::segmented(ui, *tab, &items) {
            *tab = t;
        }
        if roomy && dev.caps.onboard_profiles > 0 {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                onboard_slots(ui, dev, out)
            });
        }
    });
    if !roomy && dev.caps.onboard_profiles > 0 {
        ui.add_space(8.0);
        ui.horizontal(|ui| onboard_slots(ui, dev, out));
    }
    ui.add_space(12.0);
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            match *tab {
                Tab::Assignments => {
                    super::assignments::show(ui, &mut st.assign, dev, lib, keys, out)
                }
                Tab::Sensitivity => super::sensitivity::show(ui, &mut st.sens, dev, out),
                Tab::Lighting => super::lighting::show(ui, &mut st.light, dev, out),
                Tab::Performance => super::performance::show(ui, dev, out),
                Tab::Info => super::info::show(ui, dev),
            }
            ui.add_space(24.0);
        });
}

fn header(ui: &mut Ui, dev: &DeviceView, out: &mut Vec<Action>) {
    let pal = palette(ui.ctx());
    let vc = vendor_color(dev.desc.vendor);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(56.0, 56.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(14), with_alpha(vc, 30));
        icons::paint(
            ui.painter(),
            icons::square(rect.center(), 30.0),
            Icon::of(dev.desc.kind),
            vc,
        );
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            widgets::title(ui, dev.display_name());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                widgets::badge(ui, dev.desc.vendor.name(), vc);
                if dev.desc.simulated {
                    widgets::badge(ui, "Demo", pal.muted);
                }
                let mut info = format!("{} · {}", dev.desc.kind.name(), dev.desc.connection.name());
                if let Some(fw) = &dev.desc.firmware {
                    info.push_str(&format!(" · Firmware {fw}"));
                }
                if !dev.online {
                    info.push_str(" · Asleep");
                }
                widgets::muted(ui, info);
            });
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::primary(ui, Some(Icon::Save), "Save to device")
                .on_hover_text("Write the current settings to the device's memory")
                .clicked()
            {
                out.push(Action::SaveOnboard(dev.desc.id.clone()));
            }
            if widgets::secondary(ui, Some(Icon::Layers), "Save to profile")
                .on_hover_text("Store these settings in the active profile")
                .clicked()
            {
                out.push(Action::SaveToProfile(dev.desc.id.clone()));
            }
            if let Some(b) = dev.state.battery {
                widgets::battery_pill(ui, &b);
            }
        });
    });
}

/// Slot pills with a label; the caller's layout decides which side they sit on.
fn onboard_slots(ui: &mut Ui, dev: &DeviceView, out: &mut Vec<Action>) {
    let current = dev.state.onboard_profile.unwrap_or(0);
    let rtl = ui.layout().prefer_right_to_left();
    let muted = theme::palette(ui.ctx()).muted;
    let label = |ui: &mut Ui| {
        ui.label(RichText::new("Onboard profile").color(muted));
    };
    if !rtl {
        label(ui);
    }
    // In a right-to-left row the first item added ends up rightmost, so add the slots backwards.
    let slots: Vec<u8> = if rtl {
        (0..dev.caps.onboard_profiles).rev().collect()
    } else {
        (0..dev.caps.onboard_profiles).collect()
    };
    for slot in slots {
        if widgets::pill(ui, &(slot + 1).to_string(), slot == current)
            .on_hover_text(format!("Onboard profile {}", slot + 1))
            .clicked()
        {
            out.push(Action::Apply {
                dev: dev.desc.id.clone(),
                setting: Setting::OnboardProfile(slot),
                debounce: false,
            });
        }
    }
    if rtl {
        label(ui);
    }
}
