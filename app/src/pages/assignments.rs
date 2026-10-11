//! Assignments tab: device art with clickable hotspots, the button list and the action picker.

use egui::{
    Align, CornerRadius, FontId, Frame, Layout, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};
use gearwise_core::device::Setting;
use gearwise_core::model::{ButtonAction, ButtonCaps, DeviceKind};
use gearwise_core::profile::Library;

use super::action_label;
use super::picker::{self, Cat};
use crate::icons::{self, Icon};
use crate::illustration::callouts::{self, Callout};
use crate::illustration::headset::paint_headset;
use crate::illustration::{self, KeyPaint, Kind, MouseArt, MouseLook};
use crate::keymap::{Chord, KeyEv};
use crate::state::{Action, DeviceView};
use crate::theme::{Palette, mix, palette, with_alpha};
use crate::widgets;

#[derive(Default)]
pub struct AssignState {
    pub selected: Option<u16>,
    device: String,
    pub capturing: bool,
    pub chord: Chord,
    pub cat: Option<Cat>,
    cat_for: Option<u16>,
    pub shift_dpi: u16,
    pub key_usage: u8,
    pub key_mods: u8,
    /// Button under the pointer last frame / this frame (art, callouts and list share it).
    hover_seen: Option<u16>,
    hover_now: Option<u16>,
}

impl AssignState {
    pub fn stop_capture(&mut self) {
        self.capturing = false;
        self.chord = Chord::default();
    }
}

pub fn show(
    ui: &mut Ui,
    st: &mut AssignState,
    dev: &DeviceView,
    lib: &Library,
    keys: &[KeyEv],
    out: &mut Vec<Action>,
) {
    if st.device != dev.desc.id {
        *st = AssignState {
            device: dev.desc.id.clone(),
            ..Default::default()
        };
    }
    if st
        .selected
        .is_none_or(|id| !dev.caps.buttons.iter().any(|b| b.id == id))
    {
        st.selected = dev.caps.buttons.iter().find(|b| b.remappable).map(|b| b.id);
    }
    st.hover_seen = st.hover_now.take();
    if st.cat_for != st.selected {
        st.cat_for = st.selected;
        st.cat = None;
        st.stop_capture();
    }

    // Keyboards, and anything in a narrow window, get the art full width with the list below.
    if dev.desc.kind == DeviceKind::Keyboard || ui.available_width() < 900.0 {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            illustration_panel(ui, st, dev, lib);
        });
        ui.add_space(14.0);
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            button_list(ui, st, dev, lib, keys, out);
        });
        return;
    }
    let st = std::cell::RefCell::new(st);
    let out = std::cell::RefCell::new(out);
    widgets::card_columns(
        ui,
        0.62,
        |ui| illustration_panel(ui, &mut st.borrow_mut(), dev, lib),
        |ui| {
            button_list(
                ui,
                &mut st.borrow_mut(),
                dev,
                lib,
                keys,
                &mut out.borrow_mut(),
            )
        },
    );
}

fn action_of(dev: &DeviceView, b: &ButtonCaps) -> ButtonAction {
    dev.state
        .buttons
        .get(&b.id)
        .cloned()
        .unwrap_or(ButtonAction::Default)
}

fn illustration_panel(ui: &mut Ui, st: &mut AssignState, dev: &DeviceView, lib: &Library) {
    match dev.desc.kind {
        DeviceKind::Keyboard => keyboard_art(ui, st, dev, lib),
        DeviceKind::Mouse => mouse_art(ui, st, dev, lib),
        DeviceKind::Headset => {
            let pal = palette(ui.ctx());
            let (area, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 360.0), Sense::hover());
            paint_headset(ui.painter(), area.shrink(20.0), &pal, &MouseLook::default());
        }
        kind => {
            let pal = palette(ui.ctx());
            let (area, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 300.0), Sense::hover());
            icons::paint(
                ui.painter(),
                icons::square(area.center(), 120.0),
                Icon::of(kind),
                with_alpha(pal.muted, 120),
            );
        }
    }
}

