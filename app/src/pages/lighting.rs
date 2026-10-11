//! Lighting tab: zones, animated effect tiles, colour and speed controls, and a live preview.

use std::collections::HashMap;
use std::time::Duration;

use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{
    Align, Color32, CornerRadius, FontId, Layout, Rect, Sense, Slider, Stroke, StrokeKind,
    TextEdit, Ui, pos2, vec2,
};
use gearwise_core::device::Setting;
use gearwise_core::model::{
    DeviceKind, EffectKind, LightingEffect, LightingZoneCaps, Rgb, WaveDirection, ZoneState,
};

use super::per_key::{self, PerKeyState};
use crate::fx;
use crate::illustration::headset::paint_headset;
use crate::illustration::{self, Design, MouseArt, MouseLook, ZoneRole, zone_role};
use crate::state::{Action, DeviceView};
use crate::theme::{Palette, c32, mix, palette, rgb, with_alpha};
use crate::widgets::{self, Seg};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum LightMode {
    #[default]
    Effects,
    PerKey,
}

#[derive(Default)]
pub struct LightState {
    device: String,
    pub zone: u8,
    pub mode: LightMode,
    pub per_key: PerKeyState,
    /// Last colour chosen per zone, so switching to a colourless effect and back keeps it.
    colors: HashMap<u8, Rgb>,
    hex: String,
}

const PRESETS: [(u8, u8, u8); 14] = [
    (255, 59, 48),
    (255, 120, 20),
    (255, 176, 32),
    (255, 230, 60),
    (150, 230, 40),
    (68, 214, 44),
    (0, 212, 170),
    (0, 184, 252),
    (40, 100, 255),
    (139, 92, 246),
    (232, 93, 208),
    (255, 77, 94),
    (255, 255, 255),
    (255, 190, 140),
];

const DEFAULT_COLOR: Rgb = Rgb::new(0, 212, 170);

pub fn with_color(e: LightingEffect, c: Rgb) -> LightingEffect {
    match e {
        LightingEffect::Static { .. } => LightingEffect::Static { color: c },
        LightingEffect::Breathing { period_ms, .. } => LightingEffect::Breathing {
            color: c,
            period_ms,
        },
        LightingEffect::Reactive { duration_ms, .. } => LightingEffect::Reactive {
            color: c,
            duration_ms,
        },
        other => other,
    }
}

pub fn with_period(e: LightingEffect, ms: u16) -> LightingEffect {
    match e {
        LightingEffect::Breathing { color, .. } => LightingEffect::Breathing {
            color,
            period_ms: ms,
        },
        LightingEffect::ColorCycle { .. } => LightingEffect::ColorCycle { period_ms: ms },
        LightingEffect::Wave { direction, .. } => LightingEffect::Wave {
            period_ms: ms,
            direction,
        },
        LightingEffect::Reactive { color, .. } => LightingEffect::Reactive {
            color,
            duration_ms: ms,
        },
        other => other,
    }
}

pub fn with_direction(e: LightingEffect, d: WaveDirection) -> LightingEffect {
    match e {
        LightingEffect::Wave { period_ms, .. } => LightingEffect::Wave {
            period_ms,
            direction: d,
        },
        other => other,
    }
}

/// Pseudo zone id for "apply to every zone at once" (never a real zone id).
pub const ALL_ZONES: u8 = u8::MAX;

