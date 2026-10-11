//! Keyboard preview for the Lighting tab: the physical board, with every key lit by the zone
//! that owns it and the selected zone outlined around exactly its keys.

use std::borrow::Cow;
use std::collections::HashMap;

use egui::{Color32, CornerRadius, Rect, Stroke, StrokeKind, Ui};
use gearwise_core::keys::{KeyCell, KeyboardLayout};
use gearwise_core::model::{LightingEffect, LightingZoneCaps};

use super::lighting::{ALL_ZONES, text_on, zone_state};
use crate::fx;
use crate::illustration::{self, KeyPaint};
use crate::state::DeviceView;
use crate::theme::{Palette, c32, mix, with_alpha};

/// The physical board to draw: what the driver reports as `layout`, else its per-key
/// layout, else a generic full-size or tenkeyless ANSI board guessed from the name.
pub fn board_layout(dev: &DeviceView) -> Cow<'_, KeyboardLayout> {
    match dev.caps.layout.as_ref().or(dev.caps.per_key.as_ref()) {
        Some(l) => Cow::Borrowed(l),
        None => Cow::Owned(fallback_layout(dev.display_name())),
    }
}

/// Height for the board: tall enough to use the card's width, but no taller than what the
/// window leaves once `reserve` px of other content (and the card's own padding) are counted.
pub fn board_height(ui: &Ui, dev: &DeviceView, reserve: f32) -> f32 {
    let layout = board_layout(dev);
    let probe = illustration::keyboard_geom(
        Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(2000.0, 2000.0)),
        &layout,
    );
    let aspect = probe.total.width() / probe.total.height();
    let by_width = (ui.available_width() - 8.0) / aspect;
    let room = (crate::widgets::viewport_height(ui) - reserve).max(240.0);
    by_width.min(room).clamp(200.0, 460.0)
}

/// Layout to draw when the device doesn't report one.
fn fallback_layout(name: &str) -> KeyboardLayout {
    let n = name.to_ascii_lowercase();
    let compact = ["tkl", "tenkeyless", "mini", "60", "65", "75"]
        .iter()
        .any(|w| n.contains(w));
    if compact {
        gearwise_core::keys::ansi_tkl()
    } else {
        gearwise_core::keys::ansi_full()
    }
}

/// Zones that each light a vertical slice of the keys, left to right ("Zone 1" ... "Zone 5")
/// on boards that don't say which keys they light.
fn key_slices(zones: &[LightingZoneCaps]) -> Vec<&LightingZoneCaps> {
    let slices: Vec<&LightingZoneCaps> = zones
        .iter()
        .filter(|z| z.name.to_ascii_lowercase().starts_with("zone "))
        .collect();
    if slices.len() >= 2 {
        slices
    } else {
        Vec::new()
    }
}

/// An indicator-LED zone: lights no keys on a board whose other zones do.
fn is_indicator(z: &LightingZoneCaps) -> bool {
    z.keys.is_empty() && {
        let n = z.name.to_ascii_lowercase();
        n.contains("indicator") || n.contains("lock")
    }
}

