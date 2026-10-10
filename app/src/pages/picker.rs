//! The action picker shown under the selected button.

use egui::{
    Align, ComboBox, CornerRadius, DragValue, Layout, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Ui, vec2,
};
use gearwise_core::keys;
use gearwise_core::model::{ButtonAction, ButtonCaps, DpiFunction, Modifiers, MouseButton, media};
use gearwise_core::profile::Library;

use super::assignments::{AssignState, assign};
use crate::icons::Icon;
use crate::keymap::KeyEv;
use crate::state::{Action, DeviceView, Route};
use crate::theme::{mix, palette, with_alpha};
use crate::widgets::{self, Btn};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Default,
    Disable,
    Mouse,
    Keyboard,
    Media,
    Dpi,
    Profile,
    Macro,
}

impl Cat {
    const ALL: [Cat; 8] = [
        Cat::Default,
        Cat::Disable,
        Cat::Mouse,
        Cat::Keyboard,
        Cat::Media,
        Cat::Dpi,
        Cat::Profile,
        Cat::Macro,
    ];

    fn label(self) -> &'static str {
        match self {
            Cat::Default => "Default",
            Cat::Disable => "Disable",
            Cat::Mouse => "Mouse",
            Cat::Keyboard => "Keyboard",
            Cat::Media => "Media",
            Cat::Dpi => "DPI",
            Cat::Profile => "Profile",
            Cat::Macro => "Macro",
        }
    }

    pub fn of(a: &ButtonAction) -> Self {
        match a {
            ButtonAction::Default => Cat::Default,
            ButtonAction::Disabled => Cat::Disable,
            ButtonAction::Mouse(_) => Cat::Mouse,
            ButtonAction::Key { .. } => Cat::Keyboard,
            ButtonAction::Media(_) => Cat::Media,
            ButtonAction::Dpi(_) => Cat::Dpi,
            ButtonAction::ProfileCycle => Cat::Profile,
            ButtonAction::Macro(_) => Cat::Macro,
        }
    }
}

pub fn show(
    ui: &mut Ui,
    st: &mut AssignState,
    dev: &DeviceView,
    b: &ButtonCaps,
    lib: &Library,
    keys: &[KeyEv],
    out: &mut Vec<Action>,
) {
    let pal = palette(ui.ctx());
    let current = dev
        .state
        .buttons
        .get(&b.id)
        .cloned()
        .unwrap_or(ButtonAction::Default);
    let cat = st.cat.unwrap_or_else(|| Cat::of(&current));

    ui.horizontal(|ui| {
        let width = (ui.available_width() - 64.0).max(60.0);
        ui.allocate_ui_with_layout(
            vec2(width, 22.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("Assign an action to {}", b.name)).color(pal.text),
                    )
                    .truncate(),
                );
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if current != ButtonAction::Default
                && widgets::button(ui, Btn::Ghost, None, "Reset").clicked()
            {
                assign(dev, b, ButtonAction::Default, out);
            }
        });
    });
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for c in Cat::ALL {
            if widgets::pill(ui, c.label(), c == cat).clicked() {
                st.cat = Some(c);
                st.stop_capture();
            }
        }
    });
    ui.add_space(6.0);

    match cat {
        Cat::Default => {
            widgets::muted(
                ui,
                format!("Factory behaviour: {}", b.default_action.label()),
            );
            if widgets::secondary(ui, None, "Use default").clicked() {
                assign(dev, b, ButtonAction::Default, out);
            }
        }
        Cat::Disable => {
            widgets::muted(ui, "The button does nothing.");
            if widgets::secondary(ui, None, "Disable button").clicked() {
                assign(dev, b, ButtonAction::Disabled, out);
            }
        }
        Cat::Mouse => choice_row(
            ui,
            MouseButton::ALL
                .iter()
                .map(|m| (m.name().to_string(), ButtonAction::Mouse(*m))),
            &current,
            dev,
            b,
            out,
        ),
        Cat::Media => choice_row(
            ui,
            media::ALL
                .iter()
                .map(|(u, n)| ((*n).to_string(), ButtonAction::Media(*u))),
            &current,
            dev,
            b,
            out,
        ),
        Cat::Profile => {
            widgets::muted(ui, "Switch to the next onboard profile each press.");
            if widgets::secondary(ui, None, "Cycle profiles").clicked() {
                assign(dev, b, ButtonAction::ProfileCycle, out);
            }
        }
        Cat::Dpi => dpi_picker(ui, st, dev, b, &current, out),
        Cat::Keyboard => keyboard_picker(ui, st, dev, b, &current, keys, out),
        Cat::Macro => macro_picker(ui, dev, b, lib, &current, out),
    }
}

