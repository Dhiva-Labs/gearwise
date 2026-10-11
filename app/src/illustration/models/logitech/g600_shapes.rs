//! Outline and part geometry of the G600 MMO, measured off a top-down product shot (451 x 700 px
//! box, the cable excluded) and redrawn as original vector art. Every constant is in those
//! reference pixels; [`n`] maps them into the 0-1 art box. A broad, boxy right-hand shell with a
//! long seam either side of the wheel, the G8 / G7 pair behind it, the ring-finger (G-shift)
//! button down the right flank and the 12-button thumb panel bulging out of the left flank.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (451.0, 700.0);
/// Width divided by height of the shell (the cable sticks out above it).
pub const ASPECT: f32 = SIZE.0 / SIZE.1;
/// How far the cable reaches above the shell, as a fraction of its height, and its x.
pub const CABLE: f32 = 0.085;
pub const CABLE_X: f32 = 242.0;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose: the right flank bulges where the ring finger rests,
/// the thumb panel bulges on the left.
pub const OUTLINE: &[P] = &[
    n(260.2, 30.0),
    n(314.3, 50.0),
    n(374.6, 68.0),
    n(394.2, 76.0),
    n(397.3, 79.0),
    n(401.5, 85.0),
    n(412.5, 104.0),
    n(420.7, 114.0),
    n(430.8, 123.0),
    n(449.9, 137.0),
    n(458.4, 146.0),
    n(463.5, 156.0),
    n(467.3, 169.0),
    n(474.1, 204.0),
    n(477.0, 224.0),
    n(481.0, 268.0),
    n(481.0, 310.0),
    n(479.0, 336.0),
    n(475.0, 368.0),
    n(468.9, 405.0),
    n(460.3, 445.0),
    n(440.4, 520.0),
    n(427.4, 564.0),
    n(408.9, 616.0),
    n(397.7, 643.0),
    n(392.1, 653.0),
    n(386.0, 661.0),
    n(381.0, 666.0),
    n(367.1, 677.0),
    n(349.1, 688.0),
    n(328.6, 698.0),
    n(296.4, 711.0),
    n(270.9, 719.0),
    n(245.3, 725.0),
    n(216.3, 729.0),
    n(189.8, 729.0),
    n(173.8, 725.0),
    n(165.3, 720.0),
    n(143.9, 700.0),
    n(128.7, 683.0),
    n(106.5, 652.0),
    n(85.1, 617.0),
    n(71.2, 591.0),
    n(54.5, 553.0),
    n(49.7, 536.0),
    n(47.1, 522.0),
    n(46.0, 514.0),
    n(45.9, 505.0),
    n(40.9, 481.0),
    n(40.8, 476.0),
    n(42.1, 471.0),
    n(41.9, 466.0),
    n(39.7, 451.0),
    n(36.1, 440.0),
    n(34.1, 426.0),
    n(35.7, 418.0),
    n(33.0, 388.0),
    n(32.7, 363.0),
    n(33.0, 346.0),
    n(33.9, 339.0),
    n(30.0, 331.0),
    n(30.0, 300.0),
    n(31.6, 289.0),
    n(35.9, 281.0),
    n(36.7, 277.0),
    n(37.0, 268.0),
    n(39.0, 253.0),
    n(42.8, 233.0),
    n(41.8, 221.0),
    n(42.0, 208.0),
    n(47.9, 176.0),
    n(62.5, 111.0),
    n(68.6, 91.0),
    n(71.9, 83.0),
    n(74.7, 79.0),
    n(80.8, 73.0),
    n(91.4, 68.0),
    n(157.5, 51.0),
    n(224.7, 30.0),
];

