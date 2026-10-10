//! Macros page: build, record and assign sequences of key and mouse actions.

use std::cell::RefCell;
use std::time::Instant;

use egui::{
    Align, CornerRadius, DragValue, Frame, Layout, Margin, Modal, Popup, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, TextEdit, Ui, pos2, vec2,
};
use periph_core::device::Setting;
use periph_core::model::{ButtonAction, MouseButton};
use periph_core::profile::{Library, Macro, MacroMode, MacroStep};

use crate::icons::{self, Icon};
use crate::keymap::{Chord, KeyEv};
use crate::state::{Action, DeviceView};
use crate::theme::{palette, with_alpha};
use crate::widgets::{self, Btn, Seg};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Capture {
    Tap,
    Down,
    Up,
}

struct Recording {
    last: Option<Instant>,
}

pub struct MacrosState {
    pub selected: Option<String>,
    recording: Option<Recording>,
    collapse: bool,
    capture: Option<Capture>,
    chord: Chord,
    confirm_delete: Option<String>,
}

impl Default for MacrosState {
    fn default() -> Self {
        Self {
            selected: None,
            recording: None,
            collapse: true,
            capture: None,
            chord: Chord::default(),
            confirm_delete: None,
        }
    }
}

impl MacrosState {
    pub fn wants_keys(&self) -> bool {
        self.recording.is_some() || self.capture.is_some()
    }

    pub fn stop_all(&mut self) {
        self.recording = None;
        self.capture = None;
        self.chord = Chord::default();
    }
}

/// Append a recorded key transition, inserting the wait since the previous one.
/// With `collapse`, gaps under 10 ms are dropped.
pub fn record_event(
    steps: &mut Vec<MacroStep>,
    last: &mut Option<Instant>,
    ev: &KeyEv,
    collapse: bool,
) {
    if let Some(prev) = *last {
        let ms =
            u32::try_from(ev.at.saturating_duration_since(prev).as_millis()).unwrap_or(u32::MAX);
        if ms > 0 && !(collapse && ms < 10) {
            steps.push(MacroStep::Delay(ms));
        }
    }
    steps.push(if ev.pressed {
        MacroStep::KeyDown(ev.usage)
    } else {
        MacroStep::KeyUp(ev.usage)
    });
    *last = Some(ev.at);
}

/// Where a macro is assigned: device buttons right now, and saved profiles.
pub fn usage(lib: &Library, devices: &[DeviceView], macro_id: &str) -> Vec<String> {
    let mut out = Vec::new();
    for d in devices {
        for (btn, action) in &d.state.buttons {
            if matches!(action, ButtonAction::Macro(id) if id == macro_id) {
                let name = d
                    .caps
                    .buttons
                    .iter()
                    .find(|b| b.id == *btn)
                    .map_or_else(|| format!("Button {btn}"), |b| b.name.clone());
                out.push(format!("{name} on {}", d.display_name()));
            }
        }
    }
    for p in &lib.profiles {
        for (model, settings) in &p.devices {
            let n = settings.iter().filter(|s| matches!(s, Setting::Button { action: ButtonAction::Macro(id), .. } if id == macro_id)).count();
            if n > 0 {
                out.push(format!(
                    "Profile “{}” ({model}, {n} button{})",
                    p.name,
                    if n == 1 { "" } else { "s" }
                ));
            }
        }
    }
    out
}

fn format_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.1} s", ms as f64 / 1000.0)
    }
}

pub fn show(
    ui: &mut Ui,
    st: &mut MacrosState,
    lib: &mut Library,
    devices: &[DeviceView],
    keys: &[KeyEv],
    out: &mut Vec<Action>,
) {
    if st
        .selected
        .as_deref()
        .is_none_or(|id| lib.macro_by_id(id).is_none())
    {
        st.selected = lib.macros.first().map(|m| m.id.clone());
        st.stop_all();
    }
    consume_keys(st, lib, keys);

    ui.add_space(4.0);
    widgets::title(ui, "Macros");
    widgets::muted(
        ui,
        "Record or build a sequence, then assign it to a button from a device's Assignments tab.",
    );
    ui.add_space(14.0);

    let st_cell = RefCell::new(st);
    let lib_cell = RefCell::new(lib);
    widgets::card_columns(
        ui,
        0.3,
        |ui| list(ui, &mut st_cell.borrow_mut(), &mut lib_cell.borrow_mut()),
        |ui| {
            editor(
                ui,
                &mut st_cell.borrow_mut(),
                &mut lib_cell.borrow_mut(),
                devices,
                out,
            )
        },
    );
    let (st, lib) = (st_cell.into_inner(), lib_cell.into_inner());
    delete_dialog(ui, st, lib);
}

