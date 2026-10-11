//! Outline and part geometry of the Harpoon shell, in a 0-1 box (x across, y front to back).
//! Measured off a reference photo (a few hundred px tall) and redrawn as original vector art.
//! Every run below is in those reference pixels; [`n`] maps them into the 0-1 art box.

use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (0.0, 0.0);
pub const SIZE: P = (403.0, 699.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise.
pub const OUTLINE: &[P] = &[
    n(167.5, 0.9),
    n(170.2, 1.8),
    n(171.4, 4.5),
    n(171.6, 21.5),
    n(173.4, 25.5),
    n(174.6, 53.5),
    n(175.6, 55.1),
    n(180.4, 56.6),
    n(181.6, 74.5),
    n(184.5, 81.8),
    n(187.5, 75.8),
    n(200.5, 71.6),
    n(218.5, 71.6),
    n(225.5, 73.2),
    n(232.5, 77.8),
    n(234.1, 76.5),
    n(234.6, 56.5),
    n(235.6, 54.6),
    n(239.4, 53.1),
    n(240.4, 51.5),
    n(240.5, 24.5),
    n(241.5, 19.5),
    n(240.5, 4.5),
    n(241.2, 1.6),
    n(243.5, 0.5),
    n(258.5, 5.2),
    n(308.5, 30.2),
    n(352.5, 46.8),
    n(360.5, 50.8),
    n(373.5, 59.6),
    n(377.2, 63.5),
    n(382.2, 73.5),
    n(385.4, 86.5),
    n(385.5, 176.5),
    n(386.5, 181.5),
    n(385.6, 185.5),
    n(386.5, 202.5),
    n(385.5, 207.5),
    n(385.5, 357.5),
    n(400.4, 455.5),
    n(402.5, 477.5),
    n(402.4, 505.5),
    n(399.4, 528.5),
    n(396.2, 542.5),
    n(387.8, 566.5),
    n(376.8, 588.5),
    n(368.2, 602.5),
    n(346.4, 632.5),
    n(324.5, 655.5),
    n(300.5, 673.2),
    n(276.5, 685.8),
    n(252.5, 694.2),
    n(230.5, 698.4),
    n(204.5, 698.5),
    n(188.5, 696.4),
    n(165.5, 691.2),
    n(150.5, 686.2),
    n(128.5, 676.8),
    n(116.5, 670.2),
    n(105.5, 663.2),
    n(85.5, 647.4),
    n(70.6, 631.5),
    n(49.8, 604.5),
    n(33.8, 581.5),
    n(24.2, 565.5),
    n(13.8, 544.5),
    n(7.8, 526.5),
    n(3.6, 510.5),
    n(0.6, 492.5),
    n(0.5, 462.5),
    n(1.6, 449.5),
    n(22.4, 316.5),
    n(29.4, 251.5),
    n(27.5, 75.5),
    n(30.8, 58.5),
    n(35.5, 50.8),
    n(51.5, 42.8),
    n(58.5, 40.6),
    n(74.5, 38.4),
    n(88.5, 34.2),
    n(107.5, 26.8),
    n(150.5, 6.8),
];
pub const LEFT_BTN: &[P] = &[
    n(22.4, 316.5),
    n(29.4, 251.5),
    n(27.5, 75.5),
    n(30.8, 58.5),
    n(35.5, 50.8),
    n(51.5, 42.8),
    n(58.5, 40.6),
    n(74.5, 38.4),
    n(88.5, 34.2),
    n(107.5, 26.8),
    n(150.5, 6.8),
    n(167.5, 0.9),
    n(170.2, 1.8),
    n(171.4, 4.5),
    n(171.6, 21.5),
    n(173.4, 25.5),
    n(176.0, 56.0),
    n(180.0, 60.0),
    n(182.0, 200.0),
    n(188.0, 215.0),
    n(190.0, 330.0),
    n(100.0, 336.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(240.5, 24.5),
    n(241.5, 19.5),
    n(240.5, 4.5),
    n(241.2, 1.6),
    n(243.5, 0.5),
    n(258.5, 5.2),
    n(308.5, 30.2),
    n(352.5, 46.8),
    n(360.5, 50.8),
    n(373.5, 59.6),
    n(377.2, 63.5),
    n(382.2, 73.5),
    n(385.4, 86.5),
    n(385.5, 176.5),
    n(386.5, 181.5),
    n(385.6, 185.5),
    n(386.5, 202.5),
    n(385.5, 207.5),
    n(385.5, 357.5),
    n(300.0, 336.0),
    n(240.0, 332.0),
    n(238.0, 215.0),
    n(236.0, 200.0),
    n(240.0, 56.0),
];
pub const SLOT: &[P] = &[
    n(176.0, 56.0),
    n(238.0, 56.0),
    n(240.0, 200.0),
    n(236.0, 215.0),
    n(236.0, 325.0),
    n(228.0, 335.0),
    n(200.0, 335.0),
    n(192.0, 325.0),
    n(190.0, 215.0),
    n(180.0, 200.0),
];
pub const DPI_KEY: &[P] = &[
    n(192.0, 228.0),
    n(235.0, 228.0),
    n(236.0, 320.0),
    n(228.0, 332.0),
    n(202.0, 332.0),
    n(194.0, 320.0),
];
pub const FORWARD: &[P] = &[
    n(28.0, 140.0),
    n(38.0, 140.0),
    n(38.0, 212.0),
    n(28.0, 212.0),
];
pub const BACK: &[P] = &[
    n(26.0, 226.0),
    n(36.0, 226.0),
    n(36.0, 300.0),
    n(25.0, 300.0),
];
pub const LED: &[P] = &[n(201.0, 245.0), n(222.0, 245.0)];
pub const WHEEL: ((f32, f32), (f32, f32)) = ((186.0, 72.0), (234.0, 200.0));
pub const LOGO: (f32, f32, f32) = (216.0, 604.0, 38.0);
pub const ANCHOR_LEFT: (f32, f32) = (100.0, 170.0);
pub const ANCHOR_RIGHT: (f32, f32) = (320.0, 170.0);

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
        let runs: [&[P]; 8] = [
            OUTLINE, LEFT_BTN, RIGHT_BTN, SLOT, DPI_KEY, FORWARD, BACK, LED,
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
