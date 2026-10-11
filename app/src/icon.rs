//! The Gearwise mark, drawn procedurally (no image assets): a cog whose hollow is a
//! mouse, teal on a dark rounded tile. The geometry mirrors `assets/logo/gearwise.svg`
//! and is used for the window icon, the sidebar wordmark and `--export-icons`.

use std::f32::consts::TAU;
use std::path::Path;
use std::sync::OnceLock;

use egui::Color32;
use gearwise_core::profile::Theme;

const TEETH: usize = 8;
/// Corner rounding applied around the base polygon.
const RHO: f32 = 0.030;
/// (radius, half width) of a tooth at its root and at its tip, before rounding.
const ROOT: (f32, f32) = (0.560 - RHO, 0.150 - RHO);
const TIP: (f32, f32) = (0.700 - RHO, 0.092 - RHO);
/// Half size and corner radius of the mouse-shaped hollow.
const HOLLOW: ((f32, f32), f32) = ((0.205, 0.315), 0.19);
/// Scroll wheel (centre x, centre y, half width, half height) and corner radius.
const WHEEL: ((f32, f32, f32, f32), f32) = ((0.0, -0.14, 0.03, 0.07), 0.03);
/// Button seam: centre y and half thickness.
const SEAM: (f32, f32) = (0.0459, 0.0107);
const TILE_RADIUS: f32 = 0.44;
/// Width of the faint rim around the tile.
const RIM: f32 = 0.012;

/// Sizes written by `--export-icons`.
pub const EXPORT_SIZES: [u32; 7] = [16, 32, 48, 64, 128, 256, 512];

const TEAL: [f32; 3] = [0.0, 212.0, 170.0];
const TEAL_DEEP: [f32; 3] = [0.0, 184.0, 148.0];

fn round_box(x: f32, y: f32, half: (f32, f32), radius: f32) -> f32 {
    let (qx, qy) = (x.abs() - (half.0 - radius), y.abs() - (half.1 - radius));
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - radius
}

/// The base polygon of the cog (the outline is this polygon grown by [`RHO`]).
fn gear_polygon() -> Vec<(f32, f32)> {
    let mut pts = Vec::with_capacity(TEETH * 4);
    for k in 0..TEETH {
        let a = k as f32 * TAU / TEETH as f32 - TAU / 4.0;
        let (s, c) = a.sin_cos();
        for (r, hw) in [
            (ROOT.0, -ROOT.1),
            (TIP.0, -TIP.1),
            (TIP.0, TIP.1),
            (ROOT.0, ROOT.1),
        ] {
            pts.push((r * c - hw * s, r * s + hw * c));
        }
    }
    pts
}

/// Distance from `p` to the boundary of `poly`, negative inside.
fn signed_distance(poly: &[(f32, f32)], p: (f32, f32)) -> f32 {
    let mut best = f32::MAX;
    let mut inside = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (ab, ap) = ((b.0 - a.0, b.1 - a.1), (p.0 - a.0, p.1 - a.1));
        let t = ((ap.0 * ab.0 + ap.1 * ab.1) / (ab.0 * ab.0 + ab.1 * ab.1)).clamp(0.0, 1.0);
        let d = ((ap.0 - ab.0 * t).powi(2) + (ap.1 - ab.1 * t).powi(2)).sqrt();
        best = best.min(d);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * ab.0 {
            inside = !inside;
        }
    }
    if inside { -best } else { best }
}

/// What a point of the mark is made of.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    /// Cog, wheel or seam.
    Ink,
    Hollow,
    Outside,
}

struct Mark {
    poly: Vec<(f32, f32)>,
    detail: bool,
}

impl Mark {
    fn new(detail: bool) -> Self {
        Self {
            poly: gear_polygon(),
            detail,
        }
    }

