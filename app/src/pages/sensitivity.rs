//! Sensitivity tab: DPI stages, the scale visualisation and per-stage editing.

use egui::{
    Align, Align2, Color32, CornerRadius, DragValue, FontId, Layout, Sense, Slider, Stroke,
    StrokeKind, Ui, pos2, vec2,
};
use gearwise_core::device::Setting;
use gearwise_core::model::{DpiCaps, DpiStage, DpiState};

use crate::icons::{self, Icon};
use crate::state::{Action, DeviceView};
use crate::theme::{Palette, mix, palette, stage_color, with_alpha};
use crate::widgets;

#[derive(Default)]
pub struct SensState {
    device: String,
    separate: bool,
}

/// Position of `dpi` on the scale bar, 0..=1, on a log axis so common values spread out.
pub fn scale_pos(dpi: u16, min: u16, max: u16) -> f32 {
    let (lo, hi) = (f32::from(min.max(1)).ln(), f32::from(max.max(2)).ln());
    ((f32::from(dpi.max(1)).ln() - lo) / (hi - lo).max(f32::EPSILON)).clamp(0.0, 1.0)
}

/// What an edit means for the device: send now or wait for the drag to settle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Commit {
    Now,
    Soon,
}

pub fn show(ui: &mut Ui, st: &mut SensState, dev: &DeviceView, out: &mut Vec<Action>) {
    let (Some(caps), Some(state)) = (&dev.caps.dpi, &dev.state.dpi) else {
        widgets::empty_hint(ui, Icon::Gauge, "This device does not report DPI");
        return;
    };
    if st.device != dev.desc.id {
        st.device = dev.desc.id.clone();
        st.separate = state.stages.iter().any(|s| s.x != s.y);
    }
    let pal = palette(ui.ctx());
    let mut dpi = state.clone();
    let mut commit: Option<Commit> = None;
    let mut new_active = None;

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            widgets::subtitle(ui, "Sensitivity range");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::caption(ui, format!("{} – {} DPI", caps.min, caps.max));
            });
        });
        ui.add_space(6.0);
        if let Some(i) = scale_bar(ui, caps, &dpi, &pal) {
            new_active = Some(i);
        }
    });
    ui.add_space(14.0);

    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            widgets::subtitle(ui, "Stages");
            widgets::caption(ui, format!("{} of {}", dpi.stages.len(), caps.max_stages));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let can_add = dpi.stages.len() < caps.max_stages as usize;
                if ui
                    .add_enabled_ui(can_add, |ui| {
                        widgets::primary(ui, Some(Icon::Plus), "Add stage")
                    })
                    .inner
                    .clicked()
                {
                    let last = dpi.stages.last().map_or(caps.min, |s| s.x);
                    let v = caps.snap(last.saturating_mul(2).min(caps.max));
                    dpi.stages.push(DpiStage::uniform(v));
                    commit = Some(Commit::Now);
                }
                if caps.separate_xy {
                    widgets::toggle(ui, &mut st.separate);
                    widgets::muted(ui, "Separate X / Y");
                }
            });
        });
        ui.add_space(8.0);
        let active = dpi.active as usize;
        let n = dpi.stages.len();
        let mut remove = None;
        for i in 0..n {
            if i > 0 {
                widgets::divider(ui);
            }
            let res = stage_row(
                ui,
                i,
                i == active,
                &mut dpi.stages[i],
                caps,
                st.separate,
                n > 1,
                &pal,
            );
            if res.activate {
                new_active = Some(i as u8);
            }
            if res.remove {
                remove = Some(i);
            }
            if let Some(c) = res.commit {
                commit = Some(commit.map_or(c, |old| if old == Commit::Now { old } else { c }));
            }
        }
        if let Some(i) = remove {
            dpi.stages.remove(i);
            if dpi.active as usize >= dpi.stages.len() || (i < dpi.active as usize) {
                dpi.active = dpi.active.saturating_sub(1).min(dpi.stages.len() as u8 - 1);
            }
            commit = Some(Commit::Now);
        }
    });

    let id = dev.desc.id.clone();
    if let Some(c) = commit {
        out.push(Action::Apply {
            dev: id,
            setting: Setting::Dpi(dpi),
            debounce: c == Commit::Soon,
        });
    } else if let Some(i) = new_active.filter(|i| *i != state.active) {
        out.push(Action::Apply {
            dev: id,
            setting: Setting::DpiActiveStage(i),
            debounce: false,
        });
    }
}