/// Turn this frame's key transitions into recorded or captured steps.
fn consume_keys(st: &mut MacrosState, lib: &mut Library, keys: &[KeyEv]) {
    let Some(id) = st.selected.clone() else {
        return;
    };
    let Some(m) = lib.macros.iter_mut().find(|m| m.id == id) else {
        return;
    };
    if let Some(rec) = st.recording.as_mut() {
        for ev in keys {
            record_event(&mut m.steps, &mut rec.last, ev, st.collapse);
        }
    } else if let Some(cap) = st.capture {
        for ev in keys {
            if let Some((usage, mods)) = st.chord.feed(ev) {
                m.steps.push(match cap {
                    Capture::Tap => MacroStep::KeyTap { usage, mods },
                    Capture::Down => MacroStep::KeyDown(usage),
                    Capture::Up => MacroStep::KeyUp(usage),
                });
                st.capture = None;
                st.chord = Chord::default();
                break;
            }
        }
    }
}

fn list(ui: &mut Ui, st: &mut MacrosState, lib: &mut Library) {
    let pal = palette(ui.ctx());
    widgets::subtitle(ui, "Your macros");
    ui.add_space(4.0);
    ui.spacing_mut().item_spacing.y = 4.0;
    for m in &lib.macros {
        let selected = st.selected.as_deref() == Some(m.id.as_str());
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
        let p = ui.painter();
        if selected {
            p.rect(
                rect,
                CornerRadius::same(9),
                pal.accent_soft,
                Stroke::new(1.0, pal.accent),
                StrokeKind::Inside,
            );
        } else if resp.hovered() {
            p.rect_filled(rect, CornerRadius::same(9), pal.raised);
        }
        let g = widgets::fit_text(
            p,
            &m.name,
            egui::FontId::proportional(14.0),
            pal.text,
            rect.width() - 24.0,
        );
        p.galley(pos2(rect.min.x + 12.0, rect.min.y + 8.0), g, pal.text);
        p.text(
            pos2(rect.min.x + 12.0, rect.max.y - 11.0),
            egui::Align2::LEFT_CENTER,
            format!("{} steps · {}", m.steps.len(), m.mode.name()),
            egui::FontId::proportional(11.5),
            pal.muted,
        );
        if resp.clicked() && !selected {
            st.selected = Some(m.id.clone());
            st.stop_all();
        }
    }
    if lib.macros.is_empty() {
        widgets::caption(ui, "No macros yet");
    }
    ui.add_space(8.0);
    if widgets::primary(ui, Some(Icon::Plus), "New macro").clicked() {
        let mut name = "New macro".to_string();
        let mut n = 2;
        while lib.macros.iter().any(|m| m.name == name) {
            name = format!("New macro {n}");
            n += 1;
        }
        let m = Macro::new(name);
        st.selected = Some(m.id.clone());
        st.stop_all();
        lib.macros.push(m);
    }
}