    fn at(&self, x: f32, y: f32) -> Part {
        let r2 = x * x + y * y;
        if round_box(x, y, HOLLOW.0, HOLLOW.1) < 0.0 {
            if self.detail {
                let ((cx, cy, hw, hh), r) = WHEEL;
                if round_box(x - cx, y - cy, (hw, hh), r) < 0.0 || (y - SEAM.0).abs() < SEAM.1 {
                    return Part::Ink;
                }
            }
            return Part::Hollow;
        }
        let ring =
            r2 < 0.45 * 0.45 || (r2 < 0.76 * 0.76 && signed_distance(&self.poly, (x, y)) < RHO);
        if ring { Part::Ink } else { Part::Outside }
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Supersample `shade` (unit coordinates -> straight-alpha colour) over every pixel.
fn raster(size: u32, shade: impl Fn(f32, f32) -> Option<[f32; 4]>) -> Vec<u8> {
    let ss = if size >= 256 { 3 } else { 4 };
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for py in 0..size {
        for px in 0..size {
            let mut acc = [0.0f32; 4];
            for sy in 0..ss {
                for sx in 0..ss {
                    let u = (px as f32 + (sx as f32 + 0.5) / ss as f32) / size as f32 * 2.0 - 1.0;
                    let v = (py as f32 + (sy as f32 + 0.5) / ss as f32) / size as f32 * 2.0 - 1.0;
                    if let Some([r, g, b, a]) = shade(u, v) {
                        acc[0] += r * a;
                        acc[1] += g * a;
                        acc[2] += b * a;
                        acc[3] += a;
                    }
                }
            }
            let n = (ss * ss) as f32;
            let k = if acc[3] > 0.0 { 1.0 / acc[3] } else { 0.0 };
            out.extend_from_slice(&[
                (acc[0] * k).round() as u8,
                (acc[1] * k).round() as u8,
                (acc[2] * k).round() as u8,
                (acc[3] / n * 255.0).round() as u8,
            ]);
        }
    }
    out
}

/// The app icon: the mark on a rounded tile, as straight-alpha RGBA.
pub fn render_icon(size: u32, theme: Theme) -> Vec<u8> {
    let mark = Mark::new(size >= 40);
    let (top, bottom, ink, rim) = if theme == Theme::Dark {
        ([29.0, 36.0, 50.0], [11.0, 15.0, 22.0], TEAL, [255.0; 3])
    } else {
        (
            [248.0, 250.0, 253.0],
            [225.0, 231.0, 240.0],
            TEAL_DEEP,
            [26.0, 31.0, 43.0],
        )
    };
    raster(size, |x, y| {
        if round_box(x, y, (1.0, 1.0), TILE_RADIUS) >= 0.0 {
            return None;
        }
        let mut rgb = match mark.at(x, y) {
            Part::Ink => ink,
            _ => lerp3(top, bottom, (y + 1.0) / 2.0),
        };
        // A faint rim, as in the SVG, so the tile reads against any background.
        if round_box(x, y, (1.0, 1.0), TILE_RADIUS) > -RIM {
            rgb = lerp3(rgb, rim, 0.10);
        }
        Some([rgb[0], rgb[1], rgb[2], 1.0])
    })
}

/// The 128 px dark icon, rendered once and shared by the window icon and the sidebar.
fn icon_128() -> &'static [u8] {
    static ICON: OnceLock<Vec<u8>> = OnceLock::new();
    ICON.get_or_init(|| render_icon(128, Theme::Dark))
}

/// Window and taskbar icon.
pub fn window_icon() -> egui::IconData {
    egui::IconData {
        rgba: icon_128().to_vec(),
        width: 128,
        height: 128,
    }
}

/// The app icon as a texture (for the sidebar wordmark), created once per context.
pub fn tile_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::new("gearwise-icon-texture");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return t;
    }
    let image = egui::ColorImage::from_rgba_unmultiplied([128, 128], icon_128());
    let t = ctx.load_texture("gearwise-icon", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    t
}

/// Paint the icon texture into `rect`.
pub fn paint_tile(p: &egui::Painter, tex: &egui::TextureHandle, rect: egui::Rect) {
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    p.image(tex.id(), rect, uv, Color32::WHITE);
}

/// `--export-icons <dir>`: write the icon at every packaging size.
pub fn export(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for size in EXPORT_SIZES {
        let png = crate::png::encode_rgba(size, size, &render_icon(size, Theme::Dark));
        let path = dir.join(format!("gearwise-{size}.png"));
        std::fs::write(&path, png)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(img: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [img[i], img[i + 1], img[i + 2], img[i + 3]]
    }

    #[test]
    fn icon_is_a_rounded_tile_with_a_teal_cog_and_a_dark_hollow() {
        let icon = window_icon();
        assert_eq!((icon.width, icon.height), (128, 128));
        assert_eq!(icon.rgba.len(), 128 * 128 * 4);
        assert_eq!(px(&icon.rgba, 128, 0, 0)[3], 0, "corners are transparent");
        assert_eq!(px(&icon.rgba, 128, 64, 4)[3], 255, "the tile edge is solid");
        // The upper tooth is teal; the lower half of the hollow is the dark tile.
        let tooth = px(&icon.rgba, 128, 64, 22);
        assert!(tooth[1] > 190 && tooth[0] < 40, "{tooth:?}");
        let hollow = px(&icon.rgba, 128, 64, 78);
        assert!(hollow[1] < 60 && hollow[3] == 255, "{hollow:?}");
    }

    #[test]
    fn small_sizes_drop_the_fine_details() {
        let small = render_icon(32, Theme::Dark);
        assert_eq!(small.len(), 32 * 32 * 4);
        // The centre is hollow (no wheel or seam at this size).
        let c = px(&small, 32, 16, 16);
        assert!(c[1] < 60, "{c:?}");
    }

    #[test]
    fn light_variant_has_a_pale_tile() {
        let icon = render_icon(64, Theme::Light);
        let hollow = px(&icon, 64, 32, 40);
        assert!(hollow[0] > 200, "{hollow:?}");
        let tooth = px(&icon, 64, 32, 11);
        assert!(tooth[0] < 40 && tooth[1] > 150, "{tooth:?}");
    }

    #[test]
    fn every_export_size_renders_square_images() {
        for size in EXPORT_SIZES {
            assert_eq!(
                render_icon(size, Theme::Dark).len(),
                (size * size * 4) as usize
            );
        }
    }
}
