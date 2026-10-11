//! Reusable building blocks: cards, buttons, pills, segmented controls, toggles.

use std::sync::Arc;

use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align2, Color32, CornerRadius, FontId, Galley, Pos2, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, TextStyle, Ui, Vec2, pos2, vec2,
};
use gearwise_core::model::Battery;

use crate::icons::{self, Icon};
pub use crate::layout::{
    card, card_columns, divider, empty_hint, kv, page_scroll, setting_row, viewport_height,
};
use crate::theme::{self, Palette, mix, palette, with_alpha};

/// Small upper-case section label.
pub fn section(ui: &mut Ui, text: &str) -> Response {
    let pal = palette(ui.ctx());
    ui.label(
        RichText::new(text.to_uppercase())
            .text_style(theme::section())
            .color(pal.muted),
    )
}

pub fn muted(ui: &mut Ui, text: impl Into<String>) -> Response {
    let pal = palette(ui.ctx());
    ui.label(RichText::new(text.into()).color(pal.muted))
}

pub fn caption(ui: &mut Ui, text: impl Into<String>) -> Response {
    let pal = palette(ui.ctx());
    ui.label(
        RichText::new(text.into())
            .text_style(theme::caption())
            .color(pal.muted),
    )
}

/// Heading with a faux-bold weight (the bundled font only ships one weight).
pub fn bold(ui: &mut Ui, text: impl Into<String>, style: TextStyle, color: Color32) -> Response {
    let font = style.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(text.into(), font, color);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + vec2(0.7, 0.0), Sense::hover());
    ui.painter().galley(rect.min, galley.clone(), color);
    ui.painter()
        .galley(rect.min + vec2(0.7, 0.0), galley, color);
    resp
}

/// Page title that ends in an ellipsis (full text on hover) when wider than `max_w`.
pub fn title_fit(ui: &mut Ui, text: &str, max_w: f32) -> Response {
    let pal = palette(ui.ctx());
    let font = theme::title().resolve(ui.style());
    let natural = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), pal.text)
        .size()
        .x;
    let galley = fit_text(ui.painter(), text, font, pal.text, max_w - 1.0);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + vec2(0.7, 0.0), Sense::hover());
    ui.painter().galley(rect.min, galley.clone(), pal.text);
    ui.painter()
        .galley(rect.min + vec2(0.7, 0.0), galley, pal.text);
    if natural > max_w {
        resp.on_hover_text(text)
    } else {
        resp
    }
}

pub fn title(ui: &mut Ui, text: impl Into<String>) -> Response {
    let c = palette(ui.ctx()).text;
    bold(ui, text, theme::title(), c)
}

pub fn subtitle(ui: &mut Ui, text: impl Into<String>) -> Response {
    let c = palette(ui.ctx()).text;
    bold(ui, text, theme::subtitle(), c)
}

