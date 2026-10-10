//! The window/taskbar icon, rasterised at startup (no image assets).

const SIZE: u32 = 64;

/// Signed distance from `(x, y)` to a rounded box centred at the origin.
fn round_box(x: f32, y: f32, half: (f32, f32), radius: f32) -> f32 {
    let (qx, qy) = (x.abs() - (half.0 - radius), y.abs() - (half.1 - radius));
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - radius
}

/// Accent-coloured rounded square with a mouse outline.
pub fn window_icon() -> egui::IconData {
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for py in 0..SIZE {
        for px in 0..SIZE {
            let (x, y) = (px as f32 + 0.5 - 32.0, py as f32 + 0.5 - 32.0);
            let tile = (0.5 - round_box(x, y, (30.0, 30.0), 14.0)).clamp(0.0, 1.0);
            let body = (1.7 - round_box(x, y, (11.0, 18.0), 11.0).abs()).clamp(0.0, 1.0);
            let wheel = (1.5 - (x.abs())).clamp(0.0, 1.0) * ((y + 18.0).clamp(0.0, 1.0)) * ((-y - 1.0).clamp(0.0, 1.0));
            let ink = body.max(wheel);
            let mix = |a: f32, b: f32| (a + (b - a) * ink) as u8;
            rgba.extend_from_slice(&[mix(0.0, 11.0), mix(212.0, 16.0), mix(170.0, 20.0), (tile * 255.0) as u8]);
        }
    }
    egui::IconData { rgba, width: SIZE, height: SIZE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_square_rounded_and_not_blank() {
        let icon = window_icon();
        assert_eq!((icon.width, icon.height), (64, 64));
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);
        let alpha = |x: usize, y: usize| icon.rgba[(y * 64 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "corners are transparent");
        assert_eq!(alpha(32, 32), 255);
        assert!(icon.rgba.chunks(4).any(|p| p[3] == 255 && p[0] < 40), "outline ink is present");
    }
}
