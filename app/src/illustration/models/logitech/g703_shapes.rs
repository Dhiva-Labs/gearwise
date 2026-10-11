//! Outline and part geometry of the G703 / G403 shell (the same right-handed mouse, wireless and
//! wired), measured off a top-down product shot (381 x 700 px box) and redrawn as original
//! vector art. Every constant is in those reference pixels; [`n`] maps them into the 0-1 box.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (381.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose: the thumb rest bulges out on the left, the right
/// flank is a straight grip wall.
pub const OUTLINE: &[P] = &[
    n(197.1, 30.0),
    n(197.1, 33.0),
    n(261.2, 34.0),
    n(301.6, 51.0),
    n(332.0, 65.0),
    n(352.4, 76.0),
    n(366.9, 85.0),
    n(376.1, 92.0),
    n(385.2, 101.0),
    n(387.6, 105.0),
    n(392.5, 110.0),
    n(400.8, 124.0),
    n(406.4, 139.0),
    n(408.7, 149.0),
    n(410.0, 159.0),
    n(410.3, 175.0),
    n(411.0, 180.0),
    n(411.0, 223.0),
    n(410.3, 228.0),
    n(409.0, 262.0),
    n(403.3, 342.0),
    n(402.0, 389.0),
    n(401.2, 398.0),
    n(401.0, 424.0),
    n(400.3, 430.0),
    n(400.0, 453.0),
    n(399.3, 458.0),
    n(399.0, 474.0),
    n(397.0, 506.0),
    n(394.0, 534.0),
    n(389.9, 561.0),
    n(385.3, 582.0),
    n(377.4, 608.0),
    n(371.9, 622.0),
    n(365.8, 635.0),
    n(353.5, 655.0),
    n(340.0, 672.0),
    n(328.1, 684.0),
    n(318.7, 692.0),
    n(299.7, 705.0),
    n(286.9, 712.0),
    n(275.6, 717.0),
    n(255.1, 724.0),
    n(241.3, 727.0),
    n(227.3, 729.0),
    n(192.6, 729.0),
    n(168.7, 725.0),
    n(158.2, 722.0),
    n(144.2, 717.0),
    n(126.9, 709.0),
    n(112.1, 700.0),
    n(99.9, 691.0),
    n(84.7, 677.0),
    n(73.7, 665.0),
    n(61.5, 648.0),
    n(54.1, 635.0),
    n(44.2, 613.0),
    n(37.6, 594.0),
    n(32.3, 571.0),
    n(30.0, 554.0),
    n(30.0, 524.0),
    n(32.0, 505.0),
    n(36.9, 475.0),
    n(48.9, 418.0),
    n(49.0, 399.0),
    n(49.9, 381.0),
    n(52.1, 375.0),
    n(57.2, 368.0),
    n(56.5, 366.0),
    n(53.2, 363.0),
    n(51.2, 360.0),
    n(50.0, 356.0),
    n(49.7, 320.0),
    n(49.0, 310.0),
    n(48.7, 289.0),
    n(47.0, 264.0),
    n(46.8, 250.0),
    n(46.0, 245.0),
    n(45.7, 231.0),
    n(45.0, 226.0),
    n(47.0, 220.0),
    n(46.9, 217.0),
    n(45.0, 200.0),
    n(42.0, 160.0),
    n(42.0, 128.0),
    n(43.7, 118.0),
    n(45.5, 112.0),
    n(50.3, 101.0),
    n(58.9, 89.0),
    n(69.5, 79.0),
    n(78.1, 73.0),
    n(86.6, 68.0),
    n(107.2, 58.0),
    n(127.8, 50.0),
    n(161.3, 39.0),
    n(193.6, 30.0),
];