/// Label text and side for every placed button.
fn callout_items(
    art: &MouseArt,
    placed: &[illustration::mouse::Placement],
    dev: &DeviceView,
    lib: &Library,
) -> Vec<Callout> {
    placed
        .iter()
        .filter_map(|pl| {
            let b = dev.caps.buttons.iter().find(|b| b.id == pl.id)?;
            let action = action_of(dev, b);
            let custom = action != ButtonAction::Default;
            Some(Callout {
                id: b.id,
                anchor: pl.anchor,
                side: callouts::side_for(pl.role, pl.anchor, art.rect),
                title: b.name.clone(),
                detail: if !b.remappable {
                    "Fixed".to_string()
                } else if custom {
                    action_label(&action, lib)
                } else {
                    "Default".to_string()
                },
                custom,
                enabled: b.remappable,
                compact: matches!(pl.role, Some(illustration::Role::Grid(_))),
            })
        })
        .collect()
}

/// Widest a label pill may get before its text is cut with an ellipsis.
const MAX_PILL_W: f32 = 230.0;
/// Narrowest the device art is allowed to get before the pills give way instead.
const MIN_ART_W: f32 = 150.0;

fn mouse_art(ui: &mut Ui, st: &mut AssignState, dev: &DeviceView, lib: &Library) {
    let pal = palette(ui.ctx());
    let kind = Kind::for_name(dev.display_name());

    // Which labels go where does not depend on the size, so measure them on a probe first.
    let probe = MouseArt::new(
        illustration::art_rect(Rect::from_min_size(Pos2::ZERO, vec2(200.0, 400.0)), kind),
        kind,
    );
    let probe_items = callout_items(&probe, &probe.place(&dev.caps.buttons), dev, lib);
    let per_side = |side| {
        probe_items
            .iter()
            .filter(|c| c.side == side && !c.compact)
            .count()
    };
    let crowded = per_side(callouts::Side::Left).max(per_side(callouts::Side::Right));

    // The art fills the viewport's height (never overflowing it) and its labels take the
    // width they need; only when the art would get tiny do the labels give way.
    let budget = (widgets::viewport_height(ui) - 44.0).clamp(300.0, 580.0);
    let height = budget.max(callouts::height_needed(crowded));
    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let (mut lw, mut rw) = callouts::column_widths(ui.painter(), &probe_items, MAX_PILL_W);
    let lead = 2.0 * callouts::LEAD;
    if area.width() - lead - lw - rw < MIN_ART_W {
        let each = ((area.width() - lead - MIN_ART_W) / 2.0).max(104.0);
        (lw, rw) = (lw.min(each), rw.min(each));
    }
    let free = Rect::from_min_size(
        area.min,
        vec2((area.width() - lead - lw - rw).max(100.0), area.height()),
    );
    let mut rect = illustration::art_rect(free.shrink2(vec2(0.0, 6.0)), kind);
    // Centre label columns and art as one group.
    let group = lw + callouts::LEAD + rect.width() + callouts::LEAD + rw;
    let left = area.min.x + (area.width() - group).max(0.0) / 2.0 + lw + callouts::LEAD;
    rect = rect.translate(vec2(left - rect.min.x, 0.0));

    let art = MouseArt::new(rect, kind);
    let placed = art.place(&dev.caps.buttons);
    let tint: Vec<_> = placed
        .iter()
        .filter_map(|pl| {
            let role = pl.role?;
            let strength = if st.selected == Some(pl.id) {
                1.0
            } else if st.hover_seen == Some(pl.id) {
                0.7
            } else {
                return None;
            };
            Some((role, strength))
        })
        .collect();
    art.paint(ui.painter(), &pal, &MouseLook::default(), &tint);

    let items = callout_items(&art, &placed, dev, lib);
    let hit = callouts::show(
        ui,
        area,
        rect,
        (lw, rw),
        &items,
        st.selected,
        st.hover_seen,
        &pal,
    );
    if hit.hovered.is_some() {
        st.hover_now = hit.hovered;
    }
    if let Some(id) = hit.clicked {
        st.selected = Some(id);
    }
    let unplaced = dev.caps.buttons.len() - placed.len();
    if unplaced > 0 {
        widgets::caption(
            ui,
            format!(
                "{unplaced} more button{} listed on the right",
                if unplaced == 1 { "" } else { "s" }
            ),
        );
    }
}