/// Left primary button: from the nose down to where the seam ends, beside the thumb panel.
pub const LEFT_BTN: &[P] = &[
    n(239.0, 31.0),
    n(224.7, 30.0),
    n(157.5, 51.0),
    n(91.4, 68.0),
    n(74.7, 79.0),
    n(68.6, 91.0),
    n(62.5, 111.0),
    n(54.0, 145.0),
    n(47.9, 176.0),
    n(42.0, 208.0),
    n(43.0, 232.0),
    n(68.0, 238.0),
    n(72.0, 300.0),
    n(78.0, 385.0),
    n(239.0, 386.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(241.0, 31.0),
    n(260.2, 30.0),
    n(314.3, 50.0),
    n(374.6, 68.0),
    n(388.0, 72.0),
    n(392.0, 150.0),
    n(388.0, 250.0),
    n(381.0, 330.0),
    n(374.0, 386.0),
    n(241.0, 386.0),
];
/// The ring-finger (G-shift) button: the strip right of the long curved seam.
pub const SHIFT: &[P] = &[
    n(392.0, 80.0),
    n(412.5, 104.0),
    n(430.8, 123.0),
    n(449.9, 137.0),
    n(463.5, 156.0),
    n(467.3, 169.0),
    n(474.1, 204.0),
    n(481.0, 268.0),
    n(481.0, 310.0),
    n(475.0, 368.0),
    n(468.9, 405.0),
    n(460.3, 445.0),
    n(440.0, 480.0),
    n(364.0, 484.0),
    n(368.0, 440.0),
    n(374.0, 400.0),
    n(381.0, 340.0),
    n(388.0, 280.0),
    n(392.0, 200.0),
    n(393.0, 140.0),
];
/// The long dark seam down the right side, and the centre seams (nose, between wheel and G8,
/// behind G7).
pub const SEAM_SHIFT: &[P] = &[
    n(388.0, 72.0),
    n(392.0, 110.0),
    n(393.0, 150.0),
    n(391.0, 200.0),
    n(388.0, 250.0),
    n(384.0, 300.0),
    n(378.0, 350.0),
    n(372.0, 400.0),
    n(367.0, 445.0),
    n(364.0, 482.0),
];
pub const SEAMS: &[[P; 2]] = &[
    [n(241.0, 34.0), n(241.0, 108.0)],
    [n(241.0, 222.0), n(241.0, 243.0)],
    [n(241.0, 342.0), n(241.0, 386.0)],
];

/// Wheel window, wheel and the tilt chevrons either side of it.
pub const WHEEL_SLOT: (P, P) = ((216.0, 108.0), (270.0, 221.0));
pub const WHEEL: (P, P) = ((224.0, 118.0), (262.0, 212.0));
pub const TILT_L: P = (201.0, 167.0);
pub const TILT_R: P = (284.0, 169.0);
/// G8 (front) and G7 (rear) behind the wheel.
pub const G8: (P, P) = ((220.0, 244.0), (268.0, 285.0));
pub const G7: (P, P) = ((220.0, 294.0), (268.0, 340.0));

/// The thumb panel's pocket (an outline of the flank, closed by its inner edge) and its cells:
/// four columns front to back, three rows (bottom, middle, top) across the panel.
pub const PANEL: &[P] = &[
    n(42.8, 233.0),
    n(52.0, 234.0),
    n(67.0, 238.0),
    n(68.0, 300.0),
    n(68.0, 400.0),
    n(67.0, 480.0),
    n(66.0, 525.0),
    n(47.1, 522.0),
    n(46.0, 514.0),
    n(45.9, 505.0),
    n(40.9, 481.0),
    n(40.8, 476.0),
    n(42.1, 471.0),
    n(41.9, 466.0),
    n(39.7, 451.0),
    n(36.1, 440.0),
    n(34.1, 426.0),
    n(35.7, 418.0),
    n(33.0, 388.0),
    n(32.7, 363.0),
    n(33.0, 346.0),
    n(33.9, 339.0),
    n(30.0, 331.0),
    n(30.0, 300.0),
    n(31.6, 289.0),
    n(35.9, 281.0),
    n(36.7, 277.0),
    n(37.0, 268.0),
    n(39.0, 253.0),
];
pub const CELL_X: [f32; 3] = [38.0, 49.0, 60.0];
pub const CELL_Y: [f32; 4] = [274.0, 342.0, 410.0, 478.0];
pub const CELL_SIZE: (f32, f32) = (9.5, 52.0);

/// The palm mark (centre, radius).
pub const LOGO: (f32, f32, f32) = (241.0, 566.0, 42.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (150.0, 190.0);
pub const ANCHOR_RIGHT: P = (330.0, 190.0);
pub const ANCHOR_MIDDLE: P = (243.0, 165.0);
pub const ANCHOR_G8: P = (244.0, 265.0);
pub const ANCHOR_G7: P = (244.0, 317.0);
pub const ANCHOR_SHIFT: P = (432.0, 270.0);

/// The cell `i` of the thumb panel (Grid(i), G9 = 0): column `i / 3` front to back, row `i % 3`
/// from the bottom, as a centre in reference px.
pub fn cell(i: usize) -> P {
    (CELL_X[i % 3], CELL_Y[(i / 3).min(3)])
}

/// The smoothed outline (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
    pub seam_shift: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| Shapes {
        outline: chaikin(&pos(OUTLINE), 1),
        seam_shift: chaikin(&pos(SEAM_SHIFT), 2),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(30.0, 30.0), (0.0, 0.0));
        let (x, y) = n(481.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.644).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let raw: [&[P]; 6] = [OUTLINE, LEFT_BTN, RIGHT_BTN, SHIFT, SEAM_SHIFT, PANEL];
        for (x, y) in raw.iter().flat_map(|p| p.iter().copied()) {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
        for q in shapes().outline.iter() {
            assert!((-0.002..=1.002).contains(&q.x) && (-0.002..=1.002).contains(&q.y));
        }
    }

    #[test]
    fn the_twelve_cells_tile_the_panel_in_order() {
        let cells: Vec<P> = (0..12).map(cell).collect();
        // Column by column front to back, bottom row at the outer edge.
        assert!(cells[0].1 < cells[3].1 && cells[3].1 < cells[9].1);
        assert!(cells[0].0 < cells[1].0 && cells[1].0 < cells[2].0);
        for (x, y) in &cells {
            assert!(*x > 30.0 && *x < 68.0 && *y > 233.0 && *y < 525.0);
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let px = |poly: &[Pos2]| -> Vec<Pos2> {
            poly.iter()
                .map(|p| pos2(p.x * 451.0, p.y * 700.0))
                .collect()
        };
        let raw = |poly: &[P]| -> Vec<Pos2> { px(&pos(poly)) };
        let all: Vec<(&str, Vec<Pos2>)> = vec![
            ("outline", px(&shapes().outline)),
            ("left", raw(LEFT_BTN)),
            ("right", raw(RIGHT_BTN)),
            ("shift", raw(SHIFT)),
            ("panel", raw(PANEL)),
        ];
        for (name, pts) in all {
            assert_eq!(
                crate::illustration::shapes::triangulate(&pts).len(),
                pts.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
