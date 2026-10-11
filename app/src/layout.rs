//! Layout building blocks: cards, side-by-side card columns, settings rows, key/value rows,
//! dividers and the scrolling page body.

use egui::{
    Align, CornerRadius, Frame, InnerResponse, Layout, Margin, RichText, ScrollArea, Sense, Stroke,
    Ui, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{Palette, palette};
use crate::widgets::{caption, muted};

pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    card_with(ui, 16, add)
}

pub fn card_with<R>(ui: &mut Ui, pad: i8, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let pal = palette(ui.ctx());
    Frame::new()
        .fill(pal.card)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(pad))
        .show(ui, add)
}

/// Width a toggle switch takes in a settings row.
pub const TOGGLE_W: f32 = 40.0;

/// A settings row: title and description on the left, a toggle on the right. The text wraps
/// in the space left of the control instead of running underneath it.
pub fn setting_row<R>(
    ui: &mut Ui,
    title: &str,
    desc: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    setting_row_w(ui, title, desc, TOGGLE_W, control)
}

/// Like [`setting_row`] for a control that is `control_w` wide.
pub fn setting_row_w<R>(
    ui: &mut Ui,
    title: &str,
    desc: &str,
    control_w: f32,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    let pal = palette(ui.ctx());
    let gap = ui.spacing().item_spacing.x.max(14.0);
    let text_w = (ui.available_width() - control_w - gap).max(120.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let text =
            ui.allocate_ui_with_layout(vec2(text_w, 10.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(text_w);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new(title).color(pal.text));
                caption(ui, desc);
            });
        let h = text.response.rect.height().max(24.0);
        ui.allocate_ui_with_layout(
            vec2(control_w, h),
            Layout::right_to_left(Align::Center),
            control,
        )
        .inner
    })
    .inner
}

/// Height of the visible part of the page body (inside [`page_scroll`], the scroll viewport).
pub fn viewport_height(ui: &Ui) -> f32 {
    ui.clip_rect().height()
}

/// A page body that scrolls inside the content area (thin bar, fading edge) rather than
/// being cut off at the window's bottom edge.
pub fn page_scroll(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
            ui.add_space(24.0);
        });
}

/// Label/value pair for info pages; a long value wraps inside its column.
pub fn kv(ui: &mut Ui, key: &str, value: &str) {
    let pal = palette(ui.ctx());
    let key_w = (ui.available_width() * 0.38).clamp(96.0, 150.0);
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            vec2(key_w, 20.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.label(RichText::new(key).color(pal.muted));
            },
        );
        let w = ui.available_width().max(60.0);
        ui.allocate_ui_with_layout(vec2(w, 20.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(w);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(RichText::new(value).color(pal.text));
        });
    });
}

/// Hairline divider that respects the theme.
pub fn divider(ui: &mut Ui) {
    let pal = palette(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, pal.border),
    );
}

/// Centered placeholder text with an icon, for empty lists.
pub fn empty_hint(ui: &mut Ui, icon: Icon, text: &str) {
    let pal: Palette = palette(ui.ctx());
    ui.vertical_centered(|ui| {
        ui.add_space(18.0);
        icons::show(ui, icon, 28.0, pal.muted);
        ui.add_space(6.0);
        muted(ui, text);
        ui.add_space(18.0);
    });
}

/// Two cards side by side, `left_frac` of the width for the first. Both are
/// as tall as the taller one (measured on the previous frame).
pub fn card_columns(
    ui: &mut Ui,
    left_frac: f32,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    const GAP: f32 = 16.0;
    const FRAME: f32 = 34.0;
    let total = ui.available_width();
    let lw = ((total - GAP) * left_frac).floor();
    let rw = total - GAP - lw;
    let key = ui.id().with("card-columns-height");
    let want: f32 = ui.ctx().data(|d| d.get_temp(key)).unwrap_or(0.0);
    let mut natural = [0.0f32; 2];
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        ui.allocate_ui_with_layout(
            vec2(lw, 10.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                card(ui, |ui| {
                    ui.set_width(lw - FRAME);
                    ui.set_min_height(want);
                    let y0 = ui.cursor().min.y;
                    left(ui);
                    natural[0] = ui.cursor().min.y - y0;
                });
            },
        );
        ui.allocate_ui_with_layout(
            vec2(rw, 10.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                card(ui, |ui| {
                    ui.set_width(rw - FRAME);
                    ui.set_min_height(want);
                    let y0 = ui.cursor().min.y;
                    right(ui);
                    natural[1] = ui.cursor().min.y - y0;
                });
            },
        );
    });
    let tallest = natural[0].max(natural[1]);
    if (tallest - want).abs() > 0.5 {
        ui.ctx().data_mut(|d| d.insert_temp(key, tallest));
        ui.ctx().request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, RawInput, Rect};

    #[test]
    fn setting_rows_wrap_their_text_beside_the_control() {
        let ctx = egui::Context::default();
        crate::theme::apply(
            &ctx,
            &crate::theme::Palette::new(
                gearwise_core::profile::Theme::Dark,
                egui::Color32::from_rgb(0, 212, 170),
            ),
        );
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(500.0, 400.0));
        let mut control = Rect::NOTHING;
        let mut row = Rect::NOTHING;
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                ui.set_width(460.0);
                let desc = "Follows the focused program. Focus detection needs X11 or Windows; on Wayland switch by hand and keep going so that this wraps.";
                setting_row(ui, "Switch profiles automatically", desc, |ui| {
                    control = ui.max_rect();
                });
                row = ui.min_rect();
            },
        );
        out.textures_delta.clear();
        assert_eq!(control.width(), TOGGLE_W, "the control keeps its width");
        assert!(row.max.x <= 461.0, "nothing runs past the row: {row:?}");
        // The description wrapped to several lines instead of running under the control.
        assert!(row.height() > 42.0, "{row:?}");
        assert!(control.center().y > row.min.y && control.center().y < row.max.y);
    }
}