fn editor(
    ui: &mut Ui,
    st: &mut MacrosState,
    lib: &mut Library,
    devices: &[DeviceView],
    out: &mut Vec<Action>,
) {
    let pal = palette(ui.ctx());
    let Some(id) = st.selected.clone() else {
        widgets::empty_hint(ui, Icon::Macro, "Create a macro to get started");
        return;
    };
    let used_by = usage(lib, devices, &id);
    let Some(m) = lib.macros.iter_mut().find(|m| m.id == id) else {
        return;
    };

    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(&mut m.name)
                .font(egui::TextStyle::Name("Subtitle".into()))
                .desired_width(ui.available_width() - 48.0),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::icon_button(ui, Icon::Trash, "Delete macro").clicked() {
                st.confirm_delete = Some(id.clone());
            }
        });
    });
    ui.add_space(4.0);
    let items: Vec<Seg<MacroMode>> = MacroMode::ALL
        .iter()
        .map(|md| Seg::new(*md, md.name()))
        .collect();
    if let Some(md) = widgets::segmented(ui, m.mode, &items) {
        m.mode = md;
    }
    ui.add_space(12.0);

    // Toolbar.
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let recording = st.recording.is_some();
        if recording {
            if widgets::button(ui, Btn::Danger, Some(Icon::Stop), "Stop recording").clicked() {
                st.recording = None;
            }
        } else if widgets::primary(ui, Some(Icon::Record), "Record")
            .on_hover_text("Capture key presses and releases typed into this window")
            .clicked()
        {
            st.capture = None;
            st.recording = Some(Recording { last: None });
        }
        let add = widgets::secondary(ui, Some(Icon::Plus), "Add step");
        Popup::menu(&add).show(|ui| add_menu(ui, st, m));
        if widgets::button(ui, Btn::Ghost, None, "Clear").clicked() {
            m.steps.clear();
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let mut c = st.collapse;
            if ui.checkbox(&mut c, "Collapse delays under 10 ms").changed() {
                st.collapse = c;
            }
        });
    });
    if st.recording.is_some() {
        ui.add_space(6.0);
        status_strip(
            ui,
            "Recording. Press keys in this window; each press and release is added with its timing.",
            true,
        );
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    } else if let Some(cap) = st.capture {
        ui.add_space(6.0);
        let what = match cap {
            Capture::Tap => "key tap",
            Capture::Down => "key down",
            Capture::Up => "key up",
        };
        status_strip(ui, &format!("Press a key to add a {what}."), false);
        if widgets::button(ui, Btn::Ghost, None, "Cancel").clicked() {
            st.capture = None;
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
    ui.add_space(8.0);

    // Steps.
    ui.horizontal(|ui| {
        widgets::section(ui, "Steps");
        widgets::caption(
            ui,
            format!(
                "{} · {} of delays",
                m.steps.len(),
                format_ms(m.duration_ms())
            ),
        );
    });
    ui.add_space(2.0);
    steps_list(ui, m);

    ui.add_space(12.0);
    widgets::section(ui, "Assigned to");
    ui.add_space(2.0);
    if used_by.is_empty() {
        widgets::muted(
            ui,
            "Not assigned yet. Pick the Macro category on a button in a device's Assignments tab.",
        );
        if widgets::button(ui, Btn::Ghost, Some(Icon::Cursor), "Go to devices").clicked() {
            out.push(Action::Navigate(crate::state::Route::Device));
        }
    }
    for u in used_by {
        ui.label(RichText::new(format!("• {u}")).color(pal.text));
    }
}

fn status_strip(ui: &mut Ui, text: &str, pulse: bool) {
    let pal = palette(ui.ctx());
    Frame::new()
        .fill(with_alpha(pal.danger, 24))
        .stroke(Stroke::new(1.0, with_alpha(pal.danger, 120)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let t = ui.input(|i| i.time) as f32;
                let a = if pulse {
                    0.55 + 0.45 * (t * 5.0).sin()
                } else {
                    1.0
                };
                let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                ui.painter()
                    .circle_filled(r.center(), 5.0, pal.danger.gamma_multiply(a));
                ui.label(RichText::new(text).color(pal.text));
            });
        });
}

fn add_menu(ui: &mut Ui, st: &mut MacrosState, m: &mut Macro) {
    ui.set_min_width(200.0);
    widgets::section(ui, "Keyboard");
    for (label, cap) in [
        ("Key tap", Capture::Tap),
        ("Key down", Capture::Down),
        ("Key up", Capture::Up),
    ] {
        if ui.button(label).clicked() {
            st.recording = None;
            st.capture = Some(cap);
            st.chord = Chord::default();
            ui.close();
        }
    }
    ui.add_space(4.0);
    widgets::section(ui, "Mouse");
    for (label, down) in [("Mouse down", true), ("Mouse up", false)] {
        ui.horizontal(|ui| {
            ui.label(label);
            for b in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                if ui
                    .small_button(b.name().trim_end_matches(" click"))
                    .clicked()
                {
                    m.steps.push(if down {
                        MacroStep::MouseDown(b)
                    } else {
                        MacroStep::MouseUp(b)
                    });
                    ui.close();
                }
            }
        });
    }
    ui.add_space(4.0);
    widgets::section(ui, "Other");
    if ui.button("Delay").clicked() {
        m.steps.push(MacroStep::Delay(50));
        ui.close();
    }
    if ui.button("Type text").clicked() {
        m.steps.push(MacroStep::Text(String::new()));
        ui.close();
    }
}

