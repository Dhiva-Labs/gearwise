//! Palette, text styles and the egui visuals that make the app look like itself.

use egui::{Color32, Context, CornerRadius, FontFamily, FontId, Id, Stroke, TextStyle, Vec2};
use gearwise_core::model::Vendor;
use gearwise_core::profile::Theme;

pub const ACCENTS: [(&str, &str); 6] = [
    ("Teal", "#00d4aa"),
    ("Violet", "#8b5cf6"),
    ("Green", "#44d62c"),
    ("Amber", "#ffb020"),
    ("Sky", "#00b8fc"),
    ("Coral", "#ff4d5e"),
];

/// Colours of the DPI stages, in order.
pub const STAGE_COLORS: [Color32; 6] = [
    Color32::from_rgb(0x00, 0xb8, 0xfc),
    Color32::from_rgb(0x44, 0xd6, 0x2c),
    Color32::from_rgb(0xff, 0xb0, 0x20),
    Color32::from_rgb(0xff, 0x4d, 0x5e),
    Color32::from_rgb(0x8b, 0x5c, 0xf6),
    Color32::from_rgb(0xe8, 0x5d, 0xd0),
];

pub fn stage_color(i: usize) -> Color32 {
    STAGE_COLORS[i % STAGE_COLORS.len()]
}

