//! Per-key RGB editor: paint, rectangle-select, fill, rainbow, eyedropper.

use std::collections::BTreeSet;

use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{
    Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui, vec2,
};
use gearwise_core::device::Setting;
use gearwise_core::keys::{KeyCell, KeyboardLayout};
use gearwise_core::model::Rgb;

use super::lighting::text_on;
use crate::icons::Icon;
use crate::illustration::{self, KbGeom, KeyPaint};
use crate::state::{Action, DeviceView};
use crate::theme::{c32, mix, palette, rgb, with_alpha};
use crate::widgets::{self, Btn};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Tool {
    #[default]
    Paint,
    Eyedropper,
}

#[derive(Clone, Copy)]
enum Drag {
    Paint,
    Select { start: Pos2, additive: bool },
}

pub struct PerKeyState {
    brush: Rgb,
    tool: Tool,
    selection: BTreeSet<u16>,
    base_selection: BTreeSet<u16>,
    drag: Option<Drag>,
    last: Option<Pos2>,
    hex: String,
}

impl Default for PerKeyState {
    fn default() -> Self {
        Self {
            brush: Rgb::new(0, 212, 170),
            tool: Tool::Paint,
            selection: BTreeSet::new(),
            base_selection: BTreeSet::new(),
            drag: None,
            last: None,
            hex: String::new(),
        }
    }
}

/// Rainbow across the layout's width.
pub fn rainbow(layout: &KeyboardLayout) -> std::collections::BTreeMap<u16, Rgb> {
    let w = layout.width().max(1.0);
    layout
        .keys
        .iter()
        .map(|k| (k.id, Rgb::from_hue(300.0 * (k.x + k.w / 2.0) / w)))
        .collect()
}

/// Keys whose caps intersect `sel`.
pub fn keys_in_rect(g: &KbGeom, layout: &KeyboardLayout, sel: Rect) -> BTreeSet<u16> {
    layout
        .keys
        .iter()
        .filter(|k| illustration::key_rect(g, k).intersects(sel))
        .map(|k| k.id)
        .collect()
}

fn key_at<'a>(g: &KbGeom, layout: &'a KeyboardLayout, p: Pos2) -> Option<&'a KeyCell> {
    layout.keys.iter().find(|k| {
        illustration::key_rect(g, k)
            .expand(g.unit * 0.03)
            .contains(p)
    })
}

pub fn show(ui: &mut Ui, st: &mut PerKeyState, dev: &DeviceView, out: &mut Vec<Action>) {
    let Some(layout) = dev.caps.per_key.as_ref() else {
        return;
    };
    let pal = palette(ui.ctx());
    let id = dev.desc.id.clone();
    let mut painted = std::collections::BTreeMap::new();
    let mut flush = false;

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        brush_bar(ui, st);
        ui.add_space(8.0);

        let (area, resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), 300.0), Sense::click_and_drag());
        let g = illustration::keyboard_geom(area, layout);
        let color_of = |k: u16| dev.state.per_key.get(&k).copied().unwrap_or(Rgb::BLACK);
        let shift = ui.input(|i| i.modifiers.shift);
        let ptr = resp.interact_pointer_pos().or_else(|| resp.hover_pos());

        // Pointer handling.
        if resp.drag_started() {
            let origin = ui.input(|i| i.pointer.press_origin()).or(ptr);
            if let Some(o) = origin {
                if shift && st.tool == Tool::Paint {
                    st.base_selection = if ui.input(|i| i.modifiers.command) {
                        st.selection.clone()
                    } else {
                        BTreeSet::new()
                    };
                    st.drag = Some(Drag::Select {
                        start: o,
                        additive: true,
                    });
                } else if st.tool == Tool::Paint {
                    st.drag = Some(Drag::Paint);
                    st.last = Some(o);
                }
            }
        }
        match (st.drag, ptr) {
            (Some(Drag::Paint), Some(p)) if resp.dragged() => {
                let from = st.last.unwrap_or(p);
                let steps = ((p - from).length() / (g.unit * 0.4)).ceil().max(1.0) as usize;
                for i in 1..=steps {
                    let q = from + (p - from) * (i as f32 / steps as f32);
                    if let Some(k) = key_at(&g, layout, q)
                        && color_of(k.id) != st.brush
                        && painted.get(&k.id) != Some(&st.brush)
                    {
                        painted.insert(k.id, st.brush);
                    }
                }
                st.last = Some(p);
            }
            (Some(Drag::Select { start, additive }), Some(p)) if resp.dragged() => {
                let sel = Rect::from_two_pos(start, p);
                let mut set = if additive {
                    st.base_selection.clone()
                } else {
                    BTreeSet::new()
                };
                set.extend(keys_in_rect(&g, layout, sel));
                st.selection = set;
                ui.painter().rect(
                    sel,
                    CornerRadius::same(4),
                    with_alpha(pal.accent, 30),
                    Stroke::new(1.0, pal.accent),
                    StrokeKind::Inside,
                );
            }
            _ => {}
        }
        if resp.drag_stopped() {
            if matches!(st.drag, Some(Drag::Paint)) {
                flush = true;
            }
            st.drag = None;
            st.last = None;
        }
        if resp.clicked()
            && let Some(k) = ptr.and_then(|p| key_at(&g, layout, p))
        {
            if shift {
                if !st.selection.remove(&k.id) {
                    st.selection.insert(k.id);
                }
            } else if st.tool == Tool::Eyedropper {
                st.brush = color_of(k.id);
                st.tool = Tool::Paint;
            } else if color_of(k.id) != st.brush {
                painted.insert(k.id, st.brush);
                flush = true;
            }
        }

        // Rendering (uses the optimistic state plus this frame's strokes).
        illustration::paint_chassis(ui.painter(), &g, &pal);
        let hovered = ptr
            .filter(|_| resp.hovered() || resp.dragged())
            .and_then(|p| key_at(&g, layout, p))
            .map(|k| k.id);
        for cell in &layout.keys {
            let c = painted
                .get(&cell.id)
                .copied()
                .unwrap_or_else(|| color_of(cell.id));
            let lit = c != Rgb::BLACK;
            let col = c32(c);
            let selected = st.selection.contains(&cell.id);
            let paint = KeyPaint {
                fill: if lit {
                    col
                } else {
                    mix(pal.card, pal.raised, 0.6)
                },
                label: if lit {
                    text_on(col)
                } else {
                    with_alpha(pal.muted, 170)
                },
                stroke: if selected {
                    Stroke::new(2.0, pal.accent)
                } else if hovered == Some(cell.id) {
                    Stroke::new(1.5, pal.text)
                } else {
                    Stroke::new(
                        1.0,
                        if lit {
                            with_alpha(Color32::BLACK, 70)
                        } else {
                            with_alpha(pal.border, 200)
                        },
                    )
                },
                glow: lit.then_some(col),
            };
            illustration::paint_key(
                ui.painter(),
                illustration::key_rect(&g, cell),
                &cell.label,
                paint,
                g.unit,
            );
        }
        ui.add_space(10.0);
        tools(ui, st, layout, &mut painted, &mut flush);
    });

    if !painted.is_empty() || flush {
        out.push(Action::Apply {
            dev: id,
            setting: Setting::PerKey(painted),
            debounce: !flush,
        });
    }
}

