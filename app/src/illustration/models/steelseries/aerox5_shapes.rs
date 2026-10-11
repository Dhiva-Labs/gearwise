//! Outline and part geometry of the Aerox 5 shell, in a 0-1 box (x across, y front to back).
//! Measured off a reference photo (a few hundred px tall) and redrawn as original vector art.
//! Every run below is in those reference pixels; [`n`] maps them into the 0-1 art box.

use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (0.0, 0.0);
pub const SIZE: P = (366.0, 699.0);

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise.
pub const OUTLINE: &[P] = &[
    n(145.5, 0.8),
    n(150.5, 0.6),
    n(153.1, 2.5),
    n(153.6, 20.5),
    n(155.5, 22.4),
    n(179.5, 21.9),
    n(211.4, 23.1),
    n(212.4, 21.5),
    n(212.9, 3.5),
    n(215.5, 0.9),
    n(218.5, 0.6),
    n(261.5, 15.8),
    n(295.5, 32.2),
    n(328.5, 52.8),
    n(336.2, 61.5),
    n(344.1, 74.5),
    n(349.2, 88.5),
    n(353.4, 115.5),
    n(353.5, 170.5),
    n(352.5, 175.5),
    n(352.5, 217.5),
    n(351.5, 222.5),
    n(351.5, 321.5),
    n(352.5, 326.5),
    n(352.5, 351.5),
    n(354.6, 382.5),
    n(364.4, 461.5),
    n(365.5, 496.5),
    n(362.2, 514.5),
    n(351.6, 548.5),
    n(352.5, 562.5),
    n(350.4, 574.5),
    n(342.2, 599.5),
    n(332.2, 619.5),
    n(318.4, 638.5),
    n(296.5, 660.4),
    n(275.5, 677.2),
    n(258.5, 687.2),
    n(233.5, 694.4),
    n(211.5, 697.4),
    n(197.5, 698.5),
    n(169.5, 698.5),
    n(146.5, 696.4),
    n(123.5, 692.2),
    n(110.5, 687.8),
    n(94.5, 679.2),
    n(67.5, 657.4),
    n(48.6, 638.5),
    n(38.6, 626.5),
    n(26.8, 609.5),
    n(18.2, 593.5),
    n(10.6, 572.5),
    n(8.5, 559.5),
    n(9.4, 548.5),
    n(4.6, 528.5),
    n(0.6, 505.5),
    n(0.5, 482.5),
    n(1.6, 471.5),
    n(12.9, 393.5),
    n(7.6, 339.5),
    n(4.5, 275.5),
    n(7.5, 257.5),
    n(6.6, 228.5),
    n(12.5, 218.5),
    n(9.5, 203.5),
    n(11.5, 180.5),
    n(11.6, 113.5),
    n(18.8, 78.5),
    n(24.2, 65.5),
    n(32.6, 53.5),
    n(52.5, 38.8),
    n(72.5, 27.9),
    n(92.5, 18.8),
];
pub const LEFT_BTN: &[P] = &[
    n(92.0, 19.0),
    n(146.0, 1.0),
    n(153.0, 2.0),
    n(154.0, 20.0),
    n(154.0, 205.0),
    n(158.0, 300.0),
    n(110.0, 298.0),
    n(60.0, 282.0),
    n(8.0, 262.0),
    n(7.0, 228.0),
    n(12.0, 218.0),
    n(10.0, 204.0),
    n(12.0, 180.0),
    n(12.0, 114.0),
    n(19.0, 78.0),
    n(24.0, 66.0),
    n(33.0, 54.0),
    n(52.0, 39.0),
    n(72.0, 28.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(218.0, 1.0),
    n(262.0, 16.0),
    n(296.0, 32.0),
    n(328.0, 53.0),
    n(336.0, 62.0),
    n(344.0, 74.0),
    n(349.0, 88.0),
    n(353.0, 116.0),
    n(354.0, 170.0),
    n(352.0, 176.0),
    n(352.0, 218.0),
    n(350.0, 270.0),
    n(300.0, 295.0),
    n(222.0, 300.0),
    n(212.0, 205.0),
    n(212.0, 22.0),
    n(213.0, 4.0),
    n(216.0, 1.0),
];
pub const WELL: &[P] = &[
    n(154.0, 20.0),
    n(212.0, 20.0),
    n(212.0, 205.0),
    n(204.0, 215.0),
    n(200.0, 250.0),
    n(166.0, 250.0),
    n(162.0, 215.0),
    n(154.0, 205.0),
];
pub const DPI_KEY: &[P] = &[
    n(168.0, 252.0),
    n(198.0, 252.0),
    n(200.0, 262.0),
    n(200.0, 325.0),
    n(190.0, 335.0),
    n(176.0, 335.0),
    n(166.0, 325.0),
    n(166.0, 262.0),
];
pub const PERF_L: &[P] = &[
    n(52.0, 198.0),
    n(140.0, 172.0),
    n(146.0, 254.0),
    n(58.0, 262.0),
];
pub const PERF_R: &[P] = &[
    n(224.0, 148.0),
    n(338.0, 148.0),
    n(344.0, 238.0),
    n(224.0, 262.0),
];
pub const PERF_BODY: &[P] = &[
    n(68.0, 312.0),
    n(332.0, 312.0),
    n(338.0, 400.0),
    n(342.0, 480.0),
    n(337.0, 560.0),
    n(320.0, 612.0),
    n(292.0, 652.0),
    n(256.0, 682.0),
    n(210.0, 693.0),
    n(160.0, 689.0),
    n(120.0, 673.0),
    n(80.0, 641.0),
    n(50.0, 601.0),
    n(34.0, 560.0),
    n(32.0, 500.0),
    n(48.0, 440.0),
    n(64.0, 380.0),
];
pub const FORWARD: &[P] = &[
    n(5.0, 226.0),
    n(19.0, 222.0),
    n(24.0, 262.0),
    n(11.0, 272.0),
];
pub const BACK: &[P] = &[
    n(5.0, 284.0),
    n(19.0, 280.0),
    n(24.0, 330.0),
    n(11.0, 334.0),
];
pub const FLICK: &[P] = &[
    n(8.0, 344.0),
    n(21.0, 340.0),
    n(26.0, 396.0),
    n(13.0, 404.0),
];
pub const SEAM: &[P] = &[
    n(8.0, 262.0),
    n(60.0, 282.0),
    n(110.0, 298.0),
    n(158.0, 301.0),
    n(222.0, 301.0),
    n(300.0, 295.0),
    n(350.0, 270.0),
];
pub const WHEEL: ((f32, f32), (f32, f32)) = ((166.0, 90.0), (202.0, 198.0));
pub const LOGO: (f32, f32, f32) = (90.0, 178.0, 15.0);
pub const ANCHOR_LEFT: (f32, f32) = (80.0, 130.0);
pub const ANCHOR_RIGHT: (f32, f32) = (285.0, 130.0);

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
        let runs: [&[P]; 12] = [
            OUTLINE, LEFT_BTN, RIGHT_BTN, WELL, DPI_KEY, PERF_L, PERF_R, PERF_BODY, FORWARD, BACK,
            FLICK, SEAM,
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
