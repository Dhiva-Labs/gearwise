//! Outline and part geometry of the DeathAdder V3 / V3 Pro shell, measured off a top-down product shot (a 189 x 360 px
//! box, origin at the shell's top-left) and redrawn as original vector art. Every number below
//! is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::Pos2;

use super::shared;
use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (156.0, 70.0);
pub const SIZE: P = (189.0, 360.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Geometry: sharp front corners, then the smoothed body, clockwise from the left front ---

/// Sharp: up the left front corner, over the two button tips and the wheel cover, down the right front corner.
pub const FRONT: &[P] = &[
    n(161.0, 100.0),
    n(165.0, 92.0),
    n(170.0, 88.0),
    n(213.0, 75.0),
    n(232.0, 71.0),
    n(235.0, 73.0),
    n(236.0, 80.0),
    n(239.0, 78.0),
    n(252.0, 76.0),
    n(260.0, 78.0),
    n(264.0, 80.0),
    n(266.0, 71.0),
    n(286.0, 75.0),
    n(323.0, 91.0),
    n(331.0, 97.0),
    n(335.0, 104.0),
    n(338.0, 108.0),
];

/// Smoothed: the right flank, the round tail and back up the left flank.
pub const BODY: &[P] = &[
    n(338.0, 108.0),
    n(339.0, 140.0),
    n(337.0, 202.0),
    n(340.0, 245.0),
    n(344.0, 282.0),
    n(344.0, 320.0),
    n(339.0, 347.0),
    n(331.0, 370.0),
    n(319.0, 389.0),
    n(307.0, 403.0),
    n(291.0, 416.0),
    n(273.0, 425.0),
    n(256.0, 429.0),
    n(230.0, 429.0),
    n(204.0, 422.0),
    n(194.0, 416.0),
    n(181.0, 406.0),
    n(166.0, 387.0),
    n(161.0, 376.0),
    n(158.0, 363.0),
    n(156.0, 329.0),
    n(160.0, 300.0),
    n(162.0, 262.0),
    n(162.0, 236.0),
    n(160.0, 186.0),
    n(160.0, 140.0),
    n(161.0, 100.0),
];

/// Left primary button: from the nose to the rear seam.
pub const LEFT_BTN: &[P] = &[
    n(170.0, 88.0),
    n(213.0, 75.0),
    n(232.0, 71.0),
    n(234.0, 73.0),
    n(234.0, 166.0),
    n(248.0, 167.0),
    n(249.0, 252.0),
    n(197.0, 252.0),
    n(188.0, 249.0),
    n(181.0, 242.0),
    n(177.0, 225.0),
    n(174.0, 200.0),
    n(170.0, 150.0),
    n(166.0, 102.0),
];

/// Right primary button.
pub const RIGHT_BTN: &[P] = &[
    n(266.0, 71.0),
    n(286.0, 75.0),
    n(323.0, 91.0),
    n(331.0, 97.0),
    n(334.0, 104.0),
    n(333.0, 150.0),
    n(329.0, 200.0),
    n(326.0, 228.0),
    n(322.0, 242.0),
    n(312.0, 250.0),
    n(300.0, 252.0),
    n(250.0, 252.0),
    n(250.0, 167.0),
    n(266.0, 166.0),
];

/// Thumb buttons: slivers on the left flank (Forward in front, Back behind it).
pub const FORWARD: &[P] = &[
    n(158.0, 180.0),
    n(166.0, 181.0),
    n(168.0, 232.0),
    n(160.0, 232.0),
];

pub const BACK: &[P] = &[
    n(159.0, 238.0),
    n(167.0, 238.0),
    n(169.0, 296.0),
    n(161.0, 296.0),
];

pub const CHANNEL: (P, P) = ((233.0, 70.0), (267.0, 170.0));

/// Wheel slot, wheel and the cover (with the status LED) in front of it, as rect corners.
pub const WHEEL_SLOT: (P, P) = ((236.0, 101.0), (264.0, 167.0));

pub const WHEEL: (P, P) = ((238.0, 104.0), (262.0, 164.0));

pub const COVER: (P, P) = ((237.0, 76.0), (263.0, 102.0));

pub const LED: P = (249.0, 89.0);

pub const LOGO: P = (247.0, 374.0);

pub const ANCHOR_LEFT: P = (205.0, 170.0);

pub const ANCHOR_RIGHT: P = (294.0, 170.0);

pub const ANCHOR_MIDDLE: P = (250.0, 134.0);

pub const ANCHOR_FORWARD: P = (164.0, 206.0);

pub const ANCHOR_BACK: P = (165.0, 267.0);

pub const LOGO_RADIUS: f32 = 24.0;

/// The silhouette and anything derived from it, built once.
pub struct Shapes {
    pub outline: Vec<Pos2>,
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let outline = shared::outline(FRONT, BODY);
        Shapes { outline }
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
        let raw: [&[P]; 6] = [FRONT, BODY, LEFT_BTN, RIGHT_BTN, FORWARD, BACK];
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
            LED,
            LOGO,
            ANCHOR_LEFT,
            ANCHOR_RIGHT,
            ANCHOR_MIDDLE,
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
            COVER.0,
            COVER.1,
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