pub fn vendor_color(v: Vendor) -> Color32 {
    match v {
        Vendor::Logitech => Color32::from_rgb(0x00, 0xb8, 0xfc),
        Vendor::Razer => Color32::from_rgb(0x44, 0xd6, 0x2c),
        Vendor::Corsair => Color32::from_rgb(0xff, 0xcc, 0x00),
        Vendor::SteelSeries => Color32::from_rgb(0xff, 0x6a, 0x1a),
        Vendor::Other => Color32::from_rgb(0x8a, 0x93, 0xa6),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub window: Color32,
    pub sidebar: Color32,
    pub card: Color32,
    pub raised: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    /// Accent at low alpha, for hover and selected backgrounds.
    pub accent_soft: Color32,
    /// Readable ink on top of a solid accent fill.
    pub on_accent: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,
    /// Field background, slightly darker (or lighter) than a card.
    pub input: Color32,
}

impl Palette {
    pub fn new(theme: Theme, accent: Color32) -> Self {
        let dark = theme == Theme::Dark;
        let c = |r, g, b| Color32::from_rgb(r, g, b);
        let (window, sidebar, card, raised, border, text, muted, input) = if dark {
            (
                c(0x0e, 0x10, 0x14),
                c(0x12, 0x15, 0x1b),
                c(0x18, 0x1c, 0x24),
                c(0x1f, 0x24, 0x30),
                c(0x27, 0x2d, 0x3a),
                c(0xe7, 0xea, 0xf0),
                c(0x8a, 0x93, 0xa6),
                c(0x11, 0x14, 0x1a),
            )
        } else {
            (
                c(0xf2, 0xf4, 0xf8),
                c(0xe8, 0xec, 0xf2),
                c(0xff, 0xff, 0xff),
                c(0xee, 0xf1, 0xf6),
                c(0xd9, 0xdf, 0xe8),
                c(0x1a, 0x1f, 0x2b),
                c(0x5f, 0x69, 0x7d),
                c(0xf6, 0xf8, 0xfb),
            )
        };
        let luma = 0.299 * f32::from(accent.r())
            + 0.587 * f32::from(accent.g())
            + 0.114 * f32::from(accent.b());
        Self {
            dark,
            window,
            sidebar,
            card,
            raised,
            border,
            text,
            muted,
            accent,
            accent_soft: with_alpha(accent, if dark { 38 } else { 44 }),
            on_accent: if luma > 135.0 {
                c(0x0b, 0x10, 0x14)
            } else {
                Color32::WHITE
            },
            danger: if dark {
                c(0xff, 0x5c, 0x6c)
            } else {
                c(0xd9, 0x2d, 0x42)
            },
            success: if dark {
                c(0x3d, 0xdc, 0x84)
            } else {
                c(0x14, 0x9c, 0x5a)
            },
            warning: if dark {
                c(0xff, 0xb0, 0x20)
            } else {
                c(0xc7, 0x7d, 0x00)
            },
            input,
        }
    }

    /// Text colour for accent-coloured labels: the accent itself on dark, a
    /// darker mix on light so it stays readable.
    pub fn accent_text(&self) -> Color32 {
        if self.dark {
            self.accent
        } else {
            mix(self.accent, Color32::BLACK, 0.35)
        }
    }
}

pub fn parse_hex(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some(Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

pub fn to_hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

pub fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Linear blend, alpha included (colours are premultiplied, so this is a proper cross-fade).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

// --- Text styles -----------------------------------------------------------

pub fn title() -> TextStyle {
    TextStyle::Name("Title".into())
}
pub fn subtitle() -> TextStyle {
    TextStyle::Name("Subtitle".into())
}
pub fn section() -> TextStyle {
    TextStyle::Name("Section".into())
}
pub fn caption() -> TextStyle {
    TextStyle::Name("Caption".into())
}
pub fn big_number() -> TextStyle {
    TextStyle::Name("BigNumber".into())
}

/// The palette currently applied to `ctx`.
pub fn palette(ctx: &Context) -> Palette {
    ctx.data(|d| d.get_temp::<Palette>(Id::NULL))
        .unwrap_or_else(|| Palette::new(Theme::Dark, Color32::from_rgb(0, 0xd4, 0xaa)))
}

/// Install `pal` as the active look.
pub fn apply(ctx: &Context, pal: &Palette) {
    ctx.data_mut(|d| d.insert_temp(Id::NULL, *pal));
    ctx.set_theme(if pal.dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });

    let mut style = (*ctx.global_style()).clone();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(22.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
        (title(), FontId::new(28.0, FontFamily::Proportional)),
        (subtitle(), FontId::new(17.0, FontFamily::Proportional)),
        (section(), FontId::new(11.5, FontFamily::Proportional)),
        (caption(), FontId::new(12.0, FontFamily::Proportional)),
        (big_number(), FontId::new(20.0, FontFamily::Proportional)),
    ]
    .into();

    let sp = &mut style.spacing;
    sp.item_spacing = Vec2::new(10.0, 10.0);
    sp.button_padding = Vec2::new(14.0, 7.0);
    sp.interact_size = Vec2::new(36.0, 30.0);
    sp.slider_width = 220.0;
    sp.slider_rail_height = 6.0;
    sp.combo_width = 160.0;
    sp.text_edit_width = 200.0;
    // A thin bar that stays visible (so long pages read as scrollable), plus a fading edge.
    sp.scroll = egui::style::ScrollStyle {
        floating: true,
        bar_width: 9.0,
        floating_width: 4.0,
        floating_allocated_width: 8.0,
        foreground_color: true,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.0,
        interact_background_opacity: 0.1,
        dormant_handle_opacity: 0.32,
        active_handle_opacity: 0.55,
        interact_handle_opacity: 0.9,
        fade: egui::style::ScrollFadeStyle {
            strength: 0.95,
            size: 26.0,
        },
        ..egui::style::ScrollStyle::floating()
    };
    sp.menu_margin = egui::Margin::same(6);
    sp.window_margin = egui::Margin::same(16);
    style.animation_time = 0.12;
    style.interaction.tooltip_delay = 0.35;

    let v = &mut style.visuals;
    v.dark_mode = pal.dark;
    v.override_text_color = None;
    v.weak_text_color = Some(pal.muted);
    v.panel_fill = pal.window;
    v.window_fill = pal.card;
    v.window_stroke = Stroke::new(1.0, pal.border);
    v.window_corner_radius = CornerRadius::same(14);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_shadow = egui::Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(if pal.dark { 120 } else { 50 }),
    };
    v.popup_shadow = v.window_shadow;
    v.extreme_bg_color = pal.input;
    v.faint_bg_color = pal.raised;
    v.code_bg_color = pal.raised;
    v.hyperlink_color = pal.accent_text();
    v.warn_fg_color = pal.warning;
    v.error_fg_color = pal.danger;
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Circle;
    v.selection.bg_fill = pal.accent;
    v.selection.stroke = Stroke::new(1.0, pal.on_accent);
    v.interact_cursor = Some(egui::CursorIcon::PointingHand);

    let r = CornerRadius::same(8);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = pal.card;
    w.noninteractive.weak_bg_fill = pal.card;
    w.noninteractive.bg_stroke = Stroke::new(1.0, pal.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, pal.text);
    w.noninteractive.corner_radius = r;

    let hover_fill = mix(pal.raised, pal.accent, 0.12);
    w.inactive.bg_fill = pal.raised;
    w.inactive.weak_bg_fill = pal.raised;
    w.inactive.bg_stroke = Stroke::new(1.0, pal.border);
    w.inactive.fg_stroke = Stroke::new(1.0, pal.text);
    w.inactive.corner_radius = r;

    w.hovered.bg_fill = hover_fill;
    w.hovered.weak_bg_fill = hover_fill;
    w.hovered.bg_stroke = Stroke::new(1.0, mix(pal.border, pal.accent, 0.55));
    w.hovered.fg_stroke = Stroke::new(1.0, pal.text);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 0.0;

    w.active.bg_fill = pal.accent_soft;
    w.active.weak_bg_fill = pal.accent_soft;
    w.active.bg_stroke = Stroke::new(1.0, pal.accent);
    w.active.fg_stroke = Stroke::new(1.0, pal.text);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;

    w.open.bg_fill = pal.raised;
    w.open.weak_bg_fill = pal.raised;
    w.open.bg_stroke = Stroke::new(1.0, pal.border);
    w.open.fg_stroke = Stroke::new(1.0, pal.text);
    w.open.corner_radius = r;

    ctx.set_global_style(style);
}

pub fn c32(c: gearwise_core::model::Rgb) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

pub fn rgb(c: Color32) -> gearwise_core::model::Rgb {
    gearwise_core::model::Rgb::new(c.r(), c.g(), c.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_blends_alpha_too() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
        assert_eq!(mix(a, b, 0.5), Color32::from_rgb(100, 50, 25));
        let clear = Color32::TRANSPARENT;
        assert_eq!(mix(clear, Color32::WHITE, 0.5).a(), 128);
    }

    #[test]
    fn hex_round_trips() {
        assert_eq!(parse_hex("#00d4aa"), Some(Color32::from_rgb(0, 0xd4, 0xaa)));
        assert_eq!(to_hex(Color32::from_rgb(1, 2, 3)), "#010203");
        assert_eq!(parse_hex("nope"), None);
    }

    #[test]
    fn light_and_dark_palettes_differ_and_pick_readable_ink() {
        let teal = Color32::from_rgb(0, 0xd4, 0xaa);
        let (d, l) = (
            Palette::new(Theme::Dark, teal),
            Palette::new(Theme::Light, teal),
        );
        assert!(d.window.r() < 40 && l.window.r() > 200);
        assert_eq!(
            d.on_accent,
            Color32::from_rgb(0x0b, 0x10, 0x14),
            "bright accents get dark ink"
        );
        assert_eq!(
            Palette::new(Theme::Dark, Color32::from_rgb(40, 20, 120)).on_accent,
            Color32::WHITE
        );
    }
}
