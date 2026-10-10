//! Performance tab: polling rate, lift-off distance, angle snapping, game mode.

use egui::{CornerRadius, FontId, Sense, Stroke, StrokeKind, Ui, vec2};
use periph_core::device::Setting;

use crate::state::{Action, DeviceView};
use crate::theme::{mix, palette};
use crate::widgets::{self, Seg};

/// "8K" style label for rates of 1000 Hz and up, plain number below.
pub fn rate_label(hz: u16) -> String {
    if hz >= 1000 && hz.is_multiple_of(1000) { format!("{}K", hz / 1000) } else { hz.to_string() }
}

pub fn show(ui: &mut Ui, dev: &DeviceView, out: &mut Vec<Action>) {
    let pal = palette(ui.ctx());
    let id = &dev.desc.id;
    let send = |out: &mut Vec<Action>, s: Setting| out.push(Action::Apply { dev: id.clone(), setting: s, debounce: false });

    if !dev.caps.report_rates.is_empty() {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::subtitle(ui, "Report rate");
            widgets::muted(ui, "How often the device reports to the computer. Higher is smoother and uses more CPU.");
            ui.add_space(8.0);
            let current = dev.state.report_rate;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                for &hz in &dev.caps.report_rates {
                    let selected = current == Some(hz);
                    let (rect, resp) = ui.allocate_exact_size(vec2(84.0, 64.0), Sense::click());
                    let hot = resp.hovered();
                    let (fill, stroke) = if selected {
                        (pal.accent_soft, Stroke::new(1.5, pal.accent))
                    } else if hot {
                        (mix(pal.raised, pal.accent, 0.1), Stroke::new(1.0, mix(pal.border, pal.accent, 0.5)))
                    } else {
                        (pal.raised, Stroke::new(1.0, pal.border))
                    };
                    ui.painter().rect(rect, CornerRadius::same(10), fill, stroke, StrokeKind::Inside);
                    let ink = if selected { pal.accent_text() } else { pal.text };
                    ui.painter().text(rect.center() - vec2(0.0, 7.0), egui::Align2::CENTER_CENTER, rate_label(hz), FontId::proportional(22.0), ink);
                    ui.painter().text(rect.center() + vec2(0.0, 17.0), egui::Align2::CENTER_CENTER, "Hz", FontId::proportional(11.5), pal.muted);
                    if resp.on_hover_text(format!("{hz} Hz")).clicked() && !selected {
                        send(out, Setting::ReportRate(hz));
                    }
                }
            });
        });
        ui.add_space(14.0);
    }

    let has_switches = dev.caps.angle_snapping || dev.caps.game_mode;
    if !dev.caps.lift_off.is_empty() {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::subtitle(ui, "Lift-off distance");
            widgets::muted(ui, "How far the mouse can rise before the sensor stops tracking.");
            ui.add_space(8.0);
            let current = dev.state.lift_off.unwrap_or(0);
            let items: Vec<Seg<u8>> = dev.caps.lift_off.iter().enumerate().map(|(i, l)| Seg::new(i as u8, l.clone())).collect();
            if let Some(v) = widgets::segmented(ui, current, &items) {
                send(out, Setting::LiftOff(v));
            }
        });
        ui.add_space(14.0);
    }

    if has_switches {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            if dev.caps.angle_snapping {
                let mut on = dev.state.angle_snapping.unwrap_or(false);
                widgets::setting_row(ui, "Angle snapping", "Straightens small deviations in diagonal movement. Helps drawing, hurts aim.", |ui| {
                    if widgets::toggle(ui, &mut on) {
                        send(out, Setting::AngleSnapping(on));
                    }
                });
            }
            if dev.caps.angle_snapping && dev.caps.game_mode {
                ui.add_space(6.0);
                widgets::divider(ui);
                ui.add_space(6.0);
            }
            if dev.caps.game_mode {
                let mut on = dev.state.game_mode.unwrap_or(false);
                widgets::setting_row(ui, "Game mode", "Disables the Super key and shortcuts like Alt+Tab while you play.", |ui| {
                    if widgets::toggle(ui, &mut on) {
                        send(out, Setting::GameMode(on));
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::rate_label;

    #[test]
    fn high_rates_get_k_labels() {
        assert_eq!(rate_label(125), "125");
        assert_eq!(rate_label(500), "500");
        assert_eq!(rate_label(1000), "1K");
        assert_eq!(rate_label(2000), "2K");
        assert_eq!(rate_label(8000), "8K");
    }
}
