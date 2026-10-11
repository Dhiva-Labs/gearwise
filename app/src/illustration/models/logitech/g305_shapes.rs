//! Outline and part geometry of the G305 / G304 shell, measured off a top-down product shot
//! (389 x 700 px box) and redrawn as original vector art. Every constant is in those reference
//! pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (389.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose (the steps on the left are the side buttons).
pub const OUTLINE: &[P] = &[
    n(242.4, 30.0),
    n(263.2, 33.0),
    n(277.8, 38.0),
    n(333.1, 63.0),
    n(371.3, 83.0),
    n(384.9, 93.0),
    n(391.7, 102.0),
    n(394.3, 109.0),
    n(400.0, 143.0),
    n(410.0, 221.0),
    n(416.0, 291.0),
    n(419.0, 356.0),
    n(419.0, 417.0),
    n(418.0, 439.0),
    n(415.0, 477.0),
    n(411.0, 508.0),
    n(405.3, 540.0),
    n(400.3, 561.0),
    n(392.4, 587.0),
    n(380.9, 616.0),
    n(367.9, 641.0),
    n(352.5, 665.0),
    n(339.0, 682.0),
    n(323.0, 697.0),
    n(307.7, 708.0),
    n(286.2, 719.0),
    n(268.3, 725.0),
    n(245.3, 729.0),
    n(213.7, 729.0),
    n(194.7, 726.0),
    n(174.8, 720.0),
    n(157.8, 712.0),
    n(140.5, 701.0),
    n(125.1, 688.0),
    n(113.5, 675.0),
    n(101.5, 659.0),
    n(85.2, 633.0),
    n(70.5, 602.0),
    n(57.5, 565.0),
    n(47.2, 521.0),
    n(42.0, 487.0),
    n(39.9, 467.0),
    n(32.2, 455.0),
    n(30.0, 425.0),
    n(30.0, 379.0),
    n(31.3, 375.0),
    n(37.1, 367.0),
    n(36.9, 354.0),
    n(31.5, 346.0),
    n(30.9, 343.0),
    n(32.0, 311.0),
    n(35.1, 271.0),
    n(36.7, 266.0),
    n(43.0, 257.0),
    n(49.0, 202.0),
    n(54.0, 167.0),
    n(61.3, 123.0),
    n(64.5, 107.0),
    n(66.1, 103.0),
    n(70.0, 97.0),
    n(74.9, 92.0),
    n(85.7, 84.0),
    n(97.9, 77.0),
    n(137.9, 57.0),
    n(170.4, 42.0),
    n(185.7, 36.0),
    n(194.8, 33.0),
    n(214.7, 30.0),
];

