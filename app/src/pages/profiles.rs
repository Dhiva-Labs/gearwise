//! Profiles page: create and manage named sets of device settings.

use std::path::PathBuf;

use egui::{
    Align, CornerRadius, FontId, Layout, Modal, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui,
    pos2, vec2,
};
use periph_core::profile::{Library, Profile, new_id};

use crate::icons::Icon;
use crate::state::{Action, DeviceView, ToastKind};
use crate::theme::palette;
use crate::widgets::{self, Btn};

#[derive(Default)]
pub struct ProfilesState {
    selected: Option<String>,
    renaming: Option<String>,
    rename_buf: String,
    confirm_delete: Option<String>,
    app_input: String,
    path: String,
    path_for: Option<String>,
}

/// Default location offered for export/import.
fn default_path(name: &str) -> String {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let file: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    home.join(format!("{file}.periph-profile.json"))
        .to_string_lossy()
        .into_owned()
}

/// Clone of `src` with a new id and a name that does not clash with `existing`.
pub fn duplicate(src: &Profile, existing: &[Profile], suffix: &str) -> Profile {
    let mut p = src.clone();
    p.id = new_id("p");
    let base = format!("{}{suffix}", src.name);
    let mut name = base.clone();
    let mut n = 2;
    while existing.iter().any(|e| e.name == name) {
        name = format!("{base} {n}");
        n += 1;
    }
    p.name = name;
    p
}

pub fn show(
    ui: &mut Ui,
    st: &mut ProfilesState,
    lib: &mut Library,
    devices: &[DeviceView],
    out: &mut Vec<Action>,
) {
    if st
        .selected
        .as_deref()
        .is_none_or(|id| lib.profile(id).is_none())
    {
        st.selected = lib
            .active_profile
            .clone()
            .filter(|id| lib.profile(id).is_some())
            .or_else(|| lib.profiles.first().map(|p| p.id.clone()));
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        widgets::title(ui, "Profiles");
    });
    widgets::muted(
        ui,
        "A profile remembers button, DPI, lighting and performance settings for each device model.",
    );
    ui.add_space(14.0);

    let total = ui.available_width();
    let list_w = 270.0;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        ui.allocate_ui_with_layout(vec2(list_w, 10.0), Layout::top_down(Align::Min), |ui| {
            widgets::card(ui, |ui| {
                ui.set_width(list_w - 34.0);
                list(ui, st, lib, out);
            });
        });
        let w = total - list_w - 16.0;
        ui.allocate_ui_with_layout(vec2(w, 10.0), Layout::top_down(Align::Min), |ui| {
            detail(ui, st, lib, devices, w, out);
        });
    });
    delete_dialog(ui, st, lib);
}