/// Single-line galley that ends in an ellipsis when it exceeds `max_width`.
pub fn fit_text(
    p: &egui::Painter,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_string(), font, color);
    job.wrap = TextWrapping::truncate_at_width(max_width.max(1.0));
    p.layout_job(job)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Btn {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// Disabled buttons: a quiet but visible plate and muted, still readable, text.
pub fn disabled_style(pal: &Palette, kind: Btn) -> (Color32, Stroke, Color32) {
    let plate = match kind {
        Btn::Primary | Btn::Secondary => mix(pal.raised, pal.window, 0.25),
        Btn::Ghost | Btn::Danger => Color32::TRANSPARENT,
    };
    (
        plate,
        Stroke::new(1.0, mix(pal.border, pal.window, 0.2)),
        mix(pal.muted, pal.raised, 0.25),
    )
}

pub fn button(ui: &mut Ui, kind: Btn, icon: Option<Icon>, text: &str) -> Response {
    let pal = palette(ui.ctx());
    let font = TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, Color32::WHITE);
    let icon_w = if icon.is_some() {
        16.0 + if text.is_empty() { 0.0 } else { 7.0 }
    } else {
        0.0
    };
    let pad_x = if text.is_empty() { 8.0 } else { 14.0 };
    let size = vec2(galley.size().x + icon_w + pad_x * 2.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let enabled = ui.is_enabled();
    let hot = enabled && resp.hovered();
    let pressed = enabled && resp.is_pointer_button_down_on();

    let (fill, stroke, ink) = match kind {
        Btn::Primary => {
            let f = if pressed {
                mix(pal.accent, Color32::BLACK, 0.15)
            } else if hot {
                mix(pal.accent, Color32::WHITE, 0.16)
            } else {
                pal.accent
            };
            (f, Stroke::NONE, pal.on_accent)
        }
        Btn::Secondary => {
            let f = if hot {
                mix(pal.raised, pal.accent, 0.14)
            } else {
                pal.raised
            };
            (
                f,
                Stroke::new(
                    1.0,
                    if hot {
                        mix(pal.border, pal.accent, 0.6)
                    } else {
                        pal.border
                    },
                ),
                pal.text,
            )
        }
        Btn::Ghost => (
            if hot {
                pal.raised
            } else {
                Color32::TRANSPARENT
            },
            Stroke::NONE,
            if hot { pal.text } else { pal.muted },
        ),
        Btn::Danger => {
            let f = if hot {
                with_alpha(pal.danger, 36)
            } else {
                Color32::TRANSPARENT
            };
            (
                f,
                Stroke::new(1.0, with_alpha(pal.danger, if hot { 200 } else { 110 })),
                pal.danger,
            )
        }
    };
    let (fill, stroke, ink) = if enabled {
        (fill, stroke, ink)
    } else {
        disabled_style(&pal, kind)
    };
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::same(8),
        fill,
        stroke,
        StrokeKind::Inside,
    );
    let mut x = rect.min.x + pad_x;
    if let Some(i) = icon {
        icons::paint(
            p,
            icons::square(pos2(x + 8.0, rect.center().y), 16.0),
            i,
            ink,
        );
        x += icon_w;
    }
    if !text.is_empty() {
        p.galley_with_override_text_color(
            pos2(x, rect.center().y - galley.size().y / 2.0),
            galley,
            ink,
        );
    }
    resp
}

pub fn primary(ui: &mut Ui, icon: Option<Icon>, text: &str) -> Response {
    button(ui, Btn::Primary, icon, text)
}

pub fn secondary(ui: &mut Ui, icon: Option<Icon>, text: &str) -> Response {
    button(ui, Btn::Secondary, icon, text)
}

/// Square icon-only button without a frame.
pub fn icon_button(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let pal = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    let enabled = ui.is_enabled();
    let hot = enabled && resp.hovered();
    if hot {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(7), pal.raised);
    }
    let ink = if !enabled {
        with_alpha(pal.muted, 130)
    } else if hot {
        pal.text
    } else {
        pal.muted
    };
    icons::paint(ui.painter(), icons::square(rect.center(), 16.0), icon, ink);
    resp.on_hover_text(tip)
}

/// Rounded selectable chip.
pub fn pill(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let pal = palette(ui.ctx());
    let font = TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, Color32::WHITE);
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 26.0, 30.0), Sense::click());
    let hot = resp.hovered();
    let (fill, stroke, ink) = if selected {
        (
            pal.accent_soft,
            Stroke::new(1.0, pal.accent),
            pal.accent_text(),
        )
    } else if hot {
        (
            mix(pal.raised, pal.accent, 0.12),
            Stroke::new(1.0, mix(pal.border, pal.accent, 0.5)),
            pal.text,
        )
    } else {
        (pal.raised, Stroke::new(1.0, pal.border), pal.muted)
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(15),
        fill,
        stroke,
        StrokeKind::Inside,
    );
    ui.painter()
        .galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, ink);
    resp
}

pub struct Seg<T> {
    pub value: T,
    pub label: String,
    pub icon: Option<Icon>,
}