fn keyboard_art(ui: &mut Ui, st: &mut AssignState, dev: &DeviceView, lib: &Library) {
    let pal = palette(ui.ctx());
    let extras: Vec<&ButtonCaps> = dev
        .caps
        .buttons
        .iter()
        .filter(|b| b.hotspot.is_none() && !super::kb_preview::board_layout(dev).contains(b.id))
        .collect();
    let extra_h = if extras.is_empty() { 0.0 } else { 70.0 };
    let board_h = super::kb_preview::board_height(ui, dev, 150.0 + extra_h);
    let (area, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), board_h + extra_h),
        Sense::hover(),
    );
    let id = ui.id().with("art");
    let mut hovered: Option<(Rect, String)> = None;

    let layout = super::kb_preview::board_layout(dev);
    let kb_area = Rect::from_min_max(area.min, pos2(area.max.x, area.max.y - extra_h));
    let g = illustration::keyboard_geom(kb_area.shrink(4.0), &layout);
    illustration::paint_chassis(ui.painter(), &g, &pal, None);
    for cell in &layout.keys {
        let r = illustration::key_rect(&g, cell);
        match dev
            .caps
            .buttons
            .iter()
            .find(|b| b.id == cell.id && b.remappable)
        {
            Some(b) => {
                let resp = ui.interact(r, id.with(b.id), Sense::click());
                let paint = key_paint(
                    &pal,
                    st.selected == Some(b.id),
                    resp.hovered() || st.hover_seen == Some(b.id),
                    action_of(dev, b) != ButtonAction::Default,
                );
                illustration::paint_key(ui.painter(), r, &cell.label, paint, g.unit);
                if resp.clicked() {
                    st.selected = Some(b.id);
                }
                if resp.hovered() {
                    st.hover_now = Some(b.id);
                    hovered = Some((
                        r,
                        format!("{} · {}", b.name, action_label(&action_of(dev, b), lib)),
                    ));
                }
            }
            None => {
                let dim = KeyPaint {
                    fill: mix(pal.card, pal.raised, 0.7),
                    label: with_alpha(pal.muted, 200),
                    stroke: Stroke::new(1.0, with_alpha(pal.border, 190)),
                    glow: None,
                };
                illustration::paint_key(ui.painter(), r, &cell.label, dim, g.unit);
            }
        }
    }
    // Buttons that are not on the layout (macro keys, G-keys) sit in a row below.
    let mut x = g.chassis.min.x;
    for b in extras {
        let r = Rect::from_min_size(pos2(x, g.chassis.max.y + 18.0), vec2(58.0, 38.0));
        x += 66.0;
        let resp = ui.interact(r, id.with(b.id), Sense::click());
        let paint = key_paint(
            &pal,
            st.selected == Some(b.id),
            resp.hovered() || st.hover_seen == Some(b.id),
            action_of(dev, b) != ButtonAction::Default,
        );
        illustration::paint_key(ui.painter(), r, &b.name, paint, 40.0);
        if resp.clicked() {
            st.selected = Some(b.id);
        }
        if resp.hovered() {
            st.hover_now = Some(b.id);
            hovered = Some((
                r,
                format!("{} · {}", b.name, action_label(&action_of(dev, b), lib)),
            ));
        }
    }
    if let Some((anchor, text)) = hovered {
        label_bubble(ui, area, anchor, &text, &pal);
    }
}

fn key_paint(pal: &Palette, selected: bool, hovered: bool, assigned: bool) -> KeyPaint {
    let base = if assigned {
        mix(pal.raised, pal.accent, 0.28)
    } else {
        mix(pal.raised, pal.text, 0.04)
    };
    KeyPaint {
        fill: if hovered {
            mix(base, pal.accent, 0.25)
        } else {
            base
        },
        label: if selected {
            pal.accent_text()
        } else {
            pal.text
        },
        stroke: if selected {
            Stroke::new(2.0, pal.accent)
        } else {
            Stroke::new(1.0, mix(pal.border, pal.accent, 0.5))
        },
        glow: None,
    }
}

fn label_bubble(ui: &Ui, bounds: Rect, anchor: Rect, text: &str, pal: &Palette) {
    let p = ui.painter();
    let galley = p.layout_no_wrap(text.to_string(), FontId::proportional(12.5), pal.text);
    let size = galley.size() + vec2(18.0, 12.0);
    let mut x = anchor.center().x - size.x / 2.0;
    x = x.clamp(bounds.min.x, (bounds.max.x - size.x).max(bounds.min.x));
    let above = anchor.min.y - size.y - 6.0 >= bounds.min.y;
    let y = if above {
        anchor.min.y - size.y - 6.0
    } else {
        anchor.max.y + 6.0
    };
    let r = Rect::from_min_size(pos2(x, y), size);
    p.rect(
        r,
        CornerRadius::same(8),
        pal.raised,
        Stroke::new(1.0, pal.accent),
        StrokeKind::Inside,
    );
    p.galley(r.min + vec2(9.0, 6.0), galley, pal.text);
}

