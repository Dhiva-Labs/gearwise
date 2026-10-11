//! Outline and part geometry of the Prime shell, in a 0-1 box (x across, y front to back).
//! Measured off a reference photo (a few hundred px tall) and redrawn as original vector art.
//! Every run below is in those reference pixels; [`n`] maps them into the 0-1 art box.

use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (0.0, 0.0);
pub const SIZE: P = (350.0, 699.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise.
pub const OUTLINE: &[P] = &[
    n(188.0, 22.0),
    n(200.5, 24.4),
    n(210.5, 24.6),
    n(229.5, 31.8),
    n(260.5, 46.2),
    n(283.5, 59.8),
    n(294.5, 67.6),
    n(313.4, 84.5),
    n(326.2, 102.5),
    n(331.2, 112.5),
    n(337.2, 129.5),
    n(341.0, 147.0),
    n(345.0, 190.0),
    n(347.0, 250.0),
    n(349.0, 314.0),
    n(350.0, 420.0),
    n(349.2, 549.5),
    n(333.8, 584.5),
    n(313.2, 619.5),
    n(292.4, 647.5),
    n(270.5, 671.4),
    n(252.5, 681.8),
    n(229.5, 690.2),
    n(203.5, 696.4),
    n(186.5, 698.5),
    n(166.5, 698.5),
    n(156.5, 697.4),
    n(126.5, 691.2),
    n(100.5, 681.8),
    n(89.5, 676.2),
    n(83.5, 671.5),
    n(64.6, 651.5),
    n(52.6, 636.5),
    n(38.8, 616.5),
    n(28.2, 598.5),
    n(16.8, 574.5),
    n(7.8, 550.5),
    n(3.6, 535.5),
    n(0.5, 517.5),
    n(0.6, 506.5),
    n(6.0, 468.0),
    n(10.0, 418.0),
    n(8.0, 350.0),
    n(8.0, 304.0),
    n(10.0, 276.0),
    n(12.0, 246.0),
    n(15.4, 233.5),
    n(21.6, 147.5),
    n(26.6, 120.5),
    n(31.8, 101.5),
    n(38.2, 84.6),
    n(43.4, 82.2),
    n(46.6, 75.5),
    n(58.5, 64.6),
    n(81.5, 48.8),
    n(102.5, 37.2),
    n(120.5, 29.8),
    n(142.5, 23.1),
    n(160.0, 21.0),
    n(175.0, 20.0),
];
pub const LEFT_BTN: &[P] = &[
    n(43.0, 82.0),
    n(47.0, 76.0),
    n(58.0, 65.0),
    n(82.0, 49.0),
    n(102.0, 37.0),
    n(120.0, 30.0),
    n(142.0, 23.0),
    n(160.0, 21.0),
    n(172.0, 22.0),
    n(172.0, 290.0),
    n(160.0, 291.0),
    n(120.0, 298.0),
    n(60.0, 286.0),
    n(40.0, 270.0),
    n(38.0, 200.0),
    n(40.0, 120.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(182.0, 22.0),
    n(188.0, 22.0),
    n(200.0, 24.0),
    n(210.0, 25.0),
    n(230.0, 32.0),
    n(260.0, 46.0),
    n(284.0, 60.0),
    n(294.0, 68.0),
    n(313.0, 84.0),
    n(318.0, 100.0),
    n(318.0, 270.0),
    n(300.0, 282.0),
    n(250.0, 298.0),
    n(231.0, 300.0),
    n(182.0, 292.0),
];
pub const WELL: &[P] = &[
    n(152.0, 66.0),
    n(172.0, 60.0),
    n(184.0, 60.0),
    n(206.0, 66.0),
    n(207.0, 170.0),
    n(190.0, 184.0),
    n(188.0, 202.0),
    n(174.0, 202.0),
    n(172.0, 184.0),
    n(152.0, 170.0),
];
pub const SLOT: &[P] = &[
    n(172.0, 22.0),
    n(182.0, 22.0),
    n(182.0, 292.0),
    n(172.0, 292.0),
];
pub const FORWARD: &[P] = &[n(8.0, 298.0), n(17.0, 298.0), n(18.0, 346.0), n(8.0, 346.0)];
pub const BACK: &[P] = &[n(8.0, 354.0), n(17.0, 354.0), n(18.0, 404.0), n(9.0, 404.0)];
pub const SEAM_L: &[P] = &[
    n(40.0, 270.0),
    n(60.0, 286.0),
    n(120.0, 298.0),
    n(160.0, 291.0),
    n(172.0, 291.0),
];
pub const SEAM_R: &[P] = &[
    n(182.0, 292.0),
    n(231.0, 300.0),
    n(250.0, 298.0),
    n(300.0, 282.0),
    n(318.0, 270.0),
];
pub const GAP_L: &[P] = &[n(40.0, 84.0), n(38.0, 200.0), n(40.0, 270.0)];
pub const GAP_R: &[P] = &[
    n(313.0, 84.0),
    n(318.0, 100.0),
    n(318.0, 200.0),
    n(318.0, 270.0),
];
pub const WHEEL: ((f32, f32), (f32, f32)) = ((160.0, 80.0), (198.0, 172.0));
pub const LOGO: (f32, f32, f32) = (163.0, 632.0, 30.0);
pub const ANCHOR_LEFT: (f32, f32) = (100.0, 150.0);
pub const ANCHOR_RIGHT: (f32, f32) = (250.0, 150.0);

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
        let runs: [&[P]; 11] = [
            OUTLINE, LEFT_BTN, RIGHT_BTN, WELL, SLOT, FORWARD, BACK, SEAM_L, SEAM_R, GAP_L, GAP_R,
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
