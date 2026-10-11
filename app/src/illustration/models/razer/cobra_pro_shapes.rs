//! Outline and part geometry of the Cobra Pro shell, measured off a top-down product shot (a 233 x 447 px
//! box, origin at the shell's top-left) and redrawn as original vector art. Every number below
//! is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::Pos2;

use super::shared;
use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (108.0, 156.0);
pub const SIZE: P = (233.0, 447.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Geometry: sharp front corners, then the smoothed body, clockwise from the left front ---

/// Sharp: up the left front corner, over the two button tips and the wheel cover, down the right front corner.
pub const FRONT: &[P] = &[
    n(114.0, 224.0),
    n(119.0, 197.0),
    n(124.0, 190.0),
    n(130.0, 185.0),
    n(201.0, 156.0),
    n(204.0, 158.0),
    n(205.0, 168.0),
    n(209.0, 164.0),
    n(214.0, 161.0),
    n(236.0, 161.0),
    n(240.0, 164.0),
    n(243.0, 168.0),
    n(245.0, 157.0),
    n(246.0, 156.0),
    n(318.0, 185.0),
    n(324.0, 190.0),
    n(330.0, 197.0),
];

/// Smoothed: the right flank, the round tail and back up the left flank.
pub const BODY: &[P] = &[
    n(330.0, 197.0),
    n(334.0, 228.0),
    n(336.0, 274.0),
    n(333.0, 330.0),
    n(330.0, 375.0),
    n(332.0, 406.0),
    n(340.0, 457.0),
    n(340.0, 488.0),
    n(338.0, 510.0),
    n(330.0, 534.0),
    n(326.0, 538.0),
    n(318.0, 555.0),
    n(302.0, 572.0),
    n(286.0, 585.0),
    n(261.0, 598.0),
    n(237.0, 602.0),
    n(212.0, 602.0),
    n(196.0, 599.0),
    n(179.0, 593.0),
    n(162.0, 584.0),
    n(146.0, 572.0),
    n(128.0, 551.0),
    n(118.0, 532.0),
    n(112.0, 513.0),
    n(108.0, 494.0),
    n(108.0, 466.0),
    n(111.0, 420.0),
    n(113.0, 370.0),
    n(113.0, 330.0),
    n(112.0, 314.0),
    n(114.0, 260.0),
    n(114.0, 224.0),
];

/// The palm cover's lit front edge.
pub const BEVEL: &[P] = &[
    n(130.0, 351.0),
    n(168.0, 377.0),
    n(200.0, 402.0),
    n(207.0, 405.0),
    n(262.0, 405.0),
    n(272.0, 399.0),
    n(300.0, 379.0),
    n(322.0, 356.0),
];

/// Left primary button: the nose to the V-shaped rear edge.
pub const LEFT_BTN: &[P] = &[
    n(130.0, 185.0),
    n(201.0, 156.0),
    n(204.0, 158.0),
    n(204.0, 386.0),
    n(166.0, 360.0),
    n(126.0, 336.0),
    n(118.0, 326.0),
    n(114.0, 300.0),
    n(113.0, 260.0),
    n(114.0, 224.0),
    n(119.0, 197.0),
    n(124.0, 190.0),
];

/// Right primary button.
pub const RIGHT_BTN: &[P] = &[
    n(246.0, 156.0),
    n(318.0, 185.0),
    n(324.0, 190.0),
    n(330.0, 197.0),
    n(334.0, 228.0),
    n(336.0, 274.0),
    n(333.0, 310.0),
    n(328.0, 330.0),
    n(324.0, 338.0),
    n(284.0, 362.0),
    n(246.0, 386.0),
];

/// The dark V-shaped trench between the primary buttons and the palm cover.
pub const BAND: &[P] = &[
    n(125.0, 336.0),
    n(166.0, 362.0),
    n(204.0, 388.0),
    n(246.0, 388.0),
    n(285.0, 363.0),
    n(325.0, 338.0),
    n(322.0, 352.0),
    n(288.0, 378.0),
    n(262.0, 404.0),
    n(207.0, 404.0),
    n(168.0, 376.0),
    n(128.0, 350.0),
];

/// Thumb buttons on the left flank (Forward in front, Back behind it).
pub const FORWARD: &[P] = &[
    n(113.0, 298.0),
    n(121.0, 300.0),
    n(122.0, 360.0),
    n(114.0, 358.0),
];

pub const BACK: &[P] = &[
    n(113.0, 366.0),
    n(122.0, 368.0),
    n(126.0, 428.0),
    n(116.0, 424.0),
];

/// The rubber grips on the lower flanks.
pub const LEFT_GRIP: &[P] = &[
    n(113.0, 398.0),
    n(121.0, 402.0),
    n(130.0, 440.0),
    n(134.0, 500.0),
    n(125.0, 530.0),
    n(122.0, 536.0),
    n(118.0, 532.0),
    n(112.0, 513.0),
    n(108.0, 494.0),
    n(108.0, 466.0),
    n(111.0, 420.0),
];

pub const RIGHT_GRIP: &[P] = &[
    n(328.0, 400.0),
    n(335.0, 404.0),
    n(338.0, 440.0),
    n(340.0, 488.0),
    n(338.0, 510.0),
    n(330.0, 534.0),
    n(326.0, 538.0),
    n(319.0, 525.0),
    n(316.0, 500.0),
    n(320.0, 440.0),
];

/// The channel down the middle, the wheel window, the wheel and the DPI key behind it, as rect corners.
pub const CHANNEL: (P, P) = ((205.0, 157.0), (244.0, 390.0));

pub const WHEEL_SLOT: (P, P) = ((207.0, 207.0), (243.0, 279.0));

pub const WHEEL: (P, P) = ((213.0, 214.0), (237.0, 274.0));

pub const PILL: (P, P) = ((217.0, 300.0), (231.0, 351.0));

pub const LOGO: P = (231.0, 540.0);

pub const ANCHOR_LEFT: P = (160.0, 250.0);

pub const ANCHOR_RIGHT: P = (290.0, 250.0);

pub const ANCHOR_MIDDLE: P = (225.0, 245.0);

pub const ANCHOR_PILL: P = (224.0, 326.0);

pub const ANCHOR_FORWARD: P = (117.0, 330.0);

pub const ANCHOR_BACK: P = (119.0, 398.0);

pub const LOGO_RADIUS: f32 = 26.0;

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
        let underglow = shared::below(&outline, n(0.0, 300.0).1);
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
        let raw: [&[P]; 10] = [
            FRONT, BODY, BEVEL, LEFT_BTN, RIGHT_BTN, BAND, FORWARD, BACK, LEFT_GRIP, RIGHT_GRIP,
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
            LOGO,
            ANCHOR_LEFT,
            ANCHOR_RIGHT,
            ANCHOR_MIDDLE,
            ANCHOR_PILL,
            ANCHOR_FORWARD,
            ANCHOR_BACK,
        ];
        let corners = [
            CHANNEL.0,
            CHANNEL.1,
            WHEEL_SLOT.0,
            WHEEL_SLOT.1,
            WHEEL.0,
            WHEEL.1,
            PILL.0,
            PILL.1,
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
