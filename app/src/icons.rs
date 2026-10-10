//! Small vector icons drawn with the painter, so no font or image assets are needed.

use egui::epaint::PathStroke;
use egui::{
    Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use periph_core::model::DeviceKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Mouse,
    Keyboard,
    Headset,
    Mousepad,
    Receiver,
    Device,
    Bolt,
    Plus,
    Trash,
    Copy,
    Check,
    Close,
    ArrowUp,
    ArrowDown,
    Gear,
    User,
    Play,
    Record,
    Stop,
    Save,
    Sun,
    Moon,
    Gauge,
    Cursor,
    Info,
    Layers,
    Download,
    Upload,
    Warning,
    Macro,
    Eyedropper,
    Edit,
}

impl Icon {
    pub fn of(kind: DeviceKind) -> Self {
        match kind {
            DeviceKind::Mouse => Icon::Mouse,
            DeviceKind::Keyboard => Icon::Keyboard,
            DeviceKind::Headset => Icon::Headset,
            DeviceKind::Mousepad => Icon::Mousepad,
            DeviceKind::Receiver => Icon::Receiver,
            DeviceKind::Other => Icon::Device,
        }
    }
}

/// Paint `icon` centred in `rect` (a square is expected).
pub fn paint(p: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let s = rect.width().min(rect.height());
    let c = rect.center();
    let w = (s / 12.0).clamp(1.2, 2.2);
    let st = Stroke::new(w, color);
    // Point in unit space (-0.5..0.5) mapped onto the rect.
    let pt = |x: f32, y: f32| pos2(c.x + x * s, c.y + y * s);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], st);
    };
    let poly = |pts: &[(f32, f32)]| {
        p.add(Shape::line(
            pts.iter().map(|q| pt(q.0, q.1)).collect(),
            PathStroke::from(st),
        ));
    };
    let arc = |cx: f32, cy: f32, r: f32, from: f32, to: f32, n: usize| -> Vec<Pos2> {
        (0..=n)
            .map(|i| {
                let a = (from + (to - from) * i as f32 / n as f32).to_radians();
                pt(cx + r * a.cos(), cy + r * a.sin())
            })
            .collect()
    };
    let rrect = |x0: f32, y0: f32, x1: f32, y1: f32, r: f32| {
        p.rect_stroke(
            Rect::from_min_max(pt(x0, y0), pt(x1, y1)),
            CornerRadius::same((r * s) as u8),
            st,
            StrokeKind::Middle,
        );
    };

    match icon {
        Icon::Mouse => {
            rrect(-0.24, -0.42, 0.24, 0.42, 0.24);
            line((0.0, -0.42), (0.0, -0.1));
            line((-0.24, -0.1), (0.24, -0.1));
        }
        Icon::Keyboard => {
            rrect(-0.46, -0.28, 0.46, 0.28, 0.1);
            for x in [-0.28, -0.1, 0.08, 0.26] {
                p.circle_filled(pt(x, -0.1), w * 0.55, color);
            }
            for x in [-0.2, 0.0, 0.2] {
                p.circle_filled(pt(x, 0.04), w * 0.55, color);
            }
            line((-0.2, 0.17), (0.2, 0.17));
        }
        Icon::Headset => {
            p.add(Shape::line(
                arc(0.0, 0.04, 0.34, 180.0, 360.0, 16),
                PathStroke::from(st),
            ));
            rrect(-0.42, 0.0, -0.24, 0.3, 0.07);
            rrect(0.24, 0.0, 0.42, 0.3, 0.07);
        }
        Icon::Mousepad => {
            rrect(-0.46, -0.3, 0.46, 0.3, 0.1);
            rrect(-0.1, -0.17, 0.1, 0.17, 0.1);
        }
        Icon::Receiver => {
            rrect(-0.2, -0.06, 0.2, 0.42, 0.06);
            rrect(-0.12, -0.42, 0.12, -0.06, 0.03);
            line((-0.06, -0.3), (-0.06, -0.2));
            line((0.06, -0.3), (0.06, -0.2));
        }
        Icon::Device => {
            rrect(-0.34, -0.34, 0.34, 0.34, 0.12);
            p.circle_filled(pt(0.0, 0.0), w, color);
        }
        Icon::Bolt => {
            p.add(Shape::convex_polygon(
                vec![
                    pt(0.1, -0.46),
                    pt(-0.26, 0.06),
                    pt(-0.02, 0.06),
                    pt(-0.1, 0.46),
                    pt(0.26, -0.1),
                    pt(0.02, -0.1),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Plus => {
            line((-0.3, 0.0), (0.3, 0.0));
            line((0.0, -0.3), (0.0, 0.3));
        }
        Icon::Trash => {
            line((-0.36, -0.2), (0.36, -0.2));
            poly(&[(-0.12, -0.2), (-0.12, -0.34), (0.12, -0.34), (0.12, -0.2)]);
            poly(&[(-0.26, -0.2), (-0.22, 0.4), (0.22, 0.4), (0.26, -0.2)]);
            line((-0.08, -0.06), (-0.08, 0.28));
            line((0.08, -0.06), (0.08, 0.28));
        }
        Icon::Copy => {
            rrect(-0.1, -0.1, 0.36, 0.36, 0.07);
            poly(&[
                (0.1, -0.1),
                (0.1, -0.36),
                (-0.36, -0.36),
                (-0.36, 0.1),
                (-0.1, 0.1),
            ]);
        }
        Icon::Check => poly(&[(-0.32, 0.02), (-0.1, 0.24), (0.34, -0.22)]),
        Icon::Close => {
            line((-0.26, -0.26), (0.26, 0.26));
            line((0.26, -0.26), (-0.26, 0.26));
        }
        Icon::ArrowUp => poly(&[(-0.28, 0.12), (0.0, -0.16), (0.28, 0.12)]),
        Icon::ArrowDown => poly(&[(-0.28, -0.12), (0.0, 0.16), (0.28, -0.12)]),
        Icon::Gear => {
            p.circle_stroke(pt(0.0, 0.0), 0.2 * s, st);
            for i in 0..8 {
                let (sn, cs) = (i as f32 * std::f32::consts::FRAC_PI_4).sin_cos();
                line((0.3 * cs, 0.3 * sn), (0.42 * cs, 0.42 * sn));
            }
        }
        Icon::User => {
            p.circle_stroke(pt(0.0, -0.17), 0.14 * s, st);
            p.add(Shape::line(
                arc(0.0, 0.4, 0.3, 180.0, 360.0, 12),
                PathStroke::from(st),
            ));
        }
        Icon::Play => {
            p.add(Shape::convex_polygon(
                vec![pt(-0.2, -0.3), pt(0.3, 0.0), pt(-0.2, 0.3)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Record => {
            p.circle_filled(pt(0.0, 0.0), 0.26 * s, color);
        }
        Icon::Stop => {
            p.rect_filled(
                Rect::from_min_max(pt(-0.24, -0.24), pt(0.24, 0.24)),
                CornerRadius::same(2),
                color,
            );
        }
        Icon::Save | Icon::Download => {
            poly(&[(0.0, -0.4), (0.0, 0.12)]);
            poly(&[(-0.2, -0.08), (0.0, 0.14), (0.2, -0.08)]);
            if icon == Icon::Save {
                poly(&[(-0.34, 0.1), (-0.34, 0.38), (0.34, 0.38), (0.34, 0.1)]);
            } else {
                line((-0.34, 0.38), (0.34, 0.38));
            }
        }
        Icon::Upload => {
            poly(&[(0.0, 0.12), (0.0, -0.4)]);
            poly(&[(-0.2, -0.18), (0.0, -0.4), (0.2, -0.18)]);
            line((-0.34, 0.38), (0.34, 0.38));
        }
        Icon::Sun => {
            p.circle_stroke(pt(0.0, 0.0), 0.16 * s, st);
            for i in 0..8 {
                let (sn, cs) = (i as f32 * std::f32::consts::FRAC_PI_4).sin_cos();
                line((0.27 * cs, 0.27 * sn), (0.4 * cs, 0.4 * sn));
            }
        }
        Icon::Moon => {
            let mut pts = arc(-0.02, 0.0, 0.34, -50.0, 230.0, 24);
            pts.extend(arc(0.12, -0.1, 0.27, 220.0, -40.0, 24));
            p.add(Shape::closed_line(pts, PathStroke::from(st)));
        }
        Icon::Gauge => {
            p.add(Shape::line(
                arc(0.0, 0.1, 0.4, 153.0, 387.0, 20),
                PathStroke::from(st),
            ));
            line((0.0, 0.1), (0.2, -0.12));
            p.circle_filled(pt(0.0, 0.1), w * 0.8, color);
        }
        Icon::Cursor => {
            p.add(Shape::convex_polygon(
                vec![
                    pt(-0.22, -0.34),
                    pt(0.3, 0.06),
                    pt(0.04, 0.1),
                    pt(0.16, 0.36),
                    pt(0.04, 0.4),
                    pt(-0.08, 0.14),
                    pt(-0.22, 0.28),
                ],
                Color32::TRANSPARENT,
                st,
            ));
        }
        Icon::Info => {
            p.circle_stroke(pt(0.0, 0.0), 0.4 * s, st);
            line((0.0, -0.02), (0.0, 0.2));
            p.circle_filled(pt(0.0, -0.18), w * 0.7, color);
        }
        Icon::Layers => {
            poly(&[
                (-0.4, -0.08),
                (0.0, -0.3),
                (0.4, -0.08),
                (0.0, 0.14),
                (-0.4, -0.08),
            ]);
            poly(&[(-0.4, 0.1), (0.0, 0.32), (0.4, 0.1)]);
        }
        Icon::Warning => {
            poly(&[(0.0, -0.4), (0.4, 0.3), (-0.4, 0.3), (0.0, -0.4)]);
            line((0.0, -0.12), (0.0, 0.08));
            p.circle_filled(pt(0.0, 0.2), w * 0.7, color);
        }
        Icon::Macro => {
            rrect(-0.4, -0.3, 0.4, 0.3, 0.1);
            p.add(Shape::convex_polygon(
                vec![pt(-0.1, -0.14), pt(0.16, 0.0), pt(-0.1, 0.14)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Edit => {
            poly(&[
                (-0.34, 0.34),
                (-0.3, 0.12),
                (0.16, -0.34),
                (0.34, -0.16),
                (-0.12, 0.3),
                (-0.34, 0.34),
            ]);
            line((0.04, -0.22), (0.22, -0.04));
        }
        Icon::Eyedropper => {
            line((-0.34, 0.34), (0.06, -0.06));
            poly(&[(0.0, -0.12), (0.18, 0.06)]);
            poly(&[
                (0.0, -0.3),
                (0.3, 0.0),
                (0.4, -0.2),
                (0.2, -0.4),
                (0.0, -0.3),
            ]);
        }
    }
}

/// Allocate a square of `size` and paint an icon into it.
pub fn show(ui: &mut egui::Ui, icon: Icon, size: f32, color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    paint(ui.painter(), rect, icon, color);
    resp
}

/// Rect of `size` centred on `center`.
pub fn square(center: Pos2, size: f32) -> Rect {
    Rect::from_center_size(center, vec2(size, size))
}
