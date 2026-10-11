//! Outline and part geometry of the Viper V3 Pro shell, measured off a top-down product shot (a 182 x 363 px
//! box, origin at the shell's top-left) and redrawn as original vector art. Every number below
//! is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::Pos2;

use super::shared;
use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (160.0, 72.0);
pub const SIZE: P = (182.0, 363.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Geometry: sharp front corners, then the smoothed body, clockwise from the left front ---

/// Sharp: up the left front corner, over the two button tips and the wheel cover, down the right front corner.
pub const FRONT: &[P] = &[
    n(164.0, 104.0),
    n(168.0, 97.0),
    n(174.0, 93.0),
    n(181.0, 90.0),
    n(210.0, 79.0),
    n(234.0, 72.0),
    n(235.0, 74.0),
    n(236.0, 84.0),
    n(237.0, 86.0),
    n(244.0, 82.0),
    n(258.0, 82.0),
    n(261.0, 86.0),
    n(264.0, 86.0),
    n(265.0, 84.0),
    n(266.0, 73.0),
    n(269.0, 72.0),
    n(296.0, 81.0),
    n(324.0, 92.0),
    n(331.0, 96.0),
    n(336.0, 100.0),
    n(338.0, 108.0),
];

/// Smoothed: the right flank, the round tail and back up the left flank.
pub const BODY: &[P] = &[
    n(338.0, 108.0),
    n(338.0, 156.0),
    n(335.0, 206.0),
    n(335.0, 246.0),
    n(341.0, 336.0),
    n(340.0, 352.0),
    n(337.0, 368.0),
    n(332.0, 384.0),
    n(326.0, 395.0),
    n(316.0, 407.0),
    n(307.0, 415.0),
    n(297.0, 422.0),
    n(282.0, 429.0),
    n(269.0, 432.0),
    n(252.0, 434.0),
    n(232.0, 432.0),
    n(214.0, 427.0),
    n(198.0, 418.0),
    n(185.0, 407.0),
    n(175.0, 394.0),
    n(166.0, 376.0),
    n(162.0, 358.0),
    n(160.0, 337.0),
    n(162.0, 294.0),
    n(162.0, 252.0),
    n(162.0, 200.0),
    n(162.0, 134.0),
    n(164.0, 104.0),
];

/// Left primary button: from the nose to the rear seam.
pub const LEFT_BTN: &[P] = &[
    n(174.0, 93.0),
    n(181.0, 90.0),
    n(210.0, 79.0),
    n(234.0, 72.0),
    n(235.0, 74.0),
    n(235.0, 172.0),
    n(249.0, 174.0),
    n(250.0, 247.0),
    n(197.0, 247.0),
    n(187.0, 244.0),
    n(178.0, 237.0),
    n(173.0, 226.0),
    n(172.0, 170.0),
    n(171.0, 100.0),
];

/// Right primary button.
pub const RIGHT_BTN: &[P] = &[
    n(266.0, 73.0),
    n(269.0, 72.0),
    n(296.0, 81.0),
    n(324.0, 92.0),
    n(329.0, 96.0),
    n(329.0, 100.0),
    n(328.0, 170.0),
    n(327.0, 226.0),
    n(322.0, 237.0),
    n(313.0, 244.0),
    n(303.0, 247.0),
    n(251.0, 247.0),
    n(251.0, 174.0),
    n(265.0, 172.0),
];

/// Thumb buttons: slivers on the left flank (Forward in front, Back behind it).
pub const FORWARD: &[P] = &[
    n(160.0, 192.0),
    n(168.0, 192.0),
    n(169.0, 243.0),
    n(162.0, 243.0),
];

pub const BACK: &[P] = &[
    n(161.0, 248.0),
    n(169.0, 248.0),
    n(170.0, 298.0),
    n(163.0, 298.0),
];

/// The wheel channel, the dark window the wheel turns in, the wheel and the cover (with the status LED) in front of it, as rect corners.
pub const CHANNEL: (P, P) = ((234.0, 72.0), (266.0, 176.0));

pub const WHEEL_SLOT: (P, P) = ((237.5, 114.0), (262.5, 174.0));

pub const WHEEL: (P, P) = ((240.0, 118.0), (260.0, 170.0));

pub const COVER: (P, P) = ((240.0, 82.0), (260.0, 114.0));

pub const LED: P = (249.0, 97.0);

pub const LOGO: P = (250.0, 371.0);

pub const ANCHOR_LEFT: P = (205.0, 160.0);

pub const ANCHOR_RIGHT: P = (296.0, 160.0);

pub const ANCHOR_MIDDLE: P = (250.0, 144.0);

pub const ANCHOR_FORWARD: P = (165.0, 218.0);

pub const ANCHOR_BACK: P = (166.0, 272.0);

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