impl<T> Seg<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// Segmented control; returns the newly picked value, if any.
pub fn segmented<T: Copy + PartialEq>(ui: &mut Ui, current: T, items: &[Seg<T>]) -> Option<T> {
    let pal = palette(ui.ctx());
    let font = TextStyle::Button.resolve(ui.style());
    let galleys: Vec<Arc<Galley>> = items
        .iter()
        .map(|i| {
            ui.painter()
                .layout_no_wrap(i.label.clone(), font.clone(), Color32::WHITE)
        })
        .collect();
    let widths: Vec<f32> = items
        .iter()
        .zip(&galleys)
        .map(|(i, g)| g.size().x + 28.0 + if i.icon.is_some() { 23.0 } else { 0.0 })
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 8.0;
    let (rect, base) = ui.allocate_exact_size(vec2(total, 38.0), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(11),
        pal.input,
        Stroke::new(1.0, pal.border),
        StrokeKind::Inside,
    );
    let mut picked = None;
    let mut x = rect.min.x + 4.0;
    for (n, ((item, g), w)) in items.iter().zip(&galleys).zip(&widths).enumerate() {
        let r = Rect::from_min_size(pos2(x, rect.min.y + 4.0), vec2(*w, 30.0));
        let resp = ui.interact(r, base.id.with(n), Sense::click());
        let selected = item.value == current;
        let hot = resp.hovered();
        if selected {
            ui.painter()
                .rect_filled(r, CornerRadius::same(8), pal.accent);
        } else if hot {
            ui.painter()
                .rect_filled(r, CornerRadius::same(8), pal.raised);
        }
        let ink = if selected {
            pal.on_accent
        } else if hot {
            pal.text
        } else {
            pal.muted
        };
        let content_w = g.size().x + if item.icon.is_some() { 23.0 } else { 0.0 };
        let mut cx = r.center().x - content_w / 2.0;
        if let Some(ic) = item.icon {
            icons::paint(
                ui.painter(),
                icons::square(pos2(cx + 8.0, r.center().y), 16.0),
                ic,
                ink,
            );
            cx += 23.0;
        }
        ui.painter().galley_with_override_text_color(
            pos2(cx, r.center().y - g.size().y / 2.0),
            g.clone(),
            ink,
        );
        if resp.clicked() && !selected {
            picked = Some(item.value);
        }
        x += *w;
    }
    picked
}

/// Switch; returns true when toggled this frame.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> bool {
    let pal = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(40.0, 22.0), Sense::click());
    let mut changed = false;
    if resp.clicked() {
        *on = !*on;
        changed = true;
    }
    let t = ui.ctx().animate_bool_with_time(resp.id, *on, 0.12);
    let track = mix(mix(pal.raised, pal.muted, 0.35), pal.accent, t);
    ui.painter()
        .rect_filled(rect, CornerRadius::same(11), track);
    let knob_x = egui::lerp((rect.min.x + 11.0)..=(rect.max.x - 11.0), t);
    ui.painter().circle_filled(
        pos2(knob_x, rect.center().y),
        8.0,
        if *on { pal.on_accent } else { Color32::WHITE },
    );
    changed
}

pub fn badge(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let pal = palette(ui.ctx());
    let font = FontId::proportional(11.0);
    let ink = if pal.dark {
        color
    } else {
        mix(color, Color32::BLACK, 0.45)
    };
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, ink);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + vec2(14.0, 6.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(10),
        with_alpha(color, if pal.dark { 38 } else { 56 }),
    );
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, ink);
    resp
}

/// Colour swatch; a ring marks the selected one.
pub fn swatch(ui: &mut Ui, color: Color32, size: f32, selected: bool) -> Response {
    let pal = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let inner = rect.shrink(if selected || resp.hovered() { 3.0 } else { 0.0 });
    ui.painter().rect_filled(
        inner,
        CornerRadius::same(((size / 3.0) as u8).max(4)),
        color,
    );
    ui.painter().rect_stroke(
        inner,
        CornerRadius::same(((size / 3.0) as u8).max(4)),
        Stroke::new(1.0, with_alpha(pal.text, 40)),
        StrokeKind::Inside,
    );
    if selected {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(((size / 3.0) as u8).max(4) + 2),
            Stroke::new(2.0, pal.text),
            StrokeKind::Inside,
        );
    }
    resp
}