pub fn show(ui: &mut Ui, st: &mut LightState, dev: &DeviceView, out: &mut Vec<Action>) {
    let zones = &dev.caps.lighting;
    if st.device != dev.desc.id {
        *st = LightState {
            device: dev.desc.id.clone(),
            zone: zones.first().map_or(0, |z| z.id),
            ..Default::default()
        };
        st.mode = if zones.is_empty() {
            LightMode::PerKey
        } else {
            LightMode::Effects
        };
    }
    let all_ok = st.zone == ALL_ZONES && zones.len() > 1;
    if !all_ok && !zones.iter().any(|z| z.id == st.zone) {
        st.zone = zones.first().map_or(0, |z| z.id);
    }
    let has_effects = !zones.is_empty();
    let has_per_key = dev.caps.per_key.is_some();
    if !has_per_key {
        st.mode = LightMode::Effects;
    } else if !has_effects {
        st.mode = LightMode::PerKey;
    }

    let show_zones = has_effects && st.mode == LightMode::Effects && zones.len() > 1;
    let switch = has_effects && has_per_key;
    // With many zones there is no room beside the mode switch: it gets a row of its own, and
    // the zone chips wrap instead of running past the card.
    let own_row = switch && show_zones && zones.len() >= 5;
    let mode_switch = |ui: &mut Ui, st: &mut LightState| {
        let items = [
            Seg::new(LightMode::Effects, "Effects"),
            Seg::new(LightMode::PerKey, "Per-key"),
        ];
        if let Some(m) = widgets::segmented(ui, st.mode, &items) {
            st.mode = m;
        }
    };
    let zone_chips = |ui: &mut Ui, st: &mut LightState| {
        ui.spacing_mut().item_spacing.x = 8.0;
        widgets::muted(ui, "Zone");
        if widgets::pill(ui, "All zones", st.zone == ALL_ZONES).clicked() {
            st.zone = ALL_ZONES;
        }
        for z in zones {
            if widgets::pill(ui, &z.name, z.id == st.zone).clicked() {
                st.zone = z.id;
            }
        }
    };
    if own_row {
        ui.horizontal(|ui| mode_switch(ui, st));
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| zone_chips(ui, st));
    } else if switch {
        ui.horizontal(|ui| {
            if show_zones {
                zone_chips(ui, st);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                mode_switch(ui, st)
            });
        });
    } else if show_zones {
        ui.horizontal_wrapped(|ui| zone_chips(ui, st));
    }
    ui.add_space(12.0);

    match st.mode {
        LightMode::PerKey => per_key::show(ui, &mut st.per_key, dev, out),
        LightMode::Effects => effects(ui, st, dev, out),
    }
}

pub(super) fn zone_state(dev: &DeviceView, id: u8) -> ZoneState {
    dev.state.lighting.get(&id).copied().unwrap_or(ZoneState {
        effect: LightingEffect::Off,
        brightness: 100,
    })
}

/// The effects every zone supports, as a pseudo zone for "All zones".
fn all_zones(zones: &[LightingZoneCaps]) -> LightingZoneCaps {
    let effects = zones.first().map_or_else(Vec::new, |first| {
        first
            .effects
            .iter()
            .copied()
            .filter(|k| zones.iter().all(|z| z.effects.contains(k)))
            .collect()
    });
    LightingZoneCaps {
        keys: Vec::new(),
        id: ALL_ZONES,
        name: "All zones".into(),
        effects,
        brightness: zones.iter().all(|z| z.brightness),
    }
}

fn effects(ui: &mut Ui, st: &mut LightState, dev: &DeviceView, out: &mut Vec<Action>) {
    let all;
    let zone = if st.zone == ALL_ZONES {
        all = all_zones(&dev.caps.lighting);
        &all
    } else {
        match dev.caps.lighting.iter().find(|z| z.id == st.zone) {
            Some(z) => z,
            None => return,
        }
    };
    // For "All zones" show the first zone that is lit, else the first zone.
    let current = if zone.id == ALL_ZONES {
        dev.caps
            .lighting
            .iter()
            .map(|z| zone_state(dev, z.id))
            .find(|s| s.effect != LightingEffect::Off)
            .unwrap_or_else(|| zone_state(dev, dev.caps.lighting[0].id))
    } else {
        zone_state(dev, zone.id)
    };
    if let Some(c) = current.effect.color() {
        st.colors.insert(zone.id, c);
    }
    let t = ui.input(|i| i.time) as f32;
    ui.ctx().request_repaint_after(Duration::from_millis(33));

    let mut change: Option<(ZoneState, bool)> = None; // (new state, send immediately)
    if dev.desc.kind == DeviceKind::Keyboard {
        // A keyboard preview needs the full width: preview on top, controls below.
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            let h = super::kb_preview::board_height(ui, dev, 250.0);
            preview(ui, dev, zone, t, h);
        });
        ui.add_space(14.0);
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            controls(ui, st, zone, current, t, &mut change);
        });
    } else {
        let st_cell = std::cell::RefCell::new(st);
        let change_cell = std::cell::RefCell::new(&mut change);
        widgets::card_columns(
            ui,
            0.46,
            |ui| preview(ui, dev, zone, t, mouse_preview_height(ui)),
            |ui| {
                controls(
                    ui,
                    &mut st_cell.borrow_mut(),
                    zone,
                    current,
                    t,
                    &mut change_cell.borrow_mut(),
                )
            },
        );
    }
    if let Some((new, now)) = change {
        let targets: Vec<u8> = if zone.id == ALL_ZONES {
            dev.caps
                .lighting
                .iter()
                .filter(|z| z.effects.contains(&new.effect.kind()))
                .map(|z| z.id)
                .collect()
        } else {
            vec![zone.id]
        };
        for id in targets {
            out.push(Action::Apply {
                dev: dev.desc.id.clone(),
                setting: Setting::Lighting {
                    zone: id,
                    state: new,
                },
                debounce: !now,
            });
        }
    }
}