/// Left primary button: a wide wedge from the nose down to the diagonal seam above the palm.
pub const LEFT_BTN: &[P] = &[
    n(200.0, 33.0),
    n(194.8, 33.0),
    n(185.7, 36.0),
    n(170.0, 42.0),
    n(138.0, 57.0),
    n(98.0, 77.0),
    n(86.0, 84.0),
    n(75.0, 92.0),
    n(66.0, 103.0),
    n(62.0, 130.0),
    n(57.0, 170.0),
    n(53.0, 200.0),
    n(51.0, 240.0),
    n(49.0, 300.0),
    n(48.0, 357.0),
    n(60.0, 352.0),
    n(90.0, 342.0),
    n(120.0, 331.0),
    n(150.0, 320.0),
    n(200.0, 300.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(255.0, 33.0),
    n(263.2, 33.0),
    n(277.8, 38.0),
    n(333.1, 63.0),
    n(371.3, 83.0),
    n(381.0, 89.0),
    n(388.0, 96.0),
    n(391.0, 130.0),
    n(396.0, 170.0),
    n(400.0, 210.0),
    n(402.0, 250.0),
    n(404.0, 300.0),
    n(405.0, 352.0),
    n(380.0, 346.0),
    n(340.0, 330.0),
    n(300.0, 314.0),
    n(262.0, 298.0),
    n(255.0, 295.0),
];

/// The diagonal seams where the palm cover meets the primary buttons (left, right).
pub const SEAM_L: &[P] = &[
    n(48.0, 358.0),
    n(60.0, 353.0),
    n(90.0, 343.0),
    n(120.0, 332.0),
    n(150.0, 321.0),
    n(200.0, 301.0),
];
pub const SEAM_R: &[P] = &[
    n(255.0, 296.0),
    n(262.0, 299.0),
    n(300.0, 315.0),
    n(340.0, 331.0),
    n(380.0, 347.0),
    n(405.0, 353.0),
];

/// The centre slot that carries the wheel and the DPI key, rounded at its rear end.
pub const SLOT: &[P] = &[
    n(200.0, 31.0),
    n(255.0, 31.0),
    n(255.0, 335.0),
    n(250.0, 349.0),
    n(240.0, 357.0),
    n(227.0, 360.0),
    n(214.0, 357.0),
    n(204.0, 349.0),
    n(200.0, 335.0),
];
/// The lit cap in front of the wheel and the spine behind it.
pub const CAP: (P, P) = ((203.0, 33.0), (253.0, 103.0));
pub const SPINE: &[P] = &[
    n(202.0, 224.0),
    n(254.0, 224.0),
    n(254.0, 335.0),
    n(249.0, 348.0),
    n(239.0, 355.0),
    n(227.0, 358.0),
    n(215.0, 355.0),
    n(205.0, 348.0),
    n(202.0, 335.0),
];

/// Wheel window and wheel.
pub const WHEEL_SLOT: (P, P) = ((203.0, 105.0), (253.0, 223.0));
pub const WHEEL: (P, P) = ((207.0, 109.0), (249.0, 217.0));
/// The DPI key: its ring and the pad inside.
pub const DPI_RING: (P, P) = ((205.0, 250.0), (249.0, 306.0));
pub const DPI_PAD: (P, P) = ((210.0, 255.0), (243.0, 297.0));

/// Left side buttons: Forward (front) and Back (rear).
pub const FORWARD: &[P] = &[
    n(34.0, 268.0),
    n(40.0, 259.0),
    n(47.0, 262.0),
    n(48.0, 302.0),
    n(41.0, 305.0),
    n(35.0, 298.0),
];
pub const BACK: &[P] = &[
    n(33.0, 309.0),
    n(40.0, 307.0),
    n(48.0, 309.0),
    n(49.0, 348.0),
    n(42.0, 353.0),
    n(35.0, 346.0),
];

/// Status LED beside the DPI key, the two battery dots and the palm mark (centre, radius).
pub const LED: P = (226.0, 323.0);
pub const DOTS: [P; 2] = [(227.0, 392.0), (227.0, 408.0)];
pub const LOGO: (f32, f32, f32) = (212.0, 558.0, 40.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (120.0, 190.0);
pub const ANCHOR_RIGHT: P = (335.0, 190.0);
pub const ANCHOR_MIDDLE: P = (228.0, 163.0);
pub const ANCHOR_DPI: P = (227.0, 276.0);
pub const ANCHOR_FORWARD: P = (41.0, 283.0);
pub const ANCHOR_BACK: P = (41.0, 329.0);

/// The smoothed outline (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| Shapes {
        outline: chaikin(&pos(OUTLINE), 1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(30.0, 30.0), (0.0, 0.0));
        let (x, y) = n(419.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.556).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let raw: [&[P]; 7] = [OUTLINE, LEFT_BTN, RIGHT_BTN, SLOT, SPINE, FORWARD, BACK];
        for (x, y) in raw.iter().flat_map(|p| p.iter().copied()) {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
        for q in &shapes().outline {
            assert!((-0.002..=1.002).contains(&q.x) && (-0.002..=1.002).contains(&q.y));
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let px = |poly: &[P]| -> Vec<Pos2> {
            poly.iter()
                .map(|p| pos2(p.0 * 389.0, p.1 * 700.0))
                .collect()
        };
        let outline: Vec<Pos2> = shapes()
            .outline
            .iter()
            .map(|p| pos2(p.x * 389.0, p.y * 700.0))
            .collect();
        assert_eq!(
            crate::illustration::shapes::triangulate(&outline).len(),
            outline.len() - 2
        );
        for (name, poly) in [
            ("left", LEFT_BTN),
            ("right", RIGHT_BTN),
            ("slot", SLOT),
            ("spine", SPINE),
            ("forward", FORWARD),
            ("back", BACK),
        ] {
            let pts = px(poly);
            assert_eq!(
                crate::illustration::shapes::triangulate(&pts).len(),
                pts.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