fn list(ui: &mut Ui, st: &mut ProfilesState, lib: &mut Library, out: &mut Vec<Action>) {
    let pal = palette(ui.ctx());
    ui.horizontal(|ui| {
        widgets::subtitle(ui, "Your profiles");
    });
    ui.add_space(4.0);
    ui.spacing_mut().item_spacing.y = 4.0;
    let active = lib.active_profile.clone();
    let mut commit_rename = None;
    for p in &lib.profiles {
        let selected = st.selected.as_deref() == Some(p.id.as_str());
        if st.renaming.as_deref() == Some(p.id.as_str()) {
            let r = ui.add(
                TextEdit::singleline(&mut st.rename_buf)
                    .desired_width(ui.available_width())
                    .font(FontId::proportional(14.0)),
            );
            if !r.has_focus() && !r.gained_focus() {
                r.request_focus();
            }
            if r.lost_focus() {
                commit_rename =
                    Some((p.id.clone(), ui.input(|i| i.key_pressed(egui::Key::Escape))));
            }
            continue;
        }
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
        let painter = ui.painter();
        if selected {
            painter.rect(
                rect,
                CornerRadius::same(9),
                pal.accent_soft,
                Stroke::new(1.0, pal.accent),
                StrokeKind::Inside,
            );
        } else if resp.hovered() {
            painter.rect_filled(rect, CornerRadius::same(9), pal.raised);
        }
        let g = widgets::fit_text(
            painter,
            &p.name,
            FontId::proportional(14.0),
            pal.text,
            rect.width() - 90.0,
        );
        painter.galley(
            pos2(rect.min.x + 12.0, rect.center().y - g.size().y / 2.0),
            g,
            pal.text,
        );
        if active.as_deref() == Some(p.id.as_str()) {
            painter.text(
                pos2(rect.max.x - 12.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                "ACTIVE",
                FontId::proportional(10.5),
                pal.accent_text(),
            );
        }
        if resp.clicked() {
            st.selected = Some(p.id.clone());
        }
        if resp.double_clicked() {
            st.renaming = Some(p.id.clone());
            st.rename_buf = p.name.clone();
        }
    }
    if let Some((id, cancelled)) = commit_rename {
        let name = st.rename_buf.trim().to_string();
        if !cancelled
            && !name.is_empty()
            && let Some(p) = lib.profile_mut(&id)
        {
            p.name = name;
        }
        st.renaming = None;
    }
    ui.add_space(8.0);
    if widgets::primary(ui, Some(Icon::Plus), "New profile").clicked() {
        let p = Profile::new(unique_name(&lib.profiles, "New profile"));
        st.selected = Some(p.id.clone());
        st.renaming = Some(p.id.clone());
        st.rename_buf = p.name.clone();
        if lib.active_profile.is_none() {
            lib.active_profile = Some(p.id.clone());
        }
        lib.profiles.push(p);
        out.push(Action::Toast(ToastKind::Success, "Profile created".into()));
    }
}

fn unique_name(existing: &[Profile], base: &str) -> String {
    let mut name = base.to_string();
    let mut n = 2;
    while existing.iter().any(|p| p.name == name) {
        name = format!("{base} {n}");
        n += 1;
    }
    name
}

fn detail(
    ui: &mut Ui,
    st: &mut ProfilesState,
    lib: &mut Library,
    devices: &[DeviceView],
    w: f32,
    out: &mut Vec<Action>,
) {
    let Some(id) = st.selected.clone() else {
        widgets::card(ui, |ui| {
            ui.set_width(w - 34.0);
            widgets::empty_hint(ui, Icon::Layers, "Create a profile to get started");
        });
        return;
    };
    let is_active = lib.active_profile.as_deref() == Some(id.as_str());
    let Some(profile) = lib.profile(&id).cloned() else {
        return;
    };

    widgets::card(ui, |ui| {
        ui.set_width(w - 34.0);
        ui.horizontal(|ui| {
            widgets::title(ui, &profile.name);
            if is_active {
                widgets::badge(ui, "Active", palette(ui.ctx()).accent);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::icon_button(ui, Icon::Trash, "Delete profile").clicked() {
                    st.confirm_delete = Some(id.clone());
                }
                if widgets::icon_button(ui, Icon::Copy, "Duplicate").clicked() {
                    let copy = duplicate(&profile, &lib.profiles, " copy");
                    st.selected = Some(copy.id.clone());
                    lib.profiles.push(copy);
                }
                if widgets::icon_button(ui, Icon::Edit, "Rename").clicked() {
                    st.renaming = Some(id.clone());
                    st.rename_buf = profile.name.clone();
                }
            });
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled_ui(!is_active, |ui| {
                    widgets::primary(ui, Some(Icon::Check), "Set active")
                })
                .inner
                .clicked()
            {
                out.push(Action::ActivateProfile(id.clone()));
            }
            if widgets::secondary(ui, Some(Icon::Play), "Apply now")
                .on_hover_text("Send this profile's settings to connected devices")
                .clicked()
            {
                out.push(Action::ApplyProfile(id.clone()));
            }
        });
    });
    ui.add_space(14.0);

    // Devices configured by the profile.
    widgets::card(ui, |ui| {
        ui.set_width(w - 34.0);
        widgets::subtitle(ui, "Devices");
        ui.add_space(4.0);
        if profile.devices.is_empty() {
            widgets::muted(
                ui,
                "Nothing saved yet. Open a device and press “Save to profile”.",
            );
        }
        let mut remove = None;
        for (model, settings) in &profile.devices {
            let connected: Vec<&DeviceView> = devices
                .iter()
                .filter(|d| d.desc.model_key == *model)
                .collect();
            ui.horizontal(|ui| {
                let name = connected
                    .first()
                    .map_or_else(|| model.clone(), |d| d.display_name().to_string());
                ui.label(RichText::new(name).color(palette(ui.ctx()).text));
                widgets::caption(ui, format!("{} settings", settings.len()));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::icon_button(ui, Icon::Close, "Remove from profile").clicked() {
                        remove = Some(model.clone());
                    }
                    for d in &connected {
                        if widgets::secondary(ui, None, "Apply")
                            .on_hover_text(format!("Apply to {}", d.desc.name))
                            .clicked()
                        {
                            out.push(Action::ApplyMany {
                                dev: d.desc.id.clone(),
                                settings: settings.clone(),
                            });
                        }
                    }
                });
            });
        }
        if let Some(model) = remove
            && let Some(p) = lib.profile_mut(&id)
        {
            p.devices.remove(&model);
        }
    });
    ui.add_space(14.0);

    linked_apps(ui, st, lib, &id, &profile, w);
    ui.add_space(14.0);
    transfer(ui, st, lib, &id, &profile, w, out);
}