fn choice_row(
    ui: &mut Ui,
    items: impl Iterator<Item = (String, ButtonAction)>,
    current: &ButtonAction,
    dev: &DeviceView,
    b: &ButtonCaps,
    out: &mut Vec<Action>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for (label, action) in items {
            if widgets::pill(ui, &label, *current == action).clicked() {
                assign(dev, b, action, out);
            }
        }
    });
}

fn dpi_picker(
    ui: &mut Ui,
    st: &mut AssignState,
    dev: &DeviceView,
    b: &ButtonCaps,
    current: &ButtonAction,
    out: &mut Vec<Action>,
) {
    let simple = [
        ("DPI up", DpiFunction::Up),
        ("DPI down", DpiFunction::Down),
        ("DPI cycle", DpiFunction::Cycle),
    ];
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for (label, f) in simple {
            if widgets::pill(ui, label, *current == ButtonAction::Dpi(f)).clicked() {
                assign(dev, b, ButtonAction::Dpi(f), out);
            }
        }
    });
    ui.add_space(4.0);
    let (min, max) = dev
        .caps
        .dpi
        .as_ref()
        .map_or((100, 26_000), |d| (d.min, d.max));
    if let ButtonAction::Dpi(DpiFunction::Shift(v)) = current {
        st.shift_dpi = *v;
    } else if st.shift_dpi == 0 {
        st.shift_dpi = 400u16.clamp(min, max);
    }
    ui.horizontal(|ui| {
        widgets::muted(ui, "Hold for");
        ui.add(
            DragValue::new(&mut st.shift_dpi)
                .range(min..=max)
                .speed(10.0)
                .suffix(" DPI"),
        );
        if let Some(caps) = &dev.caps.dpi {
            st.shift_dpi = caps.snap(st.shift_dpi);
        }
        let on = matches!(current, ButtonAction::Dpi(DpiFunction::Shift(_)));
        if widgets::button(
            ui,
            if on { Btn::Secondary } else { Btn::Primary },
            None,
            if on { "Update shift" } else { "Use DPI shift" },
        )
        .clicked()
        {
            assign(
                dev,
                b,
                ButtonAction::Dpi(DpiFunction::Shift(st.shift_dpi)),
                out,
            );
        }
    });
}