/// Left primary button: from the nose down the left shoulder to the diagonal seam.
pub const LEFT_BTN: &[P] = &[
    n(195.0, 33.0),
    n(161.3, 39.0),
    n(127.8, 50.0),
    n(107.2, 58.0),
    n(86.6, 68.0),
    n(69.5, 79.0),
    n(58.9, 89.0),
    n(54.0, 100.0),
    n(55.0, 125.0),
    n(57.0, 150.0),
    n(62.0, 200.0),
    n(68.0, 250.0),
    n(72.0, 300.0),
    n(77.0, 340.0),
    n(79.0, 345.0),
    n(120.0, 330.0),
    n(160.0, 311.0),
    n(195.0, 294.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(261.0, 34.0),
    n(301.6, 51.0),
    n(332.0, 65.0),
    n(366.9, 85.0),
    n(376.1, 92.0),
    n(385.2, 101.0),
    n(391.0, 112.0),
    n(395.0, 140.0),
    n(397.0, 165.0),
    n(396.0, 200.0),
    n(393.0, 232.0),
    n(387.0, 265.0),
    n(379.0, 293.0),
    n(372.0, 325.0),
    n(367.0, 345.0),
    n(330.0, 330.0),
    n(295.0, 313.0),
    n(261.0, 293.0),
];
/// The diagonal seams where the palm cover meets the buttons (left, right).
pub const SEAM_L: &[P] = &[
    n(79.0, 345.0),
    n(120.0, 330.0),
    n(160.0, 311.0),
    n(195.0, 294.0),
];
pub const SEAM_R: &[P] = &[
    n(261.0, 293.0),
    n(295.0, 313.0),
    n(330.0, 330.0),
    n(367.0, 345.0),
];

/// The centre slot (cap, wheel window, spine) between the buttons, rounded at the rear.
pub const SLOT: &[P] = &[
    n(194.0, 31.0),
    n(262.0, 31.0),
    n(262.0, 322.0),
    n(258.0, 336.0),
    n(248.0, 344.0),
    n(228.0, 347.0),
    n(208.0, 344.0),
    n(198.0, 336.0),
    n(194.0, 322.0),
];
pub const CAP: (P, P) = ((197.0, 36.0), (259.0, 111.0));
pub const SPINE: &[P] = &[
    n(197.0, 234.0),
    n(259.0, 234.0),
    n(259.0, 322.0),
    n(255.0, 334.0),
    n(246.0, 341.0),
    n(228.0, 344.0),
    n(210.0, 341.0),
    n(201.0, 334.0),
    n(197.0, 322.0),
];
/// Wheel window, the wheel and the light strip down its middle.
pub const WHEEL_SLOT: (P, P) = ((200.0, 112.0), (257.0, 236.0));
pub const WHEEL: (P, P) = ((203.0, 118.0), (254.0, 232.0));
pub const STRIP: (P, P) = ((224.5, 119.0), (232.0, 231.0));
/// The DPI key: its ring and the dark pad inside.
pub const DPI_RING: (P, P) = ((211.0, 266.0), (251.0, 331.0));
pub const DPI_PAD: (P, P) = ((225.0, 272.0), (247.0, 325.0));

/// Left side buttons: Forward (front, a strip on the shoulder wall) and Back (rear, along the
/// thumb rest).
pub const FORWARD: &[P] = &[
    n(44.0, 226.0),
    n(52.0, 224.0),
    n(58.0, 240.0),
    n(61.0, 300.0),
    n(63.0, 340.0),
    n(56.0, 349.0),
    n(49.0, 346.0),
    n(48.0, 290.0),
];
pub const BACK: &[P] = &[
    n(52.0, 374.0),
    n(60.0, 368.0),
    n(66.0, 380.0),
    n(78.0, 430.0),
    n(92.0, 480.0),
    n(100.0, 506.0),
    n(92.0, 508.0),
    n(80.0, 498.0),
    n(64.0, 450.0),
    n(52.0, 405.0),
];
/// The palm mark (centre, radius).
pub const LOGO: (f32, f32, f32) = (215.0, 527.0, 42.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (130.0, 190.0);
pub const ANCHOR_RIGHT: P = (326.0, 190.0);
pub const ANCHOR_MIDDLE: P = (228.0, 175.0);
pub const ANCHOR_DPI: P = (236.0, 298.0);
pub const ANCHOR_FORWARD: P = (54.0, 285.0);
pub const ANCHOR_BACK: P = (80.0, 420.0);

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
        let (x, y) = n(411.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.544).abs() < 0.002);
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
                .map(|p| pos2(p.0 * 381.0, p.1 * 700.0))
                .collect()
        };
        let outline: Vec<Pos2> = shapes()
            .outline
            .iter()
            .map(|p| pos2(p.x * 381.0, p.y * 700.0))
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