fn linked_apps(
    ui: &mut Ui,
    st: &mut ProfilesState,
    lib: &mut Library,
    id: &str,
    profile: &Profile,
    w: f32,
) {
    let pal = palette(ui.ctx());
    widgets::card(ui, |ui| {
        ui.set_width(w - 34.0);
        widgets::subtitle(ui, "Linked applications");
        widgets::muted(
            ui,
            "Switch to this profile when one of these programs is in the foreground.",
        );
        ui.add_space(6.0);
        let mut remove = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for (i, app) in profile.apps.iter().enumerate() {
                let g =
                    ui.painter()
                        .layout_no_wrap(app.clone(), FontId::proportional(13.0), pal.text);
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(g.size().x + 40.0, 28.0), Sense::click());
                ui.painter().rect(
                    rect,
                    CornerRadius::same(14),
                    pal.raised,
                    Stroke::new(1.0, pal.border),
                    StrokeKind::Inside,
                );
                ui.painter()
                    .galley(rect.min + vec2(12.0, 14.0 - g.size().y / 2.0), g, pal.text);
                crate::icons::paint(
                    ui.painter(),
                    crate::icons::square(pos2(rect.max.x - 16.0, rect.center().y), 12.0),
                    Icon::Close,
                    if resp.hovered() {
                        pal.danger
                    } else {
                        pal.muted
                    },
                );
                if resp.on_hover_text("Remove").clicked() {
                    remove = Some(i);
                }
            }
            if profile.apps.is_empty() {
                widgets::caption(ui, "No programs linked");
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let r = ui.add(
                TextEdit::singleline(&mut st.app_input)
                    .hint_text("game.exe")
                    .desired_width(220.0),
            );
            let add = widgets::secondary(ui, Some(Icon::Plus), "Add").clicked()
                || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if add {
                let name = st.app_input.trim().to_string();
                if !name.is_empty() {
                    if let Some(p) = lib.profile_mut(id)
                        && !p.apps.iter().any(|a| a.eq_ignore_ascii_case(&name))
                    {
                        p.apps.push(name);
                    }
                    st.app_input.clear();
                }
            }
        });
        if let Some(i) = remove
            && let Some(p) = lib.profile_mut(id)
        {
            p.apps.remove(i);
        }
        ui.add_space(8.0);
        widgets::divider(ui);
        ui.add_space(8.0);
        let mut auto = lib.prefs.auto_switch;
        widgets::setting_row(
            ui,
            "Switch profiles automatically",
            "Follows the focused program. Focus detection needs X11 or Windows; on Wayland switch by hand.",
            |ui| {
                if widgets::toggle(ui, &mut auto) {
                    lib.prefs.auto_switch = auto;
                }
            },
        );
    });
}

