//! G HUB-style callouts: a marker on the device, a leader line and a label pill
//! in the margin, stacked so labels never overlap.

use egui::{CornerRadius, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};

use super::mouse::Role;
use crate::theme::{Palette, mix, with_alpha};
use crate::widgets::fit_text;

pub const PILL_H: f32 = 40.0;
const GAP: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Callout {
    pub id: u16,
    pub anchor: Pos2,
    pub side: Side,
    pub title: String,
    pub detail: String,
    /// The button has a non-default action (detail shown in the accent colour).
    pub custom: bool,
    /// Remappable; fixed buttons are shown but cannot be selected.
    pub enabled: bool,
    /// Crowded groups (the MMO grid) show a label only while hovered or selected.
    pub compact: bool,
}

#[derive(Default)]
pub struct Interaction {
    pub hovered: Option<u16>,
    pub clicked: Option<u16>,
}

/// Which margin a button's label goes in.
pub fn side_for(role: Option<Role>, anchor: Pos2, centre_x: f32) -> Side {
    match role {
        Some(Role::Left | Role::TiltLeft | Role::Back | Role::Forward | Role::Sniper | Role::Grid(_)) => Side::Left,
        Some(Role::Right | Role::TiltRight | Role::Middle | Role::DpiUp | Role::DpiDown | Role::DpiCycle | Role::Profile) => Side::Right,
        None => {
            if anchor.x < centre_x {
                Side::Left
            } else {
                Side::Right
            }
        }
    }
}

/// Vertical centres for labels wanting `desired` (ascending), at least `gap` apart
/// and inside `lo..=hi`. When the space is too small the labels are spread evenly.
pub fn stack(desired: &[f32], gap: f32, lo: f32, hi: f32) -> Vec<f32> {
    let n = desired.len();
    if n == 0 {
        return Vec::new();
    }
    if n > 1 && (hi - lo) < gap * (n - 1) as f32 {
        return (0..n).map(|i| lo + (hi - lo) * i as f32 / (n - 1) as f32).collect();
    }
    let mut y: Vec<f32> = desired.iter().map(|d| d.clamp(lo, hi)).collect();
    for i in 1..n {
        y[i] = y[i].max(y[i - 1] + gap);
    }
    if y[n - 1] > hi {
        y[n - 1] = hi;
        for i in (0..n - 1).rev() {
            y[i] = y[i].min(y[i + 1] - gap);
        }
    }
    y
}