fn keyboard_picker(
    ui: &mut Ui,
    st: &mut AssignState,
    dev: &DeviceView,
    b: &ButtonCaps,
    current: &ButtonAction,
    key_events: &[KeyEv],
    out: &mut Vec<Action>,
) {
    let pal = palette(ui.ctx());
    if st.capturing {
        for ev in key_events {
            if let Some((usage, mods)) = st.chord.feed(ev) {
                assign(dev, b, ButtonAction::Key { usage, mods }, out);
                st.stop_capture();
                break;
            }
        }
    }
    // Capture box.
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
    let pulse = if st.capturing {
        0.5 + 0.5 * (ui.input(|i| i.time) as f32 * 4.0).sin()
    } else {
        0.0
    };
    let stroke_col = if st.capturing {
        mix(pal.accent, pal.border, 0.5 - 0.4 * pulse)
    } else if resp.hovered() {
        mix(pal.border, pal.accent, 0.5)
    } else {
        pal.border
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(10),
        if st.capturing {
            pal.accent_soft
        } else {
            pal.card
        },
        Stroke::new(1.5, stroke_col),
        StrokeKind::Inside,
    );
    let (text, color) = if st.capturing {
        ("Press a key combination…".to_string(), pal.accent_text())
    } else if let ButtonAction::Key { .. } = current {
        (current.label(), pal.text)
    } else {
        ("Click to capture a keystroke".to_string(), pal.muted)
    };
    widgets::text_in(
        ui.painter(),
        rect,
        egui::Align2::CENTER_CENTER,
        &text,
        egui::FontId::proportional(15.0),
        color,
    );
    if resp.clicked() {
        st.capturing = !st.capturing;
        st.chord = Default::default();
    }
    if st.capturing {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
    ui.add_space(4.0);
    widgets::caption(
        ui,
        if st.capturing {
            "Release keys to finish. Click the box again to cancel."
        } else {
            "Or build one by hand:"
        },
    );

    // Manual builder.
    if let ButtonAction::Key { usage, mods } = current {
        st.key_usage = *usage;
        st.key_mods = mods.0;
    } else if st.key_usage == 0 {
        st.key_usage = 0x04;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for (label, bit) in [
            ("Ctrl", Modifiers::LCTRL),
            ("Shift", Modifiers::LSHIFT),
            ("Alt", Modifiers::LALT),
            ("Super", Modifiers::LGUI),
        ] {
            let on = st.key_mods & (bit | bit << 4) != 0;
            if widgets::pill(ui, label, on).clicked() {
                st.key_mods = if on {
                    st.key_mods & !(bit | bit << 4)
                } else {
                    st.key_mods | bit
                };
            }
        }
    });
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("key-pick")
            .width(120.0)
            .selected_text(keys::name(st.key_usage).unwrap_or("Key"))
            .show_ui(ui, |ui| {
                ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for (u, n) in keys::KEYS {
                        ui.selectable_value(&mut st.key_usage, *u, *n);
                    }
                });
            });
        let wanted = ButtonAction::Key {
            usage: st.key_usage,
            mods: Modifiers(st.key_mods),
        };
        if widgets::button(ui, Btn::Primary, None, "Assign").clicked() && *current != wanted {
            assign(dev, b, wanted, out);
        }
    });
}

fn macro_picker(
    ui: &mut Ui,
    dev: &DeviceView,
    b: &ButtonCaps,
    lib: &Library,
    current: &ButtonAction,
    out: &mut Vec<Action>,
) {
    let pal = palette(ui.ctx());
    if lib.macros.is_empty() {
        widgets::muted(ui, "No macros yet.");
        if widgets::secondary(ui, Some(Icon::Macro), "Create a macro").clicked() {
            out.push(Action::Navigate(Route::Macros));
        }
        return;
    }
    ui.spacing_mut().item_spacing.y = 4.0;
    for m in &lib.macros {
        let action = ButtonAction::Macro(m.id.clone());
        let selected = *current == action;
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
        let p = ui.painter();
        if selected {
            p.rect(
                rect,
                CornerRadius::same(8),
                pal.accent_soft,
                Stroke::new(1.0, pal.accent),
                StrokeKind::Inside,
            );
        } else if resp.hovered() {
            p.rect_filled(rect, CornerRadius::same(8), pal.raised);
        }
        crate::icons::paint(
            p,
            crate::icons::square(rect.min + vec2(20.0, 17.0), 16.0),
            Icon::Macro,
            if selected { pal.accent } else { pal.muted },
        );
        p.text(
            rect.min + vec2(38.0, 17.0),
            egui::Align2::LEFT_CENTER,
            &m.name,
            egui::FontId::proportional(14.0),
            pal.text,
        );
        p.text(
            rect.max - vec2(12.0, 17.0),
            egui::Align2::RIGHT_CENTER,
            format!("{} steps · {}", m.steps.len(), m.mode.name()),
            egui::FontId::proportional(12.0),
            with_alpha(pal.muted, 230),
        );
        if resp.clicked() {
            assign(dev, b, action, out);
        }
    }
}
