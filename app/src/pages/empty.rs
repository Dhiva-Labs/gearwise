//! Shown when no device is connected and demo devices are off.

use egui::{Align, Layout, Stroke, Ui, vec2};

use crate::icons::{self, Icon};
use crate::state::Action;
use crate::theme::{palette, with_alpha};
use crate::widgets;

pub fn show(ui: &mut Ui, out: &mut Vec<Action>) {
    let pal = palette(ui.ctx());
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space((ui.available_height() * 0.2).max(24.0));
        let (rect, _) = ui.allocate_exact_size(vec2(260.0, 170.0), egui::Sense::hover());
        let c = rect.center();
        let p = ui.painter();
        for (r, a) in [(84.0, 14u8), (62.0, 22), (42.0, 34)] {
            p.circle_filled(c, r, with_alpha(pal.accent, a));
        }
        p.circle_stroke(c, 84.0, Stroke::new(1.0, with_alpha(pal.accent, 70)));
        icons::paint(p, icons::square(c, 52.0), Icon::Mouse, pal.accent);
        icons::paint(p, icons::square(c + vec2(-92.0, 28.0), 34.0), Icon::Keyboard, with_alpha(pal.muted, 190));
        icons::paint(p, icons::square(c + vec2(92.0, 24.0), 32.0), Icon::Headset, with_alpha(pal.muted, 190));
        ui.add_space(8.0);
        widgets::bold(ui, "Connect a device", crate::theme::title(), pal.text);
        ui.add_space(4.0);
        ui.set_max_width(440.0);
        ui.label(egui::RichText::new("Plug in a supported Logitech, Razer or Corsair mouse or keyboard and it will show up here on its own.").color(pal.muted));
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(((ui.available_width() - 290.0) / 2.0).max(0.0));
            if widgets::primary(ui, Some(Icon::Play), "Show demo devices").clicked() {
                out.push(Action::SetDemo(true));
            }
            if widgets::secondary(ui, None, "Scan again").clicked() {
                out.push(Action::Rescan);
            }
        });
    });
}