// --- Preview ---------------------------------------------------------------------

/// Height for a mouse or headset preview: the art fits what the window leaves below the zone
/// pills, so the whole device is visible without scrolling.
fn mouse_preview_height(ui: &Ui) -> f32 {
    (widgets::viewport_height(ui) - 150.0).clamp(280.0, 460.0)
}

fn preview(ui: &mut Ui, dev: &DeviceView, selected: &LightingZoneCaps, t: f32, height: f32) {
    let pal = palette(ui.ctx());
    widgets::subtitle(ui, "Live preview");
    ui.add_space(4.0);
    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    // Zone colours as they currently look, animated by the effect maths.
    let mut look = MouseLook {
        highlight: Some(zone_role(&selected.name)),
        ..Default::default()
    };
    for z in &dev.caps.lighting {
        let zs = zone_state(dev, z.id);
        let color = (zs.effect != LightingEffect::Off).then(|| {
            c32(fx::sample(
                &zs.effect,
                zs.brightness,
                t,
                0.5,
                u32::from(z.id),
            ))
        });
        match zone_role(&z.name) {
            ZoneRole::Logo => look.logo = color,
            ZoneRole::Wheel => look.wheel = color,
            ZoneRole::Body => look.body = color,
        }
    }
    match dev.desc.kind {
        DeviceKind::Keyboard => {
            super::kb_preview::keyboard_preview(ui, area, dev, selected, t, &pal)
        }
        DeviceKind::Headset => paint_headset(ui.painter(), area.shrink(10.0), &pal, &look),
        _ => {
            let design = Design::for_name(dev.display_name());
            let rect = illustration::art_rect(area.shrink2(vec2(34.0, 10.0)), design);
            MouseArt::new(rect, design).paint(ui.painter(), &pal, &look, &[]);
        }
    }
    ui.add_space(6.0);
    if selected.id != ALL_ZONES {
        let zs = zone_state(dev, selected.id);
        widgets::caption(
            ui,
            format!("{} · {}", selected.name, zs.effect.kind().name()),
        );
    }
}

pub fn text_on(c: Color32) -> Color32 {
    let luma = 0.299 * f32::from(c.r()) + 0.587 * f32::from(c.g()) + 0.114 * f32::from(c.b());
    if luma > 150.0 {
        Color32::from_rgb(0x12, 0x15, 0x1b)
    } else {
        Color32::from_rgb(0xf2, 0xf4, 0xf8)
    }
}

// --- Controls ------------------------------------------------------------------

