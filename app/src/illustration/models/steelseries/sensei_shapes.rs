//! Outline and part geometry of the Sensei Ten shell, in a 0-1 box (x across, y front to back).
//! Measured off a reference photo (a few hundred px tall) and redrawn as original vector art.
//! Every run below is in those reference pixels; [`n`] maps them into the 0-1 art box.

use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (0.0, 0.0);
pub const SIZE: P = (391.2, 689.6);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise.
pub const OUTLINE: &[P] = &[
    n(196.8, 0.0),
    n(227.2, 0.0),
    n(259.2, 3.2),
    n(294.4, 9.6),
    n(326.4, 22.4),
    n(352.0, 43.2),
    n(368.0, 72.0),
    n(376.0, 107.2),
    n(379.2, 144.0),
    n(379.2, 192.0),
    n(379.2, 219.2),
    n(376.0, 280.0),
    n(374.4, 328.0),
    n(371.2, 379.2),
    n(372.8, 411.2),
    n(377.6, 448.0),
    n(385.6, 480.0),
    n(390.4, 512.0),
    n(388.8, 539.2),
    n(379.2, 564.8),
    n(361.6, 595.2),
    n(329.6, 632.0),
    n(288.0, 664.0),
    n(243.2, 683.2),
    n(196.8, 689.6),
    n(150.4, 683.2),
    n(105.6, 664.0),
    n(64.0, 632.0),
    n(32.0, 595.2),
    n(14.4, 564.8),
    n(4.8, 539.2),
    n(3.2, 512.0),
    n(8.0, 480.0),
    n(16.0, 448.0),
    n(20.8, 411.2),
    n(22.4, 379.2),
    n(19.2, 328.0),
    n(17.6, 280.0),
    n(14.4, 219.2),
    n(14.4, 192.0),
    n(14.4, 144.0),
    n(17.6, 107.2),
    n(25.6, 72.0),
    n(41.6, 43.2),
    n(67.2, 22.4),
    n(99.2, 9.6),
    n(134.4, 3.2),
    n(166.4, 0.0),
];
pub const LEFT_BTN: &[P] = &[
    n(172.8, 1.6),
    n(134.4, 3.2),
    n(99.2, 9.6),
    n(67.2, 22.4),
    n(41.6, 43.2),
    n(25.6, 72.0),
    n(17.6, 107.2),
    n(14.4, 144.0),
    n(14.4, 208.0),
    n(32.0, 216.0),
    n(70.4, 256.0),
    n(134.4, 280.0),
    n(169.6, 289.6),
    n(169.6, 176.0),
    n(168.0, 64.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(225.6, 64.0),
    n(224.0, 176.0),
    n(224.0, 289.6),
    n(259.2, 280.0),
    n(323.2, 256.0),
    n(361.6, 216.0),
    n(379.2, 208.0),
    n(379.2, 144.0),
    n(376.0, 107.2),
    n(368.0, 72.0),
    n(352.0, 43.2),
    n(326.4, 22.4),
    n(294.4, 9.6),
    n(259.2, 3.2),
    n(220.8, 1.6),
];
pub const RAIL: &[P] = &[
    n(169.6, 0.0),
    n(224.0, 0.0),
    n(230.4, 120.0),
    n(233.6, 200.0),
    n(228.8, 264.0),
    n(217.6, 299.2),
    n(176.0, 299.2),
    n(164.8, 264.0),
    n(161.6, 200.0),
    n(164.8, 120.0),
];
pub const WELL: &[P] = &[
    n(166.4, 43.2),
    n(227.2, 43.2),
    n(232.0, 104.0),
    n(230.4, 155.2),
    n(211.2, 177.6),
    n(182.4, 177.6),
    n(164.8, 155.2),
    n(163.2, 104.0),
];
pub const DPI_KEY: &[P] = &[
    n(176.0, 219.2),
    n(219.2, 219.2),
    n(224.0, 228.8),
    n(204.8, 281.6),
    n(190.4, 281.6),
    n(171.2, 228.8),
];
pub const FORWARD: &[P] = &[
    n(16.0, 212.8),
    n(33.6, 212.8),
    n(40.0, 286.4),
    n(19.2, 286.4),
];
pub const BACK: &[P] = &[
    n(19.2, 296.0),
    n(40.0, 296.0),
    n(48.0, 395.2),
    n(25.6, 395.2),
];
pub const LOBE_L: &[P] = &[
    n(14.4, 408.0),
    n(28.8, 401.6),
    n(48.0, 401.6),
    n(49.6, 488.0),
    n(44.8, 539.2),
    n(32.0, 552.0),
    n(16.0, 548.8),
    n(4.8, 528.0),
    n(6.4, 472.0),
];
pub const RIGHT_FORWARD: &[P] = &[
    n(374.4, 286.4),
    n(353.6, 286.4),
    n(360.0, 212.8),
    n(377.6, 212.8),
];
pub const RIGHT_BACK: &[P] = &[
    n(368.0, 395.2),
    n(345.6, 395.2),
    n(353.6, 296.0),
    n(374.4, 296.0),
];
pub const LOBE_R: &[P] = &[
    n(387.2, 472.0),
    n(388.8, 528.0),
    n(377.6, 548.8),
    n(361.6, 552.0),
    n(348.8, 539.2),
    n(344.0, 488.0),
    n(345.6, 401.6),
    n(364.8, 401.6),
    n(379.2, 408.0),
];
pub const WHEEL: ((f32, f32), (f32, f32)) = ((176.0, 59.2), (217.6, 164.8));
pub const LOGO: (f32, f32, f32) = (196.8, 632.0, 35.2);
pub const LED: (f32, f32, f32) = (195.2, 193.6, 8.0);
pub const ANCHOR_LEFT: (f32, f32) = (94.4, 144.0);
pub const ANCHOR_RIGHT: (f32, f32) = (299.2, 144.0);

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
            OUTLINE,
            LEFT_BTN,
            RIGHT_BTN,
            RAIL,
            WELL,
            DPI_KEY,
            FORWARD,
            BACK,
            LOBE_L,
            RIGHT_FORWARD,
            RIGHT_BACK,
            LOBE_R,
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