struct RowResult {
    commit: Option<Commit>,
    activate: bool,
    remove: bool,
}

#[allow(clippy::too_many_arguments)]
fn stage_row(
    ui: &mut Ui,
    i: usize,
    active: bool,
    stage: &mut DpiStage,
    caps: &DpiCaps,
    separate: bool,
    can_remove: bool,
    pal: &Palette,
) -> RowResult {
    let mut res = RowResult {
        commit: None,
        activate: false,
        remove: false,
    };
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        if chip(ui, i, active, pal)
            .on_hover_text(if active {
                "Active stage"
            } else {
                "Make this the active stage"
            })
            .clicked()
        {
            res.activate = true;
        }
        let slider_w = (ui.available_width() - 150.0 - 36.0 - 28.0).max(120.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            if separate {
                let before = *stage;
                axis(ui, "X", &mut stage.x, caps, slider_w, &mut res.commit);
                axis(ui, "Y", &mut stage.y, caps, slider_w, &mut res.commit);
                if before != *stage && res.commit.is_none() {
                    res.commit = Some(Commit::Soon);
                }
            } else {
                let before = stage.x;
                axis(ui, "", &mut stage.x, caps, slider_w, &mut res.commit);
                stage.y = stage.x;
                if before != stage.x && res.commit.is_none() {
                    res.commit = Some(Commit::Soon);
                }
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_enabled_ui(can_remove, |ui| {
                    widgets::icon_button(ui, Icon::Trash, "Remove stage")
                })
                .inner
                .clicked()
            {
                res.remove = true;
            }
            if active {
                widgets::badge(ui, "Active", pal.accent);
            }
        });
    });
    ui.add_space(4.0);
    res
}

fn axis(
    ui: &mut Ui,
    label: &str,
    v: &mut u16,
    caps: &DpiCaps,
    slider_w: f32,
    commit: &mut Option<Commit>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        if !label.is_empty() {
            ui.allocate_ui_with_layout(
                vec2(14.0, 20.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    widgets::muted(ui, label);
                },
            );
        }
        let mut x = *v;
        ui.spacing_mut().slider_width = slider_w - if label.is_empty() { 0.0 } else { 26.0 };
        let s = ui.add(
            Slider::new(&mut x, caps.min..=caps.max)
                .logarithmic(true)
                .show_value(false),
        );
        if s.changed() {
            *commit = Some(Commit::Soon);
        }
        if s.drag_stopped() || s.lost_focus() {
            *commit = Some(Commit::Now);
        }
        let d = ui.add(
            DragValue::new(&mut x)
                .range(caps.min..=caps.max)
                .speed(f32::from(caps.step.max(1)))
                .suffix(" DPI"),
        );
        if d.changed() {
            *commit = Some(Commit::Soon);
        }
        if d.drag_stopped() || d.lost_focus() {
            *commit = Some(Commit::Now);
        }
        *v = caps.snap(x);
    });
}

fn chip(ui: &mut Ui, i: usize, active: bool, pal: &Palette) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
    let c = stage_color(i);
    let p = ui.painter();
    let center = rect.center();
    if active {
        p.circle_filled(center, 19.0, with_alpha(c, 50));
        p.circle_filled(center, 15.0, c);
    } else {
        p.circle(
            center,
            15.0,
            with_alpha(c, if resp.hovered() { 70 } else { 36 }),
            Stroke::new(1.5, with_alpha(c, 170)),
        );
    }
    let luma = 0.299 * f32::from(c.r()) + 0.587 * f32::from(c.g()) + 0.114 * f32::from(c.b());
    let ink = if active {
        if luma > 150.0 {
            Color32::from_rgb(0x0b, 0x10, 0x14)
        } else {
            Color32::WHITE
        }
    } else if pal.dark {
        c
    } else {
        mix(c, Color32::BLACK, 0.4)
    };
    p.text(
        center,
        Align2::CENTER_CENTER,
        (i + 1).to_string(),
        FontId::proportional(14.0),
        ink,
    );
    resp
}

