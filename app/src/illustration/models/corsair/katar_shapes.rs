//! Outline and part geometry of the Katar Pro shell, in a 0-1 box (x across, y front to back).
//! Measured off a reference photo (a few hundred px tall) and redrawn as original vector art.
//! Every run below is in those reference pixels; [`n`] maps them into the 0-1 art box.

use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (0.0, 0.0);
pub const SIZE: P = (396.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise.
pub const OUTLINE: &[P] = &[
    n(168.5, 0.9),
    n(170.4, 1.6),
    n(171.4, 3.5),
    n(171.6, 28.5),
    n(173.5, 30.4),
    n(178.5, 29.5),
    n(230.4, 30.1),
    n(231.5, 27.5),
    n(231.6, 2.5),
    n(233.5, 0.6),
    n(351.5, 41.8),
    n(363.4, 48.6),
    n(366.2, 53.5),
    n(370.4, 74.5),
    n(382.4, 149.5),
    n(389.4, 211.5),
    n(393.5, 261.5),
    n(394.5, 295.5),
    n(395.5, 300.5),
    n(395.5, 346.5),
    n(392.4, 394.5),
    n(388.4, 427.5),
    n(383.4, 456.5),
    n(374.4, 498.5),
    n(356.2, 567.5),
    n(351.2, 580.5),
    n(339.8, 605.5),
    n(329.2, 623.5),
    n(314.4, 643.5),
    n(303.5, 655.5),
    n(293.5, 664.4),
    n(274.5, 678.2),
    n(257.5, 687.2),
    n(237.5, 694.2),
    n(224.5, 697.4),
    n(209.5, 699.5),
    n(192.5, 699.4),
    n(171.5, 696.4),
    n(150.5, 690.2),
    n(129.5, 680.2),
    n(108.5, 665.4),
    n(92.5, 650.5),
    n(75.8, 629.5),
    n(65.8, 613.5),
    n(55.2, 592.5),
    n(45.8, 569.5),
    n(22.6, 485.5),
    n(14.6, 447.5),
    n(12.4, 432.5),
    n(8.8, 422.5),
    n(5.6, 406.5),
    n(2.6, 381.5),
    n(0.5, 350.5),
    n(0.6, 335.5),
    n(1.9, 331.5),
    n(6.1, 325.5),
    n(4.6, 321.5),
    n(4.5, 313.5),
    n(6.4, 308.5),
    n(2.9, 303.5),
    n(1.5, 298.5),
    n(6.6, 229.5),
    n(11.4, 218.5),
    n(13.6, 191.5),
    n(22.6, 127.5),
    n(32.6, 71.5),
    n(37.2, 51.5),
    n(41.5, 46.6),
    n(46.5, 43.8),
];
pub const LEFT_BTN: &[P] = &[
    n(6.4, 308.5),
    n(2.9, 303.5),
    n(1.5, 298.5),
    n(6.6, 229.5),
    n(11.4, 218.5),
    n(13.6, 191.5),
    n(22.6, 127.5),
    n(32.6, 71.5),
    n(37.2, 51.5),
    n(41.5, 46.6),
    n(46.5, 43.8),
    n(168.5, 0.9),
    n(170.4, 1.6),
    n(171.4, 3.5),
    n(171.6, 28.5),
    n(171.0, 262.0),
    n(169.0, 268.0),
    n(4.0, 317.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(231.5, 27.5),
    n(231.6, 2.5),
    n(233.5, 0.6),
    n(351.5, 41.8),
    n(363.4, 48.6),
    n(366.2, 53.5),
    n(370.4, 74.5),
    n(382.4, 149.5),
    n(389.4, 211.5),
    n(393.5, 261.5),
    n(394.5, 295.5),
    n(395.5, 300.5),
    n(395.5, 346.5),
    n(232.0, 270.0),
    n(232.0, 262.0),
];
pub const SLOT: &[P] = &[
    n(172.0, 28.0),
    n(232.0, 28.0),
    n(232.0, 262.0),
    n(200.0, 285.0),
    n(172.0, 262.0),
];
pub const HEX: &[P] = &[
    n(200.0, 217.0),
    n(226.0, 229.0),
    n(226.0, 260.0),
    n(200.0, 271.0),
    n(172.0, 260.0),
    n(172.0, 229.0),
];
pub const CHEVRON: &[P] = &[
    n(169.0, 262.0),
    n(200.0, 278.0),
    n(231.0, 262.0),
    n(231.0, 274.0),
    n(200.0, 286.0),
    n(169.0, 274.0),
];
pub const FORWARD: &[P] = &[n(1.0, 236.0), n(8.0, 236.0), n(9.0, 296.0), n(2.0, 298.0)];
pub const BACK: &[P] = &[n(1.0, 336.0), n(9.0, 332.0), n(10.0, 396.0), n(4.0, 398.0)];
pub const SEAM_L: &[P] = &[n(2.0, 318.0), n(169.0, 268.0)];
pub const SEAM_R: &[P] = &[n(231.0, 270.0), n(396.0, 338.0)];
pub const WHEEL: ((f32, f32), (f32, f32)) = ((180.0, 74.0), (223.0, 181.0));
pub const LOGO: (f32, f32, f32) = (192.0, 552.0, 36.0);
pub const ANCHOR_LEFT: (f32, f32) = (95.0, 160.0);
pub const ANCHOR_RIGHT: (f32, f32) = (300.0, 160.0);

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(ORIGIN.0, ORIGIN.1), (0.0, 0.0));
        let (x, y) = n(ORIGIN.0 + SIZE.0, ORIGIN.1 + SIZE.1);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn every_run_stays_inside_the_art_box() {
        let runs: [&[P]; 10] = [
            OUTLINE, LEFT_BTN, RIGHT_BTN, SLOT, HEX, CHEVRON, FORWARD, BACK, SEAM_L, SEAM_R,
        ];
        for run in runs {
            for (x, y) in run {
                assert!(
                    (-0.003..=1.003).contains(x) && (-0.003..=1.003).contains(y),
                    "({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn the_outline_triangulates_cleanly() {
        let px: Vec<_> = OUTLINE
            .iter()
            .map(|p| pos2(p.0 * SIZE.0, p.1 * SIZE.1))
            .collect();
        assert_eq!(
            crate::illustration::shapes::triangulate(&px).len(),
            px.len() - 2
        );
    }
}
