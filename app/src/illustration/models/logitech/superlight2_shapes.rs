//! Outline and part geometry of the G Pro X Superlight 2 shell, measured off a top-down
//! product shot (352 x 700 px box) and redrawn as original vector art. Every constant is in
//! those reference pixels; [`n`] maps them into the 0-1 art box. The classic G Pro Wireless
//! shares the shell (its right flank carries a button pair too).

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (352.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose (the steps are the shoulders where the primary
/// panels meet the flank walls and the two side buttons on the left).
pub const OUTLINE: &[P] = &[
    n(216.4, 30.0),
    n(245.2, 34.0),
    n(274.2, 42.0),
    n(301.2, 53.0),
    n(321.7, 64.0),
    n(337.3, 75.0),
    n(349.7, 87.0),
    n(357.3, 97.0),
    n(358.2, 105.0),
    n(365.8, 111.0),
    n(370.7, 123.0),
    n(375.0, 143.0),
    n(378.0, 182.0),
    n(378.0, 205.0),
    n(375.4, 261.0),
    n(375.0, 303.0),
    n(372.3, 349.0),
    n(372.0, 399.0),
    n(380.0, 479.0),
    n(382.0, 508.0),
    n(382.0, 548.0),
    n(379.0, 575.0),
    n(375.3, 594.0),
    n(369.5, 613.0),
    n(363.8, 627.0),
    n(356.5, 641.0),
    n(345.3, 658.0),
    n(333.0, 673.0),
    n(323.0, 683.0),
    n(308.5, 695.0),
    n(290.3, 707.0),
    n(274.6, 715.0),
    n(255.8, 722.0),
    n(235.3, 727.0),
    n(218.4, 729.0),
    n(193.6, 729.0),
    n(176.8, 727.0),
    n(162.8, 724.0),
    n(144.6, 718.0),
    n(123.5, 708.0),
    n(103.5, 695.0),
    n(88.0, 682.0),
    n(79.0, 673.0),
    n(66.7, 658.0),
    n(58.5, 646.0),
    n(49.2, 629.0),
    n(42.6, 613.0),
    n(36.6, 593.0),
    n(33.0, 575.0),
    n(30.0, 547.0),
    n(30.0, 509.0),
    n(33.0, 469.0),
    n(38.8, 416.0),
    n(37.8, 367.0),
    n(40.0, 358.0),
    n(39.7, 353.0),
    n(37.3, 348.0),
    n(36.0, 342.0),
    n(32.0, 288.0),
    n(32.0, 267.0),
    n(33.0, 255.0),
    n(36.0, 249.0),
    n(34.0, 204.0),
    n(34.0, 183.0),
    n(37.0, 143.0),
    n(38.7, 133.0),
    n(42.5, 120.0),
    n(46.8, 110.0),
    n(53.8, 105.0),
    n(54.7, 97.0),
    n(64.0, 85.0),
    n(78.5, 72.0),
    n(95.5, 61.0),
    n(117.4, 50.0),
    n(144.1, 40.0),
    n(171.8, 33.0),
    n(195.6, 30.0),
];

/// Left primary button: from the nose to the seam above the palm, its outer edge down the
/// left shoulder wall.
pub const LEFT_BTN: &[P] = &[
    n(199.0, 31.0),
    n(185.0, 32.0),
    n(171.8, 33.0),
    n(144.1, 40.0),
    n(117.4, 50.0),
    n(95.5, 61.0),
    n(78.5, 72.0),
    n(64.0, 85.0),
    n(54.7, 97.0),
    n(51.0, 108.0),
    n(51.0, 130.0),
    n(52.0, 160.0),
    n(53.0, 200.0),
    n(55.0, 240.0),
    n(58.0, 270.0),
    n(61.0, 300.0),
    n(64.0, 324.0),
    n(68.0, 337.0),
    n(76.0, 343.0),
    n(90.0, 345.0),
    n(199.0, 345.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(206.0, 31.0),
    n(216.4, 31.0),
    n(245.2, 34.0),
    n(274.2, 42.0),
    n(301.2, 53.0),
    n(321.7, 64.0),
    n(337.3, 75.0),
    n(349.7, 87.0),
    n(357.3, 97.0),
    n(359.0, 108.0),
    n(359.0, 130.0),
    n(358.0, 160.0),
    n(357.0, 190.0),
    n(355.0, 220.0),
    n(353.0, 250.0),
    n(351.0, 280.0),
    n(349.0, 305.0),
    n(346.0, 325.0),
    n(341.0, 337.0),
    n(333.0, 343.0),
    n(320.0, 345.0),
    n(206.0, 345.0),
];

/// Scroll wheel: the dark slot, the tyre inside it.
pub const WHEEL_SLOT: (P, P) = ((180.0, 108.0), (229.0, 231.0));
pub const WHEEL: (P, P) = ((189.0, 119.0), (220.0, 228.0));

/// Left side buttons: Forward (front) and Back (rear), slivers along the flank. The right
/// pair is their mirror image about the shell's centre line.
pub const FORWARD: &[P] = &[
    n(32.0, 251.0),
    n(40.0, 249.0),
    n(47.0, 258.0),
    n(50.0, 300.0),
    n(50.0, 338.0),
    n(42.0, 346.0),
    n(36.0, 339.0),
    n(34.0, 300.0),
    n(32.0, 272.0),
];
pub const BACK: &[P] = &[
    n(37.0, 352.0),
    n(46.0, 350.0),
    n(52.0, 380.0),
    n(53.0, 420.0),
    n(51.0, 450.0),
    n(44.0, 456.0),
    n(39.0, 448.0),
    n(38.0, 420.0),
    n(37.0, 380.0),
];
/// Status LED behind the primary buttons and the palm mark (centre, radius).
pub const LED: P = (206.0, 387.0);
pub const LOGO: (f32, f32, f32) = (206.0, 543.0, 37.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (125.0, 190.0);
pub const ANCHOR_RIGHT: P = (287.0, 190.0);
pub const ANCHOR_MIDDLE: P = (204.0, 168.0);
pub const ANCHOR_FORWARD: P = (43.0, 300.0);
pub const ANCHOR_BACK: P = (46.0, 402.0);

/// The smoothed outline and the shapes derived from it (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
    /// The right flank's side buttons: the mirror image of the left pair about the centre line
    /// (the classic G Pro Wireless carries a pair there too).
    pub right_forward: Vec<Pos2>,
    pub right_back: Vec<Pos2>,
    /// The silhouette beside each shoulder seam, top to bottom, for the lit wall bands.
    pub left_wall: Vec<Pos2>,
    pub right_wall: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

/// The shell is symmetric about x = 0.5 (the box is centred on the centre line).
fn mirrored(q: &[P]) -> Vec<Pos2> {
    q.iter().rev().map(|p| pos2(1.0 - p.0, p.1)).collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let outline = chaikin(&pos(OUTLINE), 1);
        let wall = |left: bool| -> Vec<Pos2> {
            let (lo, hi) = (n(0.0, 108.0).1, n(0.0, 560.0).1);
            let mut run: Vec<Pos2> = outline
                .iter()
                .filter(|q| q.y >= lo && q.y <= hi && (q.x < 0.5) == left)
                .copied()
                .collect();
            run.sort_by(|a, b| a.y.total_cmp(&b.y));
            run
        };
        Shapes {
            right_forward: mirrored(FORWARD),
            right_back: mirrored(BACK),
            left_wall: wall(true),
            right_wall: wall(false),
            outline,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(30.0, 30.0), (0.0, 0.0));
        let (x, y) = n(382.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.503).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let s = shapes();
        let raw: [&[P]; 5] = [OUTLINE, LEFT_BTN, RIGHT_BTN, FORWARD, BACK];
        let all = raw.iter().flat_map(|p| p.iter().copied()).chain(
            [&s.outline, &s.right_forward, &s.right_back]
                .into_iter()
                .flat_map(|v| v.iter().map(|p| (p.x, p.y))),
        );
        for (x, y) in all {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let s = shapes();
        let px = |poly: &[Pos2]| -> Vec<Pos2> {
            poly.iter()
                .map(|p| pos2(p.x * 352.0, p.y * 700.0))
                .collect()
        };
        let outline = px(&s.outline);
        assert_eq!(
            crate::illustration::shapes::triangulate(&outline).len(),
            outline.len() - 2,
            "outline does not triangulate cleanly"
        );
        for (name, poly) in [("left", LEFT_BTN), ("right", RIGHT_BTN)] {
            let pts: Vec<Pos2> = poly
                .iter()
                .map(|p| pos2(p.0 * 352.0, p.1 * 700.0))
                .collect();
            assert_eq!(
                crate::illustration::shapes::triangulate(&pts).len(),
                pts.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
