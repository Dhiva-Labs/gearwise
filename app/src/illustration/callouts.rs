//! G HUB-style callouts: a marker on the device, a leader line and a label pill in the margin.
//! Pills are sized to their text, stacked by the height of their anchor, and the assignment of
//! labels to slots is untangled so leader lines never cross each other or another marker.

use egui::{
    CornerRadius, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};

use super::roles::Role;
use crate::theme::{Palette, mix, with_alpha};
use crate::widgets::fit_text;

pub const PILL_H: f32 = 40.0;
/// Smallest vertical gap between two pills.
const GAP: f32 = 6.0;
/// Horizontal space between the art's box and the pill columns (the leader's bend sits here).
pub const LEAD: f32 = 30.0;
const MIN_PILL_W: f32 = 104.0;
/// Radius around a marker that other leader lines keep clear of.
const CLEAR: f32 = 6.5;

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

/// Which margin a button's label goes in, given the art's `rect`.
pub fn side_for(role: Option<Role>, anchor: Pos2, art: Rect) -> Side {
    let centre_x = art.center().x;
    match role {
        Some(
            Role::Left | Role::TiltLeft | Role::Back | Role::Forward | Role::Sniper | Role::Grid(_),
        ) => Side::Left,
        // The G502's G7/G8 wedges sit left of its left button; elsewhere the DPI keys are central.
        Some(Role::DpiUp | Role::DpiDown) if anchor.x < centre_x - art.width() * 0.1 => Side::Left,
        Some(
            Role::Right
            | Role::TiltRight
            | Role::Middle
            | Role::DpiUp
            | Role::DpiDown
            | Role::DpiCycle
            | Role::Profile,
        ) => Side::Right,
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
        return (0..n)
            .map(|i| lo + (hi - lo) * i as f32 / (n - 1) as f32)
            .collect();
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

/// Height the pills of one side need so they all fit without overlapping.
pub fn height_needed(count: usize) -> f32 {
    count as f32 * (PILL_H + GAP)
}

// --- Geometry of the leader lines ------------------------------------------------------

type Line = [Pos2; 3];

fn cross(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Do segments `p1-p2` and `p3-p4` properly intersect?
fn segments_cross(p1: Pos2, p2: Pos2, p3: Pos2, p4: Pos2) -> bool {
    let (d1, d2) = (cross(p3, p4, p1), cross(p3, p4, p2));
    let (d3, d4) = (cross(p1, p2, p3), cross(p1, p2, p4));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

fn lines_cross(a: &Line, b: &Line) -> bool {
    (0..2).any(|i| (0..2).any(|j| segments_cross(a[i], a[i + 1], b[j], b[j + 1])))
}

fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = if ab.length_sq() < 1e-6 {
        0.0
    } else {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
    };
    (p - (a + ab * t)).length()
}

/// Does `line` run closer than [`CLEAR`] to `marker`?
fn runs_over(line: &Line, marker: Pos2) -> bool {
    (0..2).any(|s| dist_to_segment(marker, line[s], line[s + 1]) < CLEAR)
}

/// One label's place: its pill and the leader from the marker to it.
struct Laid {
    item: usize,
    pill: Rect,
    line: Line,
}

/// Cost of putting label `item` (by anchor order) into `slot`, apart from crossings: how far it
/// sits from its marker, and every foreign marker its leader runs over.
const MOVE_COST: f32 = 0.02;
const OVER_COST: f32 = 300.0;
const CROSS_COST: f32 = 1000.0;
/// Search effort cap, so a crowded side can never stall a frame.
const NODE_LIMIT: usize = 60_000;

struct Search<'a> {
    n: usize,
    single: &'a [Vec<f32>],
    crosses: &'a dyn Fn(usize, usize, usize, usize) -> bool,
    best: f32,
    best_perm: Vec<usize>,
    nodes: usize,
}

impl Search<'_> {
    /// Fill slots in order, trying cheap labels first and pruning against the best so far.
    fn dfs(&mut self, perm: &mut Vec<usize>, used: &mut [bool], cost: f32) {
        self.nodes += 1;
        let slot = perm.len();
        if slot == self.n {
            if cost < self.best {
                self.best = cost;
                self.best_perm = perm.clone();
            }
            return;
        }
        let mut cands: Vec<usize> = (0..self.n).filter(|i| !used[*i]).collect();
        cands.sort_by(|a, b| self.single[slot][*a].total_cmp(&self.single[slot][*b]));
        for item in cands {
            if self.nodes > NODE_LIMIT {
                return;
            }
            let mut next = cost + self.single[slot][item];
            for (s2, i2) in perm.iter().enumerate() {
                if (self.crosses)(s2, *i2, slot, item) {
                    next += CROSS_COST;
                }
            }
            if next >= self.best {
                continue;
            }
            perm.push(item);
            used[item] = true;
            self.dfs(perm, used, next);
            perm.pop();
            used[item] = false;
        }
    }
}

/// Lay out one side: sort by anchor height, stack the slots, then find the assignment of labels
/// to slots with the fewest crossings and markers run over (an exact search, capped).
/// `obstacles` are lines already laid out on the other side.
fn layout_side(
    items: &[Callout],
    idx: &[usize],
    side: Side,
    column: Rect,
    bend_x: f32,
    bounds: (f32, f32),
    obstacles: &[Line],
) -> Vec<Laid> {
    let mut order: Vec<usize> = idx.to_vec();
    order.sort_by(|a, b| {
        items[*a]
            .anchor
            .y
            .total_cmp(&items[*b].anchor.y)
            .then(items[*a].anchor.x.total_cmp(&items[*b].anchor.x))
    });
    let n = order.len();
    let desired: Vec<f32> = order.iter().map(|i| items[*i].anchor.y).collect();
    let slots = stack(&desired, PILL_H + GAP, bounds.0, bounds.1);
    let edge_x = if side == Side::Left {
        column.max.x
    } else {
        column.min.x
    };
    // The leader of the k-th item (by anchor height) sitting in `slot`.
    let line_for = |k: usize, slot: usize| -> Line {
        [
            items[order[k]].anchor,
            pos2(bend_x, slots[slot]),
            pos2(edge_x, slots[slot]),
        ]
    };
    let single: Vec<Vec<f32>> = (0..n)
        .map(|slot| {
            (0..n)
                .map(|k| {
                    let line = line_for(k, slot);
                    let over = (0..n)
                        .filter(|j| *j != k && runs_over(&line, items[order[*j]].anchor))
                        .count() as f32;
                    let blocked = obstacles.iter().filter(|o| lines_cross(&line, o)).count() as f32;
                    over * OVER_COST
                        + blocked * CROSS_COST
                        + (slots[slot] - items[order[k]].anchor.y).abs() * MOVE_COST
                })
                .collect()
        })
        .collect();
    let crosses = |s1: usize, k1: usize, s2: usize, k2: usize| {
        lines_cross(&line_for(k1, s1), &line_for(k2, s2))
    };
    let identity: Vec<usize> = (0..n).collect();
    let identity_cost: f32 = (0..n).map(|s| single[s][s]).sum::<f32>()
        + (0..n)
            .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
            .filter(|(a, b)| crosses(*a, *a, *b, *b))
            .count() as f32
            * CROSS_COST;
    let mut search = Search {
        n,
        single: &single,
        crosses: &crosses,
        best: identity_cost + 1e-3,
        best_perm: identity,
        nodes: 0,
    };
    search.dfs(&mut Vec::with_capacity(n), &mut vec![false; n], 0.0);
    let perm = search.best_perm;
    perm.iter()
        .enumerate()
        .map(|(slot, k)| {
            let y = slots[slot];
            let pill = Rect::from_min_max(
                pos2(column.min.x, y - PILL_H / 2.0),
                pos2(column.max.x, y + PILL_H / 2.0),
            );
            Laid {
                item: order[*k],
                pill,
                line: line_for(*k, slot),
            }
        })
        .collect()
}

// --- Measuring and drawing -------------------------------------------------------------

/// Width one label's pill wants for its text, capped at `max`.
fn pill_width(p: &Painter, c: &Callout, max: f32) -> f32 {
    let w = |t: &str, size: f32| {
        p.layout_no_wrap(
            t.to_string(),
            FontId::proportional(size),
            egui::Color32::WHITE,
        )
        .size()
        .x
    };
    (w(&c.title, 12.5).max(w(&c.detail, 11.5)) + 22.0).clamp(MIN_PILL_W, max.max(MIN_PILL_W))
}

/// (left, right) column widths that fit every label's text, each capped at `max`.
pub fn column_widths(p: &Painter, items: &[Callout], max: f32) -> (f32, f32) {
    let mut out = (MIN_PILL_W, MIN_PILL_W);
    for c in items {
        let w = pill_width(p, c, max);
        match c.side {
            Side::Left => out.0 = out.0.max(w),
            Side::Right => out.1 = out.1.max(w),
        }
    }
    out
}

/// Draw the callouts around `art` inside `bounds` and report hover and clicks. `widths` are
/// the pill columns (see [`column_widths`]); `hover_hint` is a button hovered elsewhere (the
/// list), highlighted here too.
#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    bounds: Rect,
    art: Rect,
    widths: (f32, f32),
    items: &[Callout],
    selected: Option<u16>,
    hover_hint: Option<u16>,
    pal: &Palette,
) -> Interaction {
    let mut out = Interaction::default();
    let base = ui.id().with("callouts");
    let shown = |c: &Callout| !c.compact || selected == Some(c.id) || hover_hint == Some(c.id);

    let left_col = Rect::from_min_max(
        pos2(art.min.x - LEAD - widths.0, bounds.min.y),
        pos2(art.min.x - LEAD, bounds.max.y),
    );
    let right_col = Rect::from_min_max(
        pos2(art.max.x + LEAD, bounds.min.y),
        pos2(art.max.x + LEAD + widths.1, bounds.max.y),
    );
    let ys = (bounds.min.y + PILL_H / 2.0, bounds.max.y - PILL_H / 2.0);
    let mut laid: Vec<Laid> = Vec::new();
    let mut obstacles: Vec<Line> = Vec::new();
    for (side, col, bend) in [
        (Side::Left, left_col, art.min.x - LEAD * 0.45),
        (Side::Right, right_col, art.max.x + LEAD * 0.45),
    ] {
        let idx: Vec<usize> = (0..items.len())
            .filter(|i| items[*i].side == side && shown(&items[*i]))
            .collect();
        let side_laid = layout_side(items, &idx, side, col, bend, ys, &obstacles);
        obstacles.extend(side_laid.iter().map(|l| l.line));
        laid.extend(side_laid);
    }

    // Interaction first, so painting can show the hover state of this very frame.
    let mut over = vec![false; items.len()];
    for (n, c) in items.iter().enumerate() {
        let sense = if c.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let dot = if c.compact {
            vec2(15.0, 15.0)
        } else {
            vec2(22.0, 22.0)
        };
        let mut resps = vec![ui.interact(
            Rect::from_center_size(c.anchor, dot),
            base.with(("dot", c.id)),
            sense,
        )];
        if let Some(l) = laid.iter().find(|l| l.item == n) {
            resps.push(ui.interact(l.pill, base.with(("pill", c.id)), sense));
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
    // Lines first so every pill and marker sits on top of them; active ones last.
    let mut order: Vec<&Laid> = laid.iter().collect();
    order.sort_by_key(|l| lit(l.item, &items[l.item]));
    for l in &order {
        let active = lit(l.item, &items[l.item]);
        p.add(Shape::line(
            l.line.to_vec(),
            Stroke::new(
                if active { 1.5 } else { 1.0 },
                if active {
                    with_alpha(pal.accent, 230)
                } else {
                    with_alpha(pal.muted, 130)
                },
            ),
        ));
    }
    for l in &order {
        let c = &items[l.item];
        let is_sel = selected == Some(c.id);
        let active = lit(l.item, c);
        if !c.compact {
            if active {
                p.circle_filled(c.anchor, 11.0, with_alpha(pal.accent, 44));
            }
            p.circle(
                c.anchor,
                if active { 5.5 } else { 4.5 },
                if c.custom || active {
                    pal.accent
                } else {
                    pal.card
                },
                Stroke::new(
                    1.5,
                    if active || c.custom {
                        pal.accent
                    } else {
                        pal.muted
                    },
                ),
            );
        }
        let fill = if active {
            mix(pal.card, pal.accent, 0.14)
        } else {
            pal.card
        };
        let border = if active {
            pal.accent
        } else {
            mix(pal.border, pal.text, 0.1)
        };
        p.rect(
            l.pill,
            CornerRadius::same(10),
            fill,
            Stroke::new(if is_sel { 1.6 } else { 1.0 }, border),
            StrokeKind::Inside,
        );
        let text_w = l.pill.width() - 20.0;
        let title = fit_text(
            &p,
            &c.title,
            FontId::proportional(12.5),
            if c.enabled { pal.text } else { pal.muted },
            text_w,
        );
        p.galley(l.pill.min + vec2(10.0, 6.0), title, pal.text);
        let detail_col = if c.custom {
            pal.accent_text()
        } else {
            pal.muted
        };
        let detail = fit_text(
            &p,
            &c.detail,
            FontId::proportional(11.5),
            detail_col,
            text_w,
        );
        p.galley(l.pill.min + vec2(10.0, 22.0), detail, detail_col);
    }
    out
}

#[cfg(test)]
mod tests;
