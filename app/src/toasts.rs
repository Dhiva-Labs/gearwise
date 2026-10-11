//! Transient notifications, bottom-right.

use std::time::{Duration, Instant};

use egui::{
    Align2, Area, Context, CornerRadius, Frame, Id, Margin, Order, RichText, Sense, Stroke, vec2,
};

use crate::icons::{self, Icon};
use crate::state::ToastKind;
use crate::theme::{palette, with_alpha};

const LIFETIME: Duration = Duration::from_millis(3600);
const FADE: Duration = Duration::from_millis(500);
const MAX_VISIBLE: usize = 4;

struct Toast {
    kind: ToastKind,
    text: String,
    born: Instant,
}

#[derive(Default)]
pub struct Toasts {
    items: Vec<Toast>,
}

impl Toasts {
    pub fn push(&mut self, kind: ToastKind, text: impl Into<String>) {
        let text = text.into();
        // Collapse repeats of the message already on screen.
        self.items.retain(|t| t.text != text);
        self.items.push(Toast {
            kind,
            text,
            born: Instant::now(),
        });
        if self.items.len() > MAX_VISIBLE {
            self.items.remove(0);
        }
    }

    /// Drop every toast (the screenshot tour starts each view clean).
    pub fn clear(&mut self) {
        self.items.clear();
    }

    #[cfg(test)]
    pub fn texts(&self) -> Vec<(ToastKind, String)> {
        self.items
            .iter()
            .map(|t| (t.kind, t.text.clone()))
            .collect()
    }

    pub fn show(&mut self, ctx: &Context) {
        let now = Instant::now();
        self.items.retain(|t| now.duration_since(t.born) < LIFETIME);
        if self.items.is_empty() {
            return;
        }
        let pal = palette(ctx);
        let mut dismissed = None;
        let mut next_wake = LIFETIME;
        Area::new(Id::new("toasts"))
            .order(Order::Tooltip)
            .anchor(Align2::RIGHT_BOTTOM, vec2(-16.0, -16.0))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                for (i, t) in self.items.iter().enumerate() {
                    let age = now.duration_since(t.born);
                    let left = LIFETIME.saturating_sub(age);
                    let alpha = (left.as_secs_f32() / FADE.as_secs_f32()).min(1.0);
                    next_wake = next_wake.min(if left < FADE {
                        Duration::from_millis(33)
                    } else {
                        left - FADE
                    });
                    let (accent, icon) = match t.kind {
                        ToastKind::Info => (pal.accent, Icon::Info),
                        ToastKind::Success => (pal.success, Icon::Check),
                        ToastKind::Error => (pal.danger, Icon::Warning),
                    };
                    ui.scope(|ui| {
                        ui.set_opacity(alpha);
                        let r = Frame::new()
                            .fill(pal.raised)
                            .stroke(Stroke::new(1.0, with_alpha(accent, 140)))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::symmetric(12, 8))
                            .shadow(egui::Shadow {
                                offset: [0, 6],
                                blur: 18,
                                spread: 0,
                                color: egui::Color32::from_black_alpha(90),
                            })
                            .show(ui, |ui| {
                                ui.set_max_width(300.0);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    icons::show(ui, icon, 16.0, accent);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&t.text).size(13.0).color(pal.text),
                                        )
                                        .wrap_mode(egui::TextWrapMode::Wrap),
                                    );
                                });
                            })
                            .response;
                        if r.interact(Sense::click()).clicked() {
                            dismissed = Some(i);
                        }
                    });
                }
            });
        if let Some(i) = dismissed {
            self.items.remove(i);
        }
        ctx.request_repaint_after(next_wake);
    }
}