fn brush_bar(ui: &mut Ui, st: &mut PerKeyState) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
        widgets::muted(ui, "Brush");
        let mut c = c32(st.brush);
        if color_edit_button_srgba(ui, &mut c, Alpha::Opaque).changed() {
            st.brush = rgb(c);
        }
        let editing = ui.memory(|m| m.has_focus(ui.id().with("brush-hex")));
        if !editing {
            st.hex = st.brush.to_hex();
        }
        let r = ui.add(
            TextEdit::singleline(&mut st.hex)
                .id(ui.id().with("brush-hex"))
                .desired_width(76.0)
                .font(FontId::monospace(13.0)),
        );
        if r.changed()
            && let Some(c) = Rgb::from_hex(&st.hex)
        {
            st.brush = c;
        }
        ui.add_space(6.0);
        for (r, g, b) in [
            (255, 59, 48),
            (255, 120, 20),
            (255, 176, 32),
            (255, 230, 60),
            (68, 214, 44),
            (0, 212, 170),
            (0, 184, 252),
            (40, 100, 255),
            (139, 92, 246),
            (232, 93, 208),
            (255, 255, 255),
        ] {
            let c = Rgb::new(r, g, b);
            if widgets::swatch(ui, c32(c), 22.0, c == st.brush).clicked() {
                st.brush = c;
                st.tool = Tool::Paint;
            }
        }
    });
}

fn tools(
    ui: &mut Ui,
    st: &mut PerKeyState,
    layout: &KeyboardLayout,
    painted: &mut std::collections::BTreeMap<u16, Rgb>,
    flush: &mut bool,
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
        if widgets::secondary(ui, None, "Fill all").clicked() {
            painted.extend(layout.keys.iter().map(|k| (k.id, st.brush)));
            *flush = true;
        }
        if widgets::secondary(ui, None, "Rainbow").clicked() {
            painted.extend(rainbow(layout));
            *flush = true;
        }
        if widgets::secondary(ui, None, "Clear").clicked() {
            painted.extend(layout.keys.iter().map(|k| (k.id, Rgb::BLACK)));
            *flush = true;
        }
        let eyedrop = st.tool == Tool::Eyedropper;
        if widgets::button(
            ui,
            if eyedrop {
                Btn::Primary
            } else {
                Btn::Secondary
            },
            Some(Icon::Eyedropper),
            "Pick colour",
        )
        .clicked()
        {
            st.tool = if eyedrop {
                Tool::Paint
            } else {
                Tool::Eyedropper
            };
        }
        ui.add_space(10.0);
        let n = st.selection.len();
        ui.add_enabled_ui(n > 0, |ui| {
            if widgets::secondary(
                ui,
                None,
                &if n == 0 {
                    "Fill selection".to_string()
                } else {
                    format!("Fill {n} selected")
                },
            )
            .clicked()
            {
                painted.extend(st.selection.iter().map(|k| (*k, st.brush)));
                *flush = true;
            }
            if widgets::button(ui, Btn::Ghost, None, "Deselect").clicked() {
                st.selection.clear();
            }
        });
    });
    ui.add_space(2.0);
    widgets::caption(
        ui,
        "Click or drag to paint · Shift + drag to select a rectangle · Shift + click toggles a key",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn rainbow_covers_every_key_with_distinct_ends() {
        let l = gearwise_core::keys::ansi_tkl();
        let m = rainbow(&l);
        assert_eq!(m.len(), l.keys.len());
        let left = m[&0x35]; // `
        let right = m[&0x4f]; // right arrow
        assert_ne!(left, right);
    }

    #[test]
    fn rectangle_selection_picks_intersecting_keys() {
        let l = gearwise_core::keys::ansi_tkl();
        let g = illustration::keyboard_geom(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 300.0)),
            &l,
        );
        let esc = l.keys.iter().find(|k| k.id == 0x29).unwrap();
        let r = illustration::key_rect(&g, esc);
        let set = keys_in_rect(&g, &l, r.shrink(2.0));
        assert_eq!(set, BTreeSet::from([0x29]));
        let all = keys_in_rect(&g, &l, g.chassis);
        assert_eq!(all.len(), l.keys.len());
    }
}
