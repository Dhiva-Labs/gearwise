//! Left sidebar: wordmark, device list, navigation and the active-profile switcher.

use egui::{Align, Align2, CornerRadius, FontId, Layout, Popup, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use periph_core::profile::Library;

use crate::icons::{self, Icon};
use crate::state::{Action, DeviceView, Route};
use crate::theme::{palette, vendor_color, with_alpha, Palette};
use crate::widgets;

pub struct Nav<'a> {
    pub devices: &'a [DeviceView],
    pub selected: Option<&'a str>,
    pub route: Route,
    pub lib: &'a Library,
}

pub fn show(ui: &mut Ui, nav: &Nav, out: &mut Vec<Action>) {
    let pal = palette(ui.ctx());
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        profile_switcher(ui, nav, &pal, out);
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            wordmark(ui, &pal);
            ui.add_space(14.0);
            widgets::section(ui, "Devices");
            ui.add_space(2.0);
            if nav.devices.is_empty() {
                widgets::caption(ui, "No devices connected");
            }
            egui::ScrollArea::vertical().max_height((ui.available_height() - 210.0).max(80.0)).auto_shrink([false, true]).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                for d in nav.devices {
                    let selected = nav.route == Route::Device && nav.selected == Some(d.desc.id.as_str());
                    if device_card(ui, d, selected, &pal).clicked() {
                        out.push(Action::SelectDevice(d.desc.id.clone()));
                    }
                }
            });
            ui.add_space(10.0);
            widgets::section(ui, "Library");
            ui.add_space(2.0);
            for (route, icon, label) in [
                (Route::Profiles, Icon::Layers, "Profiles"),
                (Route::Macros, Icon::Macro, "Macros"),
                (Route::Settings, Icon::Gear, "Settings"),
            ] {
                if nav_item(ui, icon, label, nav.route == route, &pal).clicked() {
                    out.push(Action::Navigate(route));
                }
            }
        });
    });
}

fn wordmark(ui: &mut Ui, pal: &Palette) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::same(9), pal.accent);
        // A stylised mouse cut-out in the badge.
        let inner = Rect::from_center_size(rect.center(), vec2(12.0, 18.0));
        p.rect_stroke(inner, CornerRadius::same(6), Stroke::new(1.8, pal.on_accent), StrokeKind::Middle);
        p.vline(rect.center().x, inner.min.y..=inner.min.y + 7.0, Stroke::new(1.5, pal.on_accent));
        ui.add_space(2.0);
        widgets::bold(ui, "Periph Studio", crate::theme::subtitle(), pal.text);
    });
}

fn device_card(ui: &mut Ui, d: &DeviceView, selected: bool, pal: &Palette) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::click());
    let hot = resp.hovered();
    let p = ui.painter();
    if selected {
        p.rect(rect, CornerRadius::same(11), pal.accent_soft, Stroke::new(1.0, pal.accent), StrokeKind::Inside);
    } else {
        p.rect(rect, CornerRadius::same(11), if hot { pal.raised } else { pal.card }, Stroke::new(1.0, pal.border), StrokeKind::Inside);
    }
    let vc = vendor_color(d.desc.vendor);
    let badge = Rect::from_min_size(rect.min + vec2(10.0, 11.0), vec2(40.0, 40.0));
    p.rect_filled(badge, CornerRadius::same(10), with_alpha(vc, if pal.dark { 34 } else { 46 }));
    icons::paint(p, icons::square(badge.center(), 22.0), Icon::of(d.desc.kind), if d.online { vc } else { pal.muted });

    let tx = badge.max.x + 10.0;
    let right_pad = 10.0;
    // Battery sits on the name line, right-aligned; the name truncates around it.
    let battery_w = d.state.battery.map_or(0.0, |b| battery_mini(p, pos2(rect.max.x - right_pad, rect.min.y + 21.0), &b, pal));
    let name_w = rect.max.x - tx - right_pad - if battery_w > 0.0 { battery_w + 6.0 } else { 0.0 };
    let g = widgets::fit_text(p, d.display_name(), FontId::proportional(14.0), pal.text, name_w);
    p.galley(pos2(tx, rect.min.y + 21.0 - g.size().y / 2.0), g, pal.text);

    // Second line: vendor, connection and the demo tag.
    let y = rect.max.y - 17.0;
    let sub = format!("{} · {}", d.desc.vendor.name(), if d.online { d.desc.connection.name() } else { "Asleep" });
    let g = p.layout_no_wrap(sub, FontId::proportional(11.5), pal.muted);
    p.galley(pos2(tx, y - g.size().y / 2.0), g.clone(), pal.muted);
    if d.desc.simulated {
        let g2 = p.layout_no_wrap("DEMO".into(), FontId::proportional(9.5), pal.muted);
        let r = Rect::from_min_size(pos2(tx + g.size().x + 6.0, y - g2.size().y / 2.0 - 2.0), g2.size() + vec2(8.0, 4.0));
        p.rect_filled(r, CornerRadius::same(6), pal.raised);
        p.rect_stroke(r, CornerRadius::same(6), Stroke::new(1.0, pal.border), StrokeKind::Inside);
        p.galley(r.min + vec2(4.0, 2.0), g2, pal.muted);
    }
    resp
}