/// Draw the callouts inside `bounds` and report hover and clicks. `hover_hint` is a
/// button hovered elsewhere (the list), highlighted here too.
pub fn show(ui: &mut Ui, bounds: Rect, pill_w: f32, items: &[Callout], selected: Option<u16>, hover_hint: Option<u16>, pal: &Palette) -> Interaction {
    let mut out = Interaction::default();
    let base = ui.id().with("callouts");
    let shown = |c: &Callout| !c.compact || selected == Some(c.id) || hover_hint == Some(c.id);

    let mut laid: Vec<(usize, Rect, [Pos2; 3])> = Vec::new();
    for side in [Side::Left, Side::Right] {
        let mut idx: Vec<usize> = (0..items.len()).filter(|i| items[*i].side == side && shown(&items[*i])).collect();
        idx.sort_by(|a, b| items[*a].anchor.y.total_cmp(&items[*b].anchor.y));
        let desired: Vec<f32> = idx.iter().map(|i| items[*i].anchor.y).collect();
        let ys = stack(&desired, PILL_H + GAP, bounds.min.y + PILL_H / 2.0, bounds.max.y - PILL_H / 2.0);
        for (i, y) in idx.into_iter().zip(ys) {
            let x = if side == Side::Left { bounds.min.x } else { bounds.max.x - pill_w };
            let pill = Rect::from_min_size(pos2(x, y - PILL_H / 2.0), vec2(pill_w, PILL_H));
            let edge = if side == Side::Left { pos2(pill.max.x, y) } else { pos2(pill.min.x, y) };
            let elbow = pos2(edge.x + if side == Side::Left { 18.0 } else { -18.0 }, y);
            laid.push((i, pill, [items[i].anchor, elbow, edge]));
        }
    }

    // Interaction first, so painting can show the hover state of this very frame.
    let mut over = vec![false; items.len()];
    for (n, c) in items.iter().enumerate() {
        let sense = if c.enabled { Sense::click() } else { Sense::hover() };
        let dot = if c.compact { vec2(15.0, 15.0) } else { vec2(22.0, 22.0) };
        let mut resps = vec![ui.interact(Rect::from_center_size(c.anchor, dot), base.with(("dot", c.id)), sense)];
        if let Some((_, pill, _)) = laid.iter().find(|(i, ..)| *i == n) {
            resps.push(ui.interact(*pill, base.with(("pill", c.id)), sense));
        }
        over[n] = resps.iter().any(|r| r.hovered());
        if over[n] {
            out.hovered = Some(c.id);
        }
        if c.enabled && resps.iter().any(|r| r.clicked()) {
            out.clicked = Some(c.id);
        }
    }

    let p = ui.painter().clone();
    let lit = |n: usize, c: &Callout| over[n] || selected == Some(c.id) || hover_hint == Some(c.id);
    // Compact items are marked by the part itself; just ring them when active.
    for (n, c) in items.iter().enumerate().filter(|(_, c)| c.compact) {
        if lit(n, c) {
            p.circle_stroke(c.anchor, 11.0, Stroke::new(1.5, pal.accent));
        }
    }
    for (i, pill, line) in &laid {
        let c = &items[*i];
        let is_sel = selected == Some(c.id);
        let lit = lit(*i, c);
        let line_col = if lit { with_alpha(pal.accent, 230) } else { with_alpha(pal.muted, 120) };
        p.add(Shape::line(line.to_vec(), Stroke::new(if lit { 1.5 } else { 1.0 }, line_col)));
        if !c.compact {
            if lit {
                p.circle_filled(c.anchor, 11.0, with_alpha(pal.accent, 44));
            }
            p.circle(c.anchor, if lit { 5.5 } else { 4.5 }, if c.custom || lit { pal.accent } else { pal.card }, Stroke::new(1.5, if lit || c.custom { pal.accent } else { pal.muted }));
        }
        let fill = if lit { mix(pal.card, pal.accent, 0.14) } else { pal.card };
        let border = if lit { pal.accent } else { mix(pal.border, pal.text, 0.1) };
        p.rect(*pill, CornerRadius::same(10), fill, Stroke::new(if is_sel { 1.6 } else { 1.0 }, border), StrokeKind::Inside);
        let text_w = pill.width() - 20.0;
        let title = fit_text(&p, &c.title, FontId::proportional(12.5), if c.enabled { pal.text } else { pal.muted }, text_w);
        p.galley(pill.min + vec2(10.0, 6.0), title, pal.text);
        let detail_col = if c.custom { pal.accent_text() } else { pal.muted };
        let detail = fit_text(&p, &c.detail, FontId::proportional(11.5), detail_col, text_w);
        p.galley(pill.min + vec2(10.0, 22.0), detail, detail_col);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(ys: &[f32], gap: f32, lo: f32, hi: f32) {
        assert!(ys.windows(2).all(|w| w[1] - w[0] >= gap - 1e-3), "{ys:?} overlap");
        assert!(ys.iter().all(|y| *y >= lo - 1e-3 && *y <= hi + 1e-3), "{ys:?} outside {lo}..{hi}");
    }

    #[test]
    fn spaced_labels_stay_where_they_want_to_be() {
        let ys = stack(&[50.0, 150.0, 300.0], 48.0, 20.0, 400.0);
        assert_eq!(ys, vec![50.0, 150.0, 300.0]);
    }

    #[test]
    fn crowded_labels_are_pushed_apart_in_order() {
        let ys = stack(&[100.0, 104.0, 106.0, 110.0], 48.0, 20.0, 400.0);
        valid(&ys, 48.0, 20.0, 400.0);
        assert!(ys[0] <= 100.0 + 1e-3 || ys[0] >= 20.0);
        assert!(ys.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn labels_near_the_bottom_shift_up_instead_of_leaving_the_box() {
        let ys = stack(&[380.0, 390.0, 395.0], 48.0, 20.0, 400.0);
        valid(&ys, 48.0, 20.0, 400.0);
        assert_eq!(*ys.last().unwrap(), 400.0);
    }

    #[test]
    fn too_many_labels_are_spread_evenly() {
        let ys = stack(&[10.0; 6], 48.0, 0.0, 100.0);
        assert_eq!(ys.len(), 6);
        assert!((ys[5] - 100.0).abs() < 1e-3 && ys[0].abs() < 1e-3);
        assert!(ys.windows(2).all(|w| (w[1] - w[0] - 20.0).abs() < 1e-3));
        assert!(stack(&[], 48.0, 0.0, 100.0).is_empty());
    }

    #[test]
    fn labels_go_to_the_side_the_part_lives_on() {
        let c = 100.0;
        assert_eq!(side_for(Some(Role::Back), pos2(10.0, 0.0), c), Side::Left);
        assert_eq!(side_for(Some(Role::Right), pos2(10.0, 0.0), c), Side::Right);
        assert_eq!(side_for(None, pos2(40.0, 0.0), c), Side::Left);
        assert_eq!(side_for(None, pos2(160.0, 0.0), c), Side::Right);
    }
}