fn button_list(
    ui: &mut Ui,
    st: &mut AssignState,
    dev: &DeviceView,
    lib: &Library,
    keys: &[KeyEv],
    out: &mut Vec<Action>,
) {
    let pal = palette(ui.ctx());
    ui.horizontal(|ui| {
        widgets::subtitle(ui, "Buttons");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let n = dev.state.buttons.len();
            widgets::caption(
                ui,
                if n == 0 {
                    "All default".to_string()
                } else {
                    format!("{n} customised")
                },
            );
        });
    });
    ui.add_space(4.0);
    ui.spacing_mut().item_spacing.y = 4.0;
    for b in &dev.caps.buttons {
        let action = action_of(dev, b);
        let selected = st.selected == Some(b.id);
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
        let hot = (resp.hovered() || st.hover_seen == Some(b.id)) && b.remappable;
        if resp.hovered() && b.remappable {
            st.hover_now = Some(b.id);
        }
        let p = ui.painter();
        if selected {
            p.rect(
                rect,
                CornerRadius::same(9),
                pal.accent_soft,
                Stroke::new(1.0, pal.accent),
                StrokeKind::Inside,
            );
        } else if hot {
            p.rect_filled(rect, CornerRadius::same(9), pal.raised);
        }
        let name_color = if b.remappable { pal.text } else { pal.muted };
        let (label, color) = if !b.remappable {
            ("Fixed".to_string(), pal.muted)
        } else if action == ButtonAction::Default {
            (format!("Default · {}", b.default_action.label()), pal.muted)
        } else {
            (action_label(&action, lib), pal.accent_text())
        };
        // Both texts get what they need; only when they cannot share the row is one shortened.
        let natural = |text: &str, size: f32| {
            p.layout_no_wrap(text.to_string(), FontId::proportional(size), name_color)
                .size()
                .x
        };
        let (name_w, label_w) = split_widths(
            natural(&b.name, 14.0),
            natural(&label, 13.0),
            rect.width() - 24.0 - 12.0,
        );
        let g = widgets::fit_text(p, &b.name, FontId::proportional(14.0), name_color, name_w);
        p.galley(
            pos2(rect.min.x + 12.0, rect.center().y - g.size().y / 2.0),
            g,
            name_color,
        );
        let g = widgets::fit_text(p, &label, FontId::proportional(13.0), color, label_w);
        p.galley(
            pos2(
                rect.max.x - 12.0 - g.size().x,
                rect.center().y - g.size().y / 2.0,
            ),
            g,
            color,
        );
        if resp.clicked() && b.remappable {
            st.selected = if selected { None } else { Some(b.id) };
        }
        if selected {
            Frame::new()
                .fill(pal.input)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    picker::show(ui, st, dev, b, lib, keys, out);
                });
            ui.add_space(6.0);
        }
    }
    if dev.caps.buttons.is_empty() {
        widgets::empty_hint(ui, Icon::Cursor, "This device has no configurable buttons");
    }
}

/// Split a row between a name and an action label: each gets its natural width when both
/// fit, otherwise the longer one gives way (neither drops below half unless it needs less).
fn split_widths(name: f32, label: f32, avail: f32) -> (f32, f32) {
    if name + label <= avail {
        (name, label)
    } else if name <= avail * 0.5 {
        (name, avail - name)
    } else if label <= avail * 0.5 {
        (avail - label, label)
    } else {
        (avail * 0.5, avail * 0.5)
    }
}

/// Apply `action` to the button.
pub fn assign(dev: &DeviceView, b: &ButtonCaps, action: ButtonAction, out: &mut Vec<Action>) {
    out.push(Action::Apply {
        dev: dev.desc.id.clone(),
        setting: Setting::Button { id: b.id, action },
        debounce: false,
    });
}

#[cfg(test)]
mod tests {
    use super::split_widths;

    #[test]
    fn rows_share_width_by_need() {
        assert_eq!(split_widths(100.0, 80.0, 300.0), (100.0, 80.0));
        // The short name keeps its width; the long label takes the rest.
        assert_eq!(split_widths(60.0, 400.0, 300.0), (60.0, 240.0));
        assert_eq!(split_widths(400.0, 60.0, 300.0), (240.0, 60.0));
        assert_eq!(split_widths(400.0, 400.0, 300.0), (150.0, 150.0));
    }
}
