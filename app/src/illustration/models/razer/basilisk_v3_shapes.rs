//! Outline and part geometry of the Basilisk V3 / V3 Pro shell, measured off a top-down product shot (a 287 x 496 px
//! box, origin at the shell's top-left) and redrawn as original vector art. Every number below
//! is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::Pos2;

use super::shared;
use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (75.0, 127.0);
pub const SIZE: P = (287.0, 496.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Geometry: sharp front corners, then the smoothed body, clockwise from the left front ---

/// Sharp: up the left shoulder, over the two button tips and the wheel cover, down the right front corner.
pub const FRONT: &[P] = &[
    n(117.0, 316.0),
    n(119.0, 271.0),
    n(122.0, 266.0),
    n(129.0, 221.0),
    n(134.0, 208.0),
    n(163.0, 180.0),
    n(214.0, 138.0),
    n(219.0, 139.0),
    n(220.0, 154.0),
    n(222.0, 157.0),
    n(258.0, 159.0),
    n(260.0, 155.0),
    n(261.0, 128.0),
    n(331.0, 173.0),
    n(338.0, 180.0),
    n(342.0, 186.0),
    n(351.0, 234.0),
    n(356.0, 271.0),
];

/// Smoothed: the right grip, the round tail and back up the left flank past the thumb rest.
pub const BODY: &[P] = &[
    n(356.0, 271.0),
    n(362.0, 485.0),
    n(358.0, 513.0),
    n(351.0, 539.0),
    n(346.0, 552.0),
    n(340.0, 557.0),
    n(334.0, 568.0),
    n(325.0, 582.0),
    n(307.0, 599.0),
    n(286.0, 612.0),
    n(260.0, 620.0),
    n(234.0, 622.0),
    n(206.0, 617.0),
    n(181.0, 606.0),
    n(158.0, 588.0),
    n(146.0, 576.0),
    n(138.0, 563.0),
    n(129.0, 555.0),
    n(86.0, 493.0),
    n(78.0, 478.0),
    n(76.0, 447.0),
    n(76.0, 416.0),
    n(79.0, 379.0),
    n(83.0, 359.0),
    n(92.0, 344.0),
    n(117.0, 316.0),
];

/// The palm cover's lit front edge, along the trench.
pub const PALM_EDGE: &[P] = &[
    n(130.0, 448.0),
    n(160.0, 447.0),
    n(228.0, 404.0),
    n(262.0, 386.0),
    n(300.0, 358.0),
    n(338.0, 336.0),
    n(358.0, 322.0),
];

/// The seams down either side of the palm cover.
pub const PALM_LEFT: &[P] = &[
    n(150.0, 452.0),
    n(153.0, 500.0),
    n(163.0, 540.0),
    n(175.0, 567.0),
];

pub const PALM_RIGHT: &[P] = &[
    n(322.0, 345.0),
    n(318.0, 400.0),
    n(317.0, 450.0),
    n(322.0, 500.0),
    n(332.0, 545.0),
    n(338.0, 556.0),
];

/// Left primary button: nose to the diagonal rear edge.
pub const LEFT_BTN: &[P] = &[
    n(214.0, 138.0),
    n(220.0, 141.0),
    n(221.0, 391.0),
    n(180.0, 410.0),
    n(140.0, 430.0),
    n(126.0, 430.0),
    n(123.0, 380.0),
    n(125.0, 340.0),
    n(128.0, 300.0),
    n(127.0, 268.0),
    n(124.0, 250.0),
    n(129.0, 221.0),
    n(134.0, 208.0),
    n(163.0, 180.0),
];

/// Right primary button.
pub const RIGHT_BTN: &[P] = &[
    n(261.0, 128.0),
    n(331.0, 173.0),
    n(338.0, 180.0),
    n(342.0, 186.0),
    n(351.0, 234.0),
    n(356.0, 271.0),
    n(355.0, 298.0),
    n(320.0, 322.0),
    n(290.0, 340.0),
    n(259.0, 363.0),
    n(259.0, 162.0),
];

/// The dark diagonal trench between the buttons and the palm cover.
pub const TRENCH: &[P] = &[
    n(124.0, 432.0),
    n(140.0, 430.0),
    n(180.0, 410.0),
    n(221.0, 391.0),
    n(259.0, 363.0),
    n(290.0, 340.0),
    n(320.0, 322.0),
    n(355.0, 298.0),
    n(356.0, 320.0),
    n(338.0, 336.0),
    n(300.0, 358.0),
    n(262.0, 386.0),
    n(228.0, 404.0),
    n(160.0, 447.0),
    n(130.0, 448.0),
];

/// The DPI key behind the wheel-mode key (its rear corner is cut).
pub const DPI_BTN: &[P] = &[
    n(229.0, 347.0),
    n(252.0, 347.0),
    n(252.0, 368.0),
    n(244.0, 377.0),
    n(229.0, 377.0),
];

/// Thumb keys on the left wall: the DPI clutch at the front, then Forward and Back.
pub const SNIPER: &[P] = &[
    n(118.0, 268.0),
    n(126.0, 266.0),
    n(127.0, 330.0),
    n(119.0, 332.0),
];

pub const FORWARD: &[P] = &[
    n(122.0, 384.0),
    n(133.0, 386.0),
    n(141.0, 438.0),
    n(131.0, 440.0),
];

pub const BACK: &[P] = &[
    n(131.0, 446.0),
    n(142.0, 448.0),
    n(150.0, 500.0),
    n(140.0, 502.0),
];

/// The rubber grips: the thumb rest on the left, the pinky wall on the right.
pub const LEFT_GRIP: &[P] = &[
    n(117.0, 316.0),
    n(92.0, 344.0),
    n(83.0, 359.0),
    n(79.0, 379.0),
    n(76.0, 416.0),
    n(76.0, 447.0),
    n(78.0, 478.0),
    n(86.0, 493.0),
    n(129.0, 555.0),
    n(138.0, 563.0),
    n(146.0, 574.0),
    n(158.0, 560.0),
    n(152.0, 520.0),
    n(145.0, 480.0),
    n(133.0, 440.0),
    n(122.0, 400.0),
    n(119.0, 360.0),
];

pub const RIGHT_GRIP: &[P] = &[
    n(354.0, 300.0),
    n(357.0, 330.0),
    n(360.0, 400.0),
    n(362.0, 485.0),
    n(358.0, 513.0),
    n(351.0, 539.0),
    n(346.0, 552.0),
    n(336.0, 556.0),
    n(325.0, 520.0),
    n(322.0, 470.0),
    n(326.0, 430.0),
    n(333.0, 380.0),
    n(341.0, 332.0),
    n(349.0, 310.0),
];

/// The wheel window, the wheel, the pocket behind it and the wheel-mode key, as rect corners.
pub const WHEEL_SLOT: (P, P) = ((221.0, 208.0), (258.0, 306.0));

pub const WHEEL: (P, P) = ((229.0, 213.0), (250.0, 300.0));

pub const MODE_BTN: (P, P) = ((229.0, 316.0), (251.0, 340.0));

pub const CHANNEL: (P, P) = ((219.0, 140.0), (260.0, 394.0));

/// The tilt chevrons printed on the primary buttons either side of the wheel.
pub const TILT_L: P = (209.0, 248.0);

pub const TILT_R: P = (268.0, 248.0);

pub const LOGO: P = (233.0, 547.0);

pub const ANCHOR_LEFT: P = (172.0, 250.0);

pub const ANCHOR_RIGHT: P = (306.0, 250.0);

pub const ANCHOR_MIDDLE: P = (239.0, 256.0);

pub const ANCHOR_TILT_L: P = (209.0, 248.0);

pub const ANCHOR_TILT_R: P = (268.0, 248.0);

pub const ANCHOR_MODE: P = (240.0, 328.0);

pub const ANCHOR_DPI: P = (240.0, 360.0);

pub const ANCHOR_SNIPER: P = (122.0, 300.0);

pub const ANCHOR_FORWARD: P = (131.0, 412.0);

pub const ANCHOR_BACK: P = (141.0, 474.0);

pub const LOGO_RADIUS: f32 = 28.0;

/// The silhouette and anything derived from it, built once.
pub struct Shapes {
    pub outline: Vec<Pos2>,
    /// The run of the silhouette that carries the underglow strip.
    pub underglow: Vec<Pos2>,
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let outline = shared::outline(FRONT, BODY);
        let underglow = shared::below(&outline, n(0.0, 330.0).1);
        Shapes { outline, underglow }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(ORIGIN.0, ORIGIN.1), (0.0, 0.0));
        let (x, y) = n(ORIGIN.0 + SIZE.0, ORIGIN.1 + SIZE.1);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let raw: [&[P]; 14] = [
            FRONT, BODY, PALM_EDGE, PALM_LEFT, PALM_RIGHT, LEFT_BTN, RIGHT_BTN, TRENCH, DPI_BTN,
            SNIPER, FORWARD, BACK, LEFT_GRIP, RIGHT_GRIP,
        ];
        let outline = shapes().outline.iter().map(|p| (p.x, p.y));
        for (x, y) in raw.iter().flat_map(|p| p.iter().copied()).chain(outline) {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
        let inside =
            |p: P| (0.0..=1.0).contains(&n(p.0, p.1).0) && (0.0..=1.0).contains(&n(p.0, p.1).1);
        let points: &[P] = &[
            TILT_L,
            TILT_R,
            LOGO,
            ANCHOR_LEFT,
            ANCHOR_RIGHT,
            ANCHOR_MIDDLE,
            ANCHOR_TILT_L,
            ANCHOR_TILT_R,
            ANCHOR_MODE,
            ANCHOR_DPI,
            ANCHOR_SNIPER,
            ANCHOR_FORWARD,
            ANCHOR_BACK,
        ];
        let corners = [
            WHEEL_SLOT.0,
            WHEEL_SLOT.1,
            WHEEL.0,
            WHEEL.1,
            MODE_BTN.0,
            MODE_BTN.1,
            CHANNEL.0,
            CHANNEL.1,
        ];
        assert!(points.iter().copied().all(inside));
        assert!(corners.iter().copied().all(inside));
    }

    #[test]
    fn the_outline_is_a_simple_polygon() {
        let px: Vec<Pos2> = shapes()
            .outline
            .iter()
            .map(|p| egui::pos2(p.x * 400.0, p.y * 690.0))
            .collect();
        let tris = crate::illustration::shapes::triangulate(&px);
        assert_eq!(
            tris.len(),
            px.len() - 2,
            "the outline does not triangulate cleanly"
        );
    }
}