/// Log-scale bar with a marker per stage; returns a clicked stage.
fn scale_bar(ui: &mut Ui, caps: &DpiCaps, dpi: &DpiState, pal: &Palette) -> Option<u8> {
    let (rect, base) = ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::hover());
    let track = egui::Rect::from_min_max(
        pos2(rect.min.x + 14.0, rect.min.y + 44.0),
        pos2(rect.max.x - 14.0, rect.min.y + 52.0),
    );
    let x_at = |v: u16| track.min.x + scale_pos(v, caps.min, caps.max) * track.width();
    let p = ui.painter();
    p.rect_filled(track, CornerRadius::same(4), pal.input);
    p.rect_stroke(
        track,
        CornerRadius::same(4),
        Stroke::new(1.0, pal.border),
        StrokeKind::Inside,
    );

    // Fill up to the active stage.
    if let Some(cur) = dpi.current() {
        let fill = egui::Rect::from_min_max(track.min, pos2(x_at(cur.x), track.max.y));
        p.rect_filled(fill, CornerRadius::same(4), with_alpha(pal.accent, 150));
    }
    // Reference ticks.
    for t in [100u16, 200, 400, 800, 1600, 3200, 6400, 12_800, 25_600] {
        if t < caps.min || t > caps.max {
            continue;
        }
        let x = x_at(t);
        p.vline(
            x,
            track.max.y + 4.0..=track.max.y + 9.0,
            Stroke::new(1.0, pal.border),
        );
        p.text(
            pos2(x, track.max.y + 20.0),
            Align2::CENTER_CENTER,
            t.to_string(),
            FontId::proportional(11.0),
            pal.muted,
        );
    }
    let mut clicked = None;
    let mut hovered = None;
    for (i, s) in dpi.stages.iter().enumerate() {
        let active = i == dpi.active as usize;
        let center = pos2(x_at(s.x), track.center().y);
        let hit = icons::square(center, 26.0);
        let resp = ui.interact(hit, base.id.with(i), Sense::click());
        let c = stage_color(i);
        let p = ui.painter();
        if active {
            p.circle_filled(center, 15.0, with_alpha(c, 55));
            p.circle(center, 10.0, c, Stroke::new(2.0, pal.card));
        } else {
            p.circle(
                center,
                if resp.hovered() { 9.0 } else { 7.0 },
                c,
                Stroke::new(2.0, pal.card),
            );
        }
        if resp.clicked() {
            clicked = Some(i as u8);
        }
        if resp.hovered() {
            hovered = Some((i, center));
        }
    }
    let label_for = hovered.or_else(|| {
        let i = dpi.active as usize;
        dpi.stages
            .get(i)
            .map(|s| (i, pos2(x_at(s.x), track.center().y)))
    });
    if let Some((i, c)) = label_for {
        let s = dpi.stages[i];
        let text = if s.x == s.y {
            format!("{} DPI", s.x)
        } else {
            format!("{} × {} DPI", s.x, s.y)
        };
        let p = ui.painter();
        let g = p.layout_no_wrap(text, FontId::proportional(13.0), pal.text);
        let size = g.size() + vec2(18.0, 10.0);
        let x = (c.x - size.x / 2.0).clamp(rect.min.x, rect.max.x - size.x);
        let r = egui::Rect::from_min_size(pos2(x, track.min.y - size.y - 14.0), size);
        p.rect(
            r,
            CornerRadius::same(8),
            pal.raised,
            Stroke::new(1.0, stage_color(i)),
            StrokeKind::Inside,
        );
        p.galley(r.min + vec2(9.0, 5.0), g, pal.text);
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_logarithmic_and_clamped() {
        assert_eq!(scale_pos(100, 100, 25_600), 0.0);
        assert_eq!(scale_pos(25_600, 100, 25_600), 1.0);
        // Each doubling covers the same distance.
        let d1 = scale_pos(800, 100, 25_600) - scale_pos(400, 100, 25_600);
        let d2 = scale_pos(3200, 100, 25_600) - scale_pos(1600, 100, 25_600);
        assert!((d1 - d2).abs() < 1e-4);
        assert_eq!(scale_pos(50, 100, 25_600), 0.0);
        assert_eq!(scale_pos(60_000, 100, 25_600), 1.0);
    }
}