/// Row text for a step. The bundled font has no arrow glyphs, so avoid the core labels' arrows.
pub fn step_text(s: &MacroStep) -> String {
    let key = |u: &u8| periph_core::keys::name(*u).unwrap_or("?");
    match s {
        MacroStep::KeyDown(u) => format!("Key down · {}", key(u)),
        MacroStep::KeyUp(u) => format!("Key up · {}", key(u)),
        MacroStep::KeyTap { .. } => format!("Tap · {}", s.label()),
        MacroStep::MouseDown(b) => format!("Mouse down · {}", b.name()),
        MacroStep::MouseUp(b) => format!("Mouse up · {}", b.name()),
        MacroStep::Delay(_) | MacroStep::Text(_) => s.label(),
    }
}

fn step_icon(s: &MacroStep) -> Icon {
    match s {
        MacroStep::KeyDown(_)
        | MacroStep::KeyUp(_)
        | MacroStep::KeyTap { .. }
        | MacroStep::Text(_) => Icon::Keyboard,
        MacroStep::MouseDown(_) | MacroStep::MouseUp(_) => Icon::Mouse,
        MacroStep::Delay(_) => Icon::Gauge,
    }
}

fn steps_list(ui: &mut Ui, m: &mut Macro) {
    let pal = palette(ui.ctx());
    if m.steps.is_empty() {
        widgets::empty_hint(
            ui,
            Icon::Macro,
            "No steps yet. Record something or add a step.",
        );
        return;
    }
    let mut action: Option<(usize, i32)> = None; // (index, -1 up / 1 down / 0 delete)
    let n = m.steps.len();
    ScrollArea::vertical()
        .max_height(330.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (i, step) in m.steps.iter_mut().enumerate() {
                Frame::new()
                    .fill(pal.raised)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.allocate_ui_with_layout(
                                vec2(24.0, 22.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    widgets::caption(ui, (i + 1).to_string());
                                },
                            );
                            icons::show(ui, step_icon(step), 15.0, pal.muted);
                            match step {
                                MacroStep::Delay(ms) => {
                                    widgets::muted(ui, "Wait");
                                    ui.add(
                                        DragValue::new(ms)
                                            .range(1..=600_000)
                                            .speed(5.0)
                                            .suffix(" ms"),
                                    );
                                }
                                MacroStep::Text(t) => {
                                    ui.add(
                                        TextEdit::singleline(t)
                                            .hint_text("Text to type")
                                            .desired_width(
                                                (ui.available_width() - 110.0).max(80.0),
                                            ),
                                    );
                                }
                                other => {
                                    ui.label(RichText::new(step_text(other)).color(pal.text));
                                }
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::icon_button(ui, Icon::Trash, "Delete step").clicked() {
                                    action = Some((i, 0));
                                }
                                if ui
                                    .add_enabled_ui(i + 1 < n, |ui| {
                                        widgets::icon_button(ui, Icon::ArrowDown, "Move down")
                                    })
                                    .inner
                                    .clicked()
                                {
                                    action = Some((i, 1));
                                }
                                if ui
                                    .add_enabled_ui(i > 0, |ui| {
                                        widgets::icon_button(ui, Icon::ArrowUp, "Move up")
                                    })
                                    .inner
                                    .clicked()
                                {
                                    action = Some((i, -1));
                                }
                            });
                        });
                    });
            }
        });
    match action {
        Some((i, 0)) => {
            m.steps.remove(i);
        }
        Some((i, d)) => {
            let j = (i as i32 + d) as usize;
            m.steps.swap(i, j);
        }
        None => {}
    }
}