/// Compact battery readout, right-aligned at `right`; returns its width.
fn battery_mini(p: &egui::Painter, right: egui::Pos2, b: &periph_core::model::Battery, pal: &Palette) -> f32 {
    let pct = b.percent.unwrap_or(0);
    let color = if pct <= 15 { pal.danger } else if pct <= 35 { pal.warning } else { pal.success };
    let text = b.percent.map_or("--".to_string(), |v| format!("{v}%"));
    let g = p.layout_no_wrap(text, FontId::proportional(11.5), pal.text);
    let w = g.size().x + 8.0 + 20.0 + if b.charging { 12.0 } else { 0.0 };
    let r = Rect::from_min_size(pos2(right.x - w, right.y - 10.0), vec2(w, 20.0));
    p.rect(r, CornerRadius::same(10), with_alpha(color, if pal.dark { 30 } else { 40 }), Stroke::NONE, StrokeKind::Inside);
    let mut x = r.min.x + 7.0;
    if b.charging {
        icons::paint(p, icons::square(pos2(x + 4.0, r.center().y), 11.0), Icon::Bolt, pal.warning);
        x += 11.0;
    }
    let body = Rect::from_min_size(pos2(x, r.center().y - 3.5), vec2(11.0, 7.0));
    p.rect_stroke(body, CornerRadius::same(1), Stroke::new(1.0, color), StrokeKind::Inside);
    p.rect_filled(Rect::from_min_size(body.min + vec2(1.5, 1.5), vec2((body.width() - 3.0) * f32::from(pct.min(100)) / 100.0, body.height() - 3.0)), CornerRadius::ZERO, color);
    p.galley(pos2(body.max.x + 5.0, r.center().y - g.size().y / 2.0), g, pal.text);
    w
}

fn nav_item(ui: &mut Ui, icon: Icon, label: &str, selected: bool, pal: &Palette) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(10), pal.accent_soft);
    } else if resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(10), pal.raised);
    }
    let ink = if selected { pal.accent_text() } else if resp.hovered() { pal.text } else { pal.muted };
    icons::paint(p, icons::square(rect.min + vec2(22.0, 19.0), 18.0), icon, ink);
    p.text(rect.min + vec2(42.0, 19.0), Align2::LEFT_CENTER, label, FontId::proportional(14.5), if selected { pal.text } else { ink });
    resp
}

fn profile_switcher(ui: &mut Ui, nav: &Nav, pal: &Palette, out: &mut Vec<Action>) {
    let active = nav.lib.active_profile.as_deref().and_then(|id| nav.lib.profile(id));
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(11), if resp.hovered() { pal.raised } else { pal.card }, Stroke::new(1.0, pal.border), StrokeKind::Inside);
    icons::paint(p, icons::square(rect.min + vec2(22.0, 23.0), 18.0), Icon::User, pal.accent_text());
    p.text(rect.min + vec2(42.0, 14.0), Align2::LEFT_CENTER, "ACTIVE PROFILE", FontId::proportional(9.5), pal.muted);
    let name = active.map_or("None", |a| a.name.as_str());
    let g = widgets::fit_text(p, name, FontId::proportional(14.0), pal.text, rect.width() - 80.0);
    p.galley(pos2(rect.min.x + 42.0, rect.min.y + 22.0), g, pal.text);
    icons::paint(p, icons::square(pos2(rect.max.x - 18.0, rect.center().y), 14.0), Icon::ArrowUp, pal.muted);
    Popup::menu(&resp).width(rect.width()).show(|ui| {
        ui.set_min_width(rect.width() - 16.0);
        if nav.lib.profiles.is_empty() {
            widgets::caption(ui, "No profiles yet");
        }
        for pr in &nav.lib.profiles {
            let is_active = nav.lib.active_profile.as_deref() == Some(pr.id.as_str());
            let (r, rr) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
            if rr.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(8), pal.raised);
            }
            ui.painter().text(r.min + vec2(10.0, 16.0), Align2::LEFT_CENTER, &pr.name, FontId::proportional(14.0), if is_active { pal.accent_text() } else { pal.text });
            if is_active {
                icons::paint(ui.painter(), icons::square(r.max - vec2(18.0, 16.0), 14.0), Icon::Check, pal.accent_text());
            }
            if rr.clicked() {
                out.push(Action::ActivateProfile(pr.id.clone()));
                ui.close();
            }
        }
    });
    ui.add_space(8.0);
}