fn controls(
    ui: &mut Ui,
    st: &mut LightState,
    zone: &LightingZoneCaps,
    current: ZoneState,
    t: f32,
    change: &mut Option<(ZoneState, bool)>,
) {
    let pal = palette(ui.ctx());
    widgets::subtitle(ui, &zone.name);
    ui.add_space(2.0);
    widgets::section(ui, "Effect");
    let base_color = st.colors.get(&zone.id).copied().unwrap_or(DEFAULT_COLOR);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
        for kind in EffectKind::ALL
            .into_iter()
            .filter(|k| zone.effects.contains(k))
        {
            let sample_effect = if kind == current.effect.kind() {
                current.effect
            } else {
                LightingEffect::default_for(kind, base_color)
            };
            let selected = kind == current.effect.kind();
            if effect_tile(ui, kind, &sample_effect, selected, t, &pal).clicked() && !selected {
                *change = Some((
                    ZoneState {
                        effect: LightingEffect::default_for(kind, base_color),
                        ..current
                    },
                    true,
                ));
            }
        }
    });

    let mut effect = current.effect;
    let mut soon = false;
    let mut now = false;

    if let Some(color) = effect.color() {
        ui.add_space(10.0);
        widgets::section(ui, "Colour");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (r, g, b) in PRESETS {
                let c = Rgb::new(r, g, b);
                if widgets::swatch(ui, c32(c), 24.0, c == color).clicked() {
                    effect = with_color(effect, c);
                    now = true;
                }
            }
        });
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let mut c = c32(color);
            if color_edit_button_srgba(ui, &mut c, Alpha::Opaque)
                .on_hover_text("Open the colour picker")
                .changed()
            {
                effect = with_color(effect, rgb(c));
                soon = true;
            }
            let editing = ui.memory(|m| m.has_focus(ui.id().with("hex")));
            if !editing {
                st.hex = color.to_hex();
            }
            let r = ui.add(
                TextEdit::singleline(&mut st.hex)
                    .id(ui.id().with("hex"))
                    .desired_width(86.0)
                    .font(FontId::monospace(13.0)),
            );
            if r.changed()
                && let Some(c) = Rgb::from_hex(&st.hex)
            {
                effect = with_color(effect, c);
                soon = true;
            }
            widgets::caption(ui, "Pick any colour or type a hex value");
        });
    }

    if let Some(ms) = effect.period_ms() {
        ui.add_space(8.0);
        let reactive = effect.kind() == EffectKind::Reactive;
        widgets::section(ui, if reactive { "Fade time" } else { "Cycle time" });
        let mut v = ms;
        let (lo, hi) = if reactive {
            (100u16, 3000u16)
        } else {
            (500u16, 20_000u16)
        };
        ui.spacing_mut().slider_width = (ui.available_width() - 90.0).clamp(160.0, 460.0);
        let r = ui.add(
            Slider::new(&mut v, lo..=hi)
                .logarithmic(true)
                .custom_formatter(|n, _| format!("{:.1} s", n / 1000.0)),
        );
        if r.changed() {
            effect = with_period(effect, v);
            soon = true;
        }
        if r.drag_stopped() {
            now = true;
        }
    }

    if let LightingEffect::Wave { direction, .. } = effect {
        ui.add_space(8.0);
        widgets::section(ui, "Direction");
        let items = [
            Seg::new(WaveDirection::LeftToRight, "Left to right"),
            Seg::new(WaveDirection::RightToLeft, "Right to left"),
        ];
        if let Some(d) = widgets::segmented(ui, direction, &items) {
            effect = with_direction(effect, d);
            now = true;
        }
    }

    let mut brightness = current.brightness;
    if zone.brightness && current.effect != LightingEffect::Off {
        ui.add_space(8.0);
        widgets::section(ui, "Brightness");
        ui.spacing_mut().slider_width = (ui.available_width() - 90.0).clamp(160.0, 460.0);
        let r = ui.add(Slider::new(&mut brightness, 0..=100).suffix("%"));
        if r.changed() {
            soon = true;
        }
        if r.drag_stopped() {
            now = true;
        }
    }

    if (soon || now) && change.is_none() {
        *change = Some((ZoneState { effect, brightness }, now));
    }
}

/// Selectable tile with a miniature animated strip of LEDs.
fn effect_tile(
    ui: &mut Ui,
    kind: EffectKind,
    sample: &LightingEffect,
    selected: bool,
    t: f32,
    pal: &Palette,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(108.0, 84.0), Sense::click());
    let p = ui.painter();
    let (fill, stroke) = if selected {
        (pal.accent_soft, Stroke::new(1.5, pal.accent))
    } else if resp.hovered() {
        (
            mix(pal.raised, pal.accent, 0.1),
            Stroke::new(1.0, mix(pal.border, pal.accent, 0.5)),
        )
    } else {
        (pal.raised, Stroke::new(1.0, pal.border))
    };
    p.rect(
        rect,
        CornerRadius::same(10),
        fill,
        stroke,
        StrokeKind::Inside,
    );
    let strip = Rect::from_min_size(rect.min + vec2(10.0, 11.0), vec2(rect.width() - 20.0, 34.0));
    p.rect_filled(strip, CornerRadius::same(6), pal.input);
    let n = 10;
    let w = (strip.width() - 8.0) / n as f32;
    for i in 0..n {
        let pos = i as f32 / (n - 1) as f32;
        let c = fx::sample(sample, 100, t, pos, i as u32 * 7 + 3);
        let cell = Rect::from_min_size(
            strip.min + vec2(4.0 + i as f32 * w, 4.0),
            vec2(w - 2.0, strip.height() - 8.0),
        );
        let col = if kind == EffectKind::Off {
            with_alpha(pal.muted, 40)
        } else {
            c32(c)
        };
        p.rect_filled(cell, CornerRadius::same(3), col);
    }
    p.text(
        pos2(rect.center().x, rect.max.y - 16.0),
        egui::Align2::CENTER_CENTER,
        kind.name(),
        FontId::proportional(13.0),
        if selected {
            pal.accent_text()
        } else {
            pal.text
        },
    );
    resp
}

#[cfg(test)]
mod tests;