/// Battery readout: glyph, percentage and a bolt while charging.
pub fn battery_pill(ui: &mut Ui, b: &Battery) -> Response {
    let pal = palette(ui.ctx());
    let pct = b.percent;
    let color = match pct {
        Some(p) if p <= 15 => pal.danger,
        Some(p) if p <= 35 => pal.warning,
        _ => pal.success,
    };
    let text = pct.map_or("--".to_string(), |p| format!("{p}%"));
    let galley = ui
        .painter()
        .layout_no_wrap(text, FontId::proportional(12.5), pal.text);
    let bolt = if b.charging { 12.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(
        vec2(24.0 + galley.size().x + bolt + 16.0, 24.0),
        Sense::hover(),
    );
    ui.painter().rect(
        rect,
        CornerRadius::same(12),
        pal.raised,
        Stroke::new(1.0, pal.border),
        StrokeKind::Inside,
    );
    let body = Rect::from_min_size(
        pos2(rect.min.x + 9.0, rect.center().y - 4.5),
        vec2(15.0, 9.0),
    );
    ui.painter().rect_stroke(
        body,
        CornerRadius::same(2),
        Stroke::new(1.0, pal.muted),
        StrokeKind::Inside,
    );
    ui.painter().rect_filled(
        Rect::from_min_size(pos2(body.max.x + 0.5, body.min.y + 2.5), vec2(1.8, 4.0)),
        CornerRadius::same(1),
        pal.muted,
    );
    let fill_w = (body.width() - 3.0) * f32::from(pct.unwrap_or(0).min(100)) / 100.0;
    ui.painter().rect_filled(
        Rect::from_min_size(
            body.min + vec2(1.5, 1.5),
            vec2(fill_w.max(0.0), body.height() - 3.0),
        ),
        CornerRadius::same(1),
        color,
    );
    let mut x = body.max.x + 7.0;
    if b.charging {
        icons::paint(
            ui.painter(),
            icons::square(pos2(x + 4.0, rect.center().y), 12.0),
            Icon::Bolt,
            pal.warning,
        );
        x += bolt;
    }
    ui.painter().galley(
        pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        pal.text,
    );
    resp
}

/// Paint text anchored inside `rect`, clipped to it.
pub fn text_in(
    p: &egui::Painter,
    rect: Rect,
    anchor: Align2,
    text: &str,
    font: FontId,
    color: Color32,
) {
    let p = p.with_clip_rect(rect);
    let pos: Pos2 = match anchor {
        Align2::LEFT_CENTER => pos2(rect.min.x, rect.center().y),
        Align2::RIGHT_CENTER => pos2(rect.max.x, rect.center().y),
        _ => rect.center(),
    };
    p.text(pos, anchor, text, font, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gearwise_core::profile::Theme;

    /// WCAG relative luminance contrast of two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lin = |c: u8| {
            let c = f32::from(c) / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let l = |c: Color32| 0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b());
        let (hi, lo) = (l(a).max(l(b)), l(a).min(l(b)));
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn disabled_buttons_stay_readable_and_visible_in_both_themes() {
        for theme in [Theme::Dark, Theme::Light] {
            let pal = Palette::new(theme, Color32::from_rgb(0, 212, 170));
            for kind in [Btn::Primary, Btn::Secondary] {
                let (fill, stroke, ink) = disabled_style(&pal, kind);
                assert!(contrast(ink, fill) >= 3.0, "{theme:?} text on its plate");
                assert!(
                    contrast(fill, pal.card) > 1.03 || contrast(stroke.color, pal.card) > 1.1,
                    "{theme:?} the plate or its border shows against a card"
                );
            }
            let (_, _, ghost_ink) = disabled_style(&pal, Btn::Ghost);
            assert!(contrast(ghost_ink, pal.card) >= 2.5, "{theme:?} ghost text");
        }
    }

    #[test]
    fn titles_are_cut_with_an_ellipsis_to_their_width() {
        let ctx = egui::Context::default();
        crate::theme::apply(
            &ctx,
            &Palette::new(Theme::Dark, Color32::from_rgb(0, 212, 170)),
        );
        let mut widths = (0.0, 0.0);
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let long = "Logitech G502 HERO Gaming Mouse Wireless Pro Max Edition";
            widths = (
                title_fit(ui, long, 200.0).rect.width(),
                title_fit(ui, "G502", 200.0).rect.width(),
            );
        });
        out.textures_delta.clear();
        assert!(widths.0 <= 202.0, "{widths:?}");
        assert!(widths.1 < 120.0, "{widths:?}");
    }
}