fn delete_dialog(ui: &mut Ui, st: &mut MacrosState, lib: &mut Library) {
    let Some(id) = st.confirm_delete.clone() else {
        return;
    };
    let Some(name) = lib.macro_by_id(&id).map(|m| m.name.clone()) else {
        st.confirm_delete = None;
        return;
    };
    let mut close = false;
    let modal = Modal::new(egui::Id::new("delete-macro")).show(ui.ctx(), |ui| {
        ui.set_width(340.0);
        widgets::subtitle(ui, "Delete macro?");
        ui.add_space(6.0);
        ui.label(format!(
            "“{name}” will be removed. Buttons that use it stop doing anything."
        ));
        ui.add_space(12.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::button(ui, Btn::Danger, Some(Icon::Trash), "Delete").clicked() {
                lib.macros.retain(|m| m.id != id);
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
    use crate::worker::DeviceSnapshot;
    use std::time::Duration;

    fn ev(usage: u8, pressed: bool, at: Instant) -> KeyEv {
        KeyEv { usage, pressed, at }
    }

    #[test]
    fn recording_inserts_delays_between_events() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut steps = Vec::new();
        let mut last = None;
        for e in [
            ev(0x04, true, t0),
            ev(0x04, false, t0 + ms(80)),
            ev(0x05, true, t0 + ms(300)),
        ] {
            record_event(&mut steps, &mut last, &e, false);
        }
        assert_eq!(
            steps,
            vec![
                MacroStep::KeyDown(0x04),
                MacroStep::Delay(80),
                MacroStep::KeyUp(0x04),
                MacroStep::Delay(220),
                MacroStep::KeyDown(0x05)
            ]
        );
    }

    #[test]
    fn collapsing_drops_tiny_gaps_only() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut steps = Vec::new();
        let mut last = None;
        for e in [
            ev(0xe0, true, t0),
            ev(0x06, true, t0 + ms(4)),
            ev(0x06, false, t0 + ms(60)),
        ] {
            record_event(&mut steps, &mut last, &e, true);
        }
        assert_eq!(
            steps,
            vec![
                MacroStep::KeyDown(0xe0),
                MacroStep::KeyDown(0x06),
                MacroStep::Delay(56),
                MacroStep::KeyUp(0x06)
            ]
        );
    }

    #[test]
    fn usage_lists_live_buttons_and_profiles() {
        let mut d = periph_core::demo::devices().remove(0);
        let mut state = d.read_state().unwrap();
        state.buttons.insert(0x53, ButtonAction::Macro("m1".into()));
        let view = DeviceView::new(DeviceSnapshot {
            descriptor: d.descriptor().clone(),
            capabilities: d.capabilities().clone(),
            state,
        });
        let mut lib = Library::starter();
        lib.profiles[0].devices.insert(
            "logitech:c095".into(),
            vec![Setting::Button {
                id: 0x52,
                action: ButtonAction::Macro("m1".into()),
            }],
        );
        let u = usage(&lib, &[view], "m1");
        assert_eq!(u.len(), 2);
        assert!(u[0].contains("G4 · Back on G502 X Plus"));
        assert!(u[1].contains("Default"));
        assert!(usage(&lib, &[], "other").is_empty());
    }

    #[test]
    fn step_text_avoids_arrow_glyphs() {
        let steps = [
            MacroStep::KeyDown(0x04),
            MacroStep::KeyUp(0x04),
            MacroStep::KeyTap {
                usage: 0x06,
                mods: periph_core::model::Modifiers(1),
            },
            MacroStep::MouseDown(MouseButton::Left),
            MacroStep::MouseUp(MouseButton::Right),
        ];
        for s in steps {
            assert!(!step_text(&s).contains(['↓', '↑']), "{}", step_text(&s));
        }
        assert_eq!(
            step_text(&MacroStep::KeyTap {
                usage: 0x06,
                mods: periph_core::model::Modifiers(1)
            }),
            "Tap · Ctrl+C"
        );
    }

    #[test]
    fn durations_format_readably() {
        assert_eq!(format_ms(250), "250 ms");
        assert_eq!(format_ms(1500), "1.5 s");
    }
}