fn transfer(
    ui: &mut Ui,
    st: &mut ProfilesState,
    lib: &mut Library,
    id: &str,
    profile: &Profile,
    w: f32,
    out: &mut Vec<Action>,
) {
    if st.path_for.as_deref() != Some(id) {
        st.path_for = Some(id.to_string());
        st.path = default_path(&profile.name);
    }
    widgets::card(ui, |ui| {
        ui.set_width(w - 34.0);
        widgets::subtitle(ui, "Import and export");
        widgets::muted(
            ui,
            "Share a profile as a JSON file. Type or paste the file path.",
        );
        ui.add_space(6.0);
        ui.add(
            TextEdit::singleline(&mut st.path)
                .desired_width(ui.available_width())
                .font(FontId::monospace(13.0)),
        );
        ui.horizontal(|ui| {
            if widgets::secondary(ui, Some(Icon::Upload), "Export").clicked() {
                let result = serde_json::to_string_pretty(profile)
                    .map_err(|e| e.to_string())
                    .and_then(|s| std::fs::write(&st.path, s).map_err(|e| e.to_string()));
                out.push(match result {
                    Ok(()) => Action::Toast(ToastKind::Success, format!("Exported to {}", st.path)),
                    Err(e) => Action::Toast(ToastKind::Error, format!("Export failed: {e}")),
                });
            }
            if widgets::secondary(ui, Some(Icon::Download), "Import").clicked() {
                let result = std::fs::read_to_string(&st.path)
                    .map_err(|e| e.to_string())
                    .and_then(|s| serde_json::from_str::<Profile>(&s).map_err(|e| e.to_string()));
                match result {
                    Ok(p) => {
                        let p = duplicate(&p, &lib.profiles, "");
                        out.push(Action::Toast(
                            ToastKind::Success,
                            format!("Imported “{}”", p.name),
                        ));
                        st.selected = Some(p.id.clone());
                        lib.profiles.push(p);
                    }
                    Err(e) => out.push(Action::Toast(
                        ToastKind::Error,
                        format!("Import failed: {e}"),
                    )),
                }
            }
        });
    });
}

fn delete_dialog(ui: &mut Ui, st: &mut ProfilesState, lib: &mut Library) {
    let Some(id) = st.confirm_delete.clone() else {
        return;
    };
    let Some(name) = lib.profile(&id).map(|p| p.name.clone()) else {
        st.confirm_delete = None;
        return;
    };
    let mut close = false;
    let modal = Modal::new(egui::Id::new("delete-profile")).show(ui.ctx(), |ui| {
        ui.set_width(340.0);
        widgets::subtitle(ui, "Delete profile?");
        ui.add_space(6.0);
        ui.label(format!("“{name}” and the settings saved in it will be removed. Devices keep their current settings."));
        ui.add_space(12.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::button(ui, Btn::Danger, Some(Icon::Trash), "Delete").clicked() {
                lib.profiles.retain(|p| p.id != id);
                if lib.active_profile.as_deref() == Some(id.as_str()) {
                    lib.active_profile = lib.profiles.first().map(|p| p.id.clone());
                }
                st.selected = None;
                close = true;
            }
            if widgets::secondary(ui, None, "Cancel").clicked() {
                close = true;
            }
        });
    });
    if close || modal.should_close() {
        st.confirm_delete = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicates_get_fresh_ids_and_unique_names() {
        let a = Profile::new("FPS");
        let mut all = vec![a.clone()];
        let b = duplicate(&a, &all, " copy");
        assert_ne!(a.id, b.id);
        assert_eq!(b.name, "FPS copy");
        all.push(b);
        assert_eq!(duplicate(&a, &all, " copy").name, "FPS copy 2");
    }

    #[test]
    fn new_profile_names_do_not_clash() {
        let ps = vec![Profile::new("New profile"), Profile::new("New profile 2")];
        assert_eq!(unique_name(&ps, "New profile"), "New profile 3");
        assert_eq!(unique_name(&[], "New profile"), "New profile");
    }
}