/// Where a zone sits in the preview.
enum Source<'a> {
    /// Each key belongs to the zone that lists it.
    Listed(HashMap<u16, &'a LightingZoneCaps>),
    /// Vertical slices left to right.
    Slices(Vec<&'a LightingZoneCaps>),
    /// One zone colours all keys.
    All(Option<&'a LightingZoneCaps>),
}

impl<'a> Source<'a> {
    fn new(zones: &'a [LightingZoneCaps]) -> Self {
        if zones.iter().any(|z| !z.keys.is_empty()) {
            return Source::Listed(
                zones
                    .iter()
                    .flat_map(|z| z.keys.iter().map(move |k| (*k, z)))
                    .collect(),
            );
        }
        let slices = key_slices(zones);
        if !slices.is_empty() {
            return Source::Slices(slices);
        }
        Source::All(
            zones
                .iter()
                .find(|z| z.name.to_ascii_lowercase().contains("key"))
                .or(zones.first()),
        )
    }

    /// The zone that colours `cell`, given the layout's width.
    fn zone_of(&self, cell: &KeyCell, width: f32) -> Option<&'a LightingZoneCaps> {
        match self {
            Source::Listed(map) => map.get(&cell.id).copied(),
            Source::Slices(s) => {
                let x = (cell.x + cell.w / 2.0) / width;
                s.get(((x * s.len() as f32) as usize).min(s.len().saturating_sub(1)))
                    .copied()
            }
            Source::All(z) => *z,
        }
    }

    /// Does `z` colour keys (as opposed to glowing around the case)?
    fn colours_keys(&self, z: &LightingZoneCaps) -> bool {
        match self {
            Source::Listed(_) => !z.keys.is_empty(),
            Source::Slices(s) => s.iter().any(|s| s.id == z.id),
            Source::All(k) => k.is_some_and(|k| k.id == z.id),
        }
    }
}

pub fn keyboard_preview(
    ui: &mut Ui,
    area: Rect,
    dev: &DeviceView,
    selected: &LightingZoneCaps,
    t: f32,
    pal: &Palette,
) {
    let layout = board_layout(dev);
    let g = illustration::keyboard_geom(area, &layout);
    let zones = &dev.caps.lighting;
    let source = Source::new(zones);
    let sample = |z: &LightingZoneCaps, pos: f32, seed: u32| {
        let zs = zone_state(dev, z.id);
        c32(fx::sample(&zs.effect, zs.brightness, t, pos, seed))
    };
    let lit_zone = |z: &LightingZoneCaps| zone_state(dev, z.id).effect != LightingEffect::Off;

    // Zones that colour no keys glow around the case, or light the indicator LEDs.
    let mut indicator = None;
    for z in zones.iter().filter(|z| !source.colours_keys(z)) {
        if !lit_zone(z) {
            continue;
        }
        let c = sample(z, 0.5, u32::from(z.id));
        if g.style.top_bar && is_indicator(z) {
            indicator = Some(c);
            continue;
        }
        for (w, a) in [(30.0, 16u8), (20.0, 30), (12.0, 56), (5.0, 120)] {
            ui.painter().rect_stroke(
                g.chassis.expand(w * 0.35),
                CornerRadius::same((g.unit * 0.5) as u8),
                Stroke::new(w, with_alpha(c, a)),
                StrokeKind::Middle,
            );
        }
    }
    illustration::paint_chassis(ui.painter(), &g, pal, indicator);

    // The selected zone's keys, outlined as a group underneath the caps.
    let width = layout.width().max(1.0);
    let mut whole = selected.id == ALL_ZONES;
    let mut ring_indicator = false;
    if !whole {
        if selected.keys.is_empty() {
            if g.style.top_bar && is_indicator(selected) {
                ring_indicator = true;
            } else {
                whole = !matches!(source, Source::Slices(_) | Source::Listed(_))
                    || !source.colours_keys(selected);
            }
        } else {
            let rects: Vec<Rect> = layout
                .keys
                .iter()
                .filter(|c| selected.keys.contains(&c.id))
                .map(|c| illustration::key_rect(&g, c))
                .collect();
            illustration::paint_zone_outline(ui.painter(), &g, &rects, pal);
        }
    }
    if let Source::Slices(slices) = &source
        && let Some(i) = slices.iter().position(|z| z.id == selected.id)
    {
        let rects: Vec<Rect> = layout
            .keys
            .iter()
            .filter(|c| {
                source
                    .zone_of(c, width)
                    .is_some_and(|z| z.id == slices[i].id)
            })
            .map(|c| illustration::key_rect(&g, c))
            .collect();
        illustration::paint_zone_outline(ui.painter(), &g, &rects, pal);
    }

    for cell in &layout.keys {
        let color = source.zone_of(cell, width).map_or(Color32::BLACK, |z| {
            sample(z, (cell.x + cell.w / 2.0) / width, u32::from(cell.id))
        });
        let lit = color != Color32::BLACK;
        let paint = KeyPaint {
            fill: if lit {
                color
            } else {
                mix(pal.card, pal.raised, 0.6)
            },
            label: if lit {
                text_on(color)
            } else {
                with_alpha(pal.muted, 160)
            },
            stroke: Stroke::new(
                1.0,
                if lit {
                    with_alpha(Color32::BLACK, 70)
                } else {
                    with_alpha(pal.border, 200)
                },
            ),
            glow: lit.then_some(color),
        };
        illustration::paint_key(
            ui.painter(),
            illustration::key_rect(&g, cell),
            &cell.label,
            paint,
            g.unit,
        );
    }

    if ring_indicator && let Some(dots) = illustration::indicator_dots(&g) {
        let r =
            Rect::from_min_max(dots[0], dots[2]).expand2(egui::vec2(g.unit * 0.3, g.unit * 0.3));
        ui.painter().rect_stroke(
            r,
            CornerRadius::same((g.unit * 0.3) as u8),
            Stroke::new(1.5, pal.accent),
            StrokeKind::Outside,
        );
    }
    if whole {
        ui.painter().rect_stroke(
            g.chassis.expand(3.0),
            CornerRadius::same((g.unit * 0.5) as u8),
            Stroke::new(1.5, pal.accent),
            StrokeKind::Outside,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gearwise_core::keys::ansi_full;

    fn zone(id: u8, name: &str, keys: Vec<u16>) -> LightingZoneCaps {
        LightingZoneCaps {
            id,
            name: name.into(),
            keys,
            ..Default::default()
        }
    }

    #[test]
    fn listed_keys_beat_the_slice_heuristic() {
        let zones = vec![
            zone(1, "Zone 1", vec![0x04]),
            zone(2, "Zone 2", vec![0x05]),
            zone(0, "Win-lock indicator", vec![]),
        ];
        let src = Source::new(&zones);
        assert!(matches!(src, Source::Listed(_)));
        let layout = ansi_full();
        let a = layout.keys.iter().find(|k| k.id == 0x04).unwrap();
        let z = src.zone_of(a, layout.width()).unwrap();
        assert_eq!(z.id, 1);
        // A key no zone lists is unlit, and the indicator colours no keys.
        let space = layout.keys.iter().find(|k| k.id == 0x2c).unwrap();
        assert!(src.zone_of(space, layout.width()).is_none());
        assert!(!src.colours_keys(&zones[2]));
        assert!(is_indicator(&zones[2]));
    }

    #[test]
    fn without_listed_keys_zones_slice_the_board_left_to_right() {
        let zones: Vec<_> = (1..=5)
            .map(|i| zone(i, &format!("Zone {i}"), vec![]))
            .collect();
        let src = Source::new(&zones);
        assert!(matches!(src, Source::Slices(_)));
        let layout = ansi_full();
        let esc = layout.keys.iter().find(|k| k.id == 0x29).unwrap();
        let right = layout.keys.iter().find(|k| k.id == 0x63).unwrap();
        assert_eq!(src.zone_of(esc, layout.width()).unwrap().id, 1);
        assert_eq!(src.zone_of(right, layout.width()).unwrap().id, 5);
    }

    #[test]
    fn a_single_keys_zone_colours_the_whole_board() {
        let zones = vec![zone(0, "Keys", vec![]), zone(1, "Underglow", vec![])];
        let src = Source::new(&zones);
        let layout = ansi_full();
        assert_eq!(src.zone_of(&layout.keys[0], 22.5).unwrap().id, 0);
        assert!(src.colours_keys(&zones[0]) && !src.colours_keys(&zones[1]));
    }

    #[test]
    fn board_prefers_the_reported_layout() {
        let layout = KeyboardLayout {
            name: "K55 RGB PRO".into(),
            keys: ansi_full().keys,
        };
        let caps = gearwise_core::device::Capabilities {
            layout: Some(layout.clone()),
            per_key: Some(gearwise_core::keys::ansi_tkl()),
            ..Default::default()
        };
        let view = crate::state::DeviceView::new(crate::worker::DeviceSnapshot {
            descriptor: gearwise_core::demo::devices()[0].descriptor().clone(),
            capabilities: caps,
            state: Default::default(),
        });
        assert_eq!(board_layout(&view).name, "K55 RGB PRO");
        let mut v2 = view;
        v2.caps.layout = None;
        assert_eq!(board_layout(&v2).name, "ANSI tenkeyless");
        v2.caps.per_key = None;
        assert_eq!(board_layout(&v2).keys.len(), 104);
    }
}
