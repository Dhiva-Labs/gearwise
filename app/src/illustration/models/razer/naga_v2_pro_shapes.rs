//! Outline and part geometry of the Naga V2 Pro shell, measured off a top-down product shot (a 216 x 346 px
//! box, origin at the shell's top-left) and redrawn as original vector art. Every number below
//! is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::Pos2;

use super::shared;
use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (153.0, 87.0);
pub const SIZE: P = (216.0, 346.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Geometry: sharp front corners, then the smoothed body, clockwise from the left front ---

/// Sharp: up the left shoulder, over the two button tips and the wheel cover, down the right front corner.
pub const FRONT: &[P] = &[
    n(160.0, 162.0),
    n(162.0, 137.0),
    n(166.0, 120.0),
    n(172.0, 112.0),
    n(183.0, 104.0),
    n(232.0, 88.0),
    n(234.0, 96.0),
    n(237.0, 97.0),
    n(241.0, 94.0),
    n(250.0, 93.0),
    n(260.0, 95.0),
    n(264.0, 98.0),
    n(267.0, 90.0),
    n(268.0, 88.0),
    n(292.0, 99.0),
    n(326.0, 110.0),
    n(331.0, 118.0),
];

/// Smoothed: the right-hand wing, the round tail and back up the left flank past the thumb plate.
pub const BODY: &[P] = &[
    n(331.0, 118.0),
    n(334.0, 134.0),
    n(342.0, 158.0),
    n(352.0, 173.0),
    n(366.0, 187.0),
    n(368.0, 196.0),
    n(367.0, 256.0),
    n(362.0, 295.0),
    n(358.0, 314.0),
    n(350.0, 341.0),
    n(336.0, 375.0),
    n(329.0, 388.0),
    n(318.0, 402.0),
    n(300.0, 418.0),
    n(289.0, 425.0),
    n(276.0, 429.0),
    n(252.0, 432.0),
    n(233.0, 430.0),
    n(216.0, 424.0),
    n(202.0, 414.0),
    n(190.0, 399.0),
    n(181.0, 385.0),
    n(173.0, 369.0),
    n(167.0, 350.0),
    n(162.0, 326.0),
    n(156.0, 311.0),
    n(154.0, 290.0),
    n(154.0, 262.0),
    n(154.0, 238.0),
    n(157.0, 226.0),
    n(160.0, 218.0),
    n(160.0, 162.0),
];

/// Left primary button: nose to the end of the wheel channel; its left edge is the seam of the thumb plate.
pub const LEFT_BTN: &[P] = &[
    n(166.0, 120.0),
    n(172.0, 112.0),
    n(183.0, 104.0),
    n(232.0, 88.0),
    n(234.0, 96.0),
    n(234.0, 278.0),
    n(212.0, 282.0),
    n(196.0, 277.0),
    n(191.0, 262.0),
    n(189.0, 240.0),
    n(182.0, 215.0),
    n(172.0, 196.0),
    n(164.0, 172.0),
    n(162.0, 137.0),
];

/// Right primary button, including the shoulder.
pub const RIGHT_BTN: &[P] = &[
    n(268.0, 88.0),
    n(292.0, 99.0),
    n(326.0, 110.0),
    n(331.0, 118.0),
    n(334.0, 134.0),
    n(342.0, 158.0),
    n(352.0, 173.0),
    n(366.0, 187.0),
    n(367.0, 200.0),
    n(364.0, 250.0),
    n(352.0, 275.0),
    n(320.0, 284.0),
    n(268.0, 282.0),
];

/// The bezel round the thumb keys, on the left flank.
pub const PLATE: &[P] = &[
    n(160.0, 162.0),
    n(164.0, 175.0),
    n(172.0, 196.0),
    n(182.0, 215.0),
    n(189.0, 240.0),
    n(191.0, 268.0),
    n(188.0, 296.0),
    n(180.0, 322.0),
    n(167.0, 350.0),
    n(162.0, 326.0),
    n(156.0, 311.0),
    n(154.0, 290.0),
    n(154.0, 262.0),
    n(154.0, 238.0),
    n(157.0, 226.0),
    n(160.0, 218.0),
];

/// The wheel channel, the wheel window, the wheel and the two DPI keys behind it, as rect corners.
pub const CHANNEL: (P, P) = ((235.0, 88.0), (266.0, 278.0));

pub const WHEEL_SLOT: (P, P) = ((239.0, 125.0), (263.0, 198.0));

pub const WHEEL: (P, P) = ((241.0, 131.0), (262.0, 192.0));

pub const DPI_UP: (P, P) = ((242.0, 213.0), (260.0, 233.0));

pub const DPI_DOWN: (P, P) = ((242.0, 240.0), (260.0, 265.0));

/// The tilt chevrons printed on the primary buttons either side of the wheel.
pub const TILT_L: P = (226.0, 166.0);

pub const TILT_R: P = (274.0, 166.0);

pub const LOGO: P = (252.0, 352.0);

pub const ANCHOR_LEFT: P = (210.0, 160.0);

pub const ANCHOR_RIGHT: P = (298.0, 160.0);

pub const ANCHOR_MIDDLE: P = (251.0, 162.0);

pub const ANCHOR_TILT_L: P = (226.0, 166.0);

pub const ANCHOR_TILT_R: P = (274.0, 166.0);

pub const ANCHOR_DPI_UP: P = (251.0, 223.0);

pub const ANCHOR_DPI_DOWN: P = (251.0, 252.0);

pub const LOGO_RADIUS: f32 = 26.0;

/// The thumb plate: 12 keys in 4 rows of 3, numbered row by row from the front. The photo shows
/// one broad key top per row (the inner column) and the narrower numbered faces of the other
/// two, so the cells have those widths, outermost first, and shear a little with the flank.
pub const GRID_ORIGIN: P = (153.0, 211.0);
pub const GRID_ROW_PITCH: f32 = 27.0;
pub const GRID_COLS: [(f32, f32); 3] = [(0.0, 7.0), (7.6, 7.0), (15.2, 15.0)];
pub const GRID_ROW_H: f32 = 24.0;
pub const GRID_SHEAR: f32 = 1.2;

/// Cell `i` (0-11) of the thumb plate.
pub fn grid_cell(i: usize) -> Vec<P> {
    let (r, c) = ((i / 3) as f32, i % 3);
    let (dx, w) = GRID_COLS[c];
    let x0 = GRID_ORIGIN.0 + dx + r * GRID_SHEAR;
    let y0 = GRID_ORIGIN.1 + r * GRID_ROW_PITCH;
    let h = GRID_ROW_H;
    vec![
        n(x0, y0),
        n(x0 + w, y0 + 1.0),
        n(x0 + w, y0 + h),
        n(x0, y0 + h - 1.0),
    ]
}

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
        let raw: [&[P]; 5] = [FRONT, BODY, LEFT_BTN, RIGHT_BTN, PLATE];
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
            ANCHOR_DPI_UP,
            ANCHOR_DPI_DOWN,
        ];
        let corners = [
            CHANNEL.0,
            CHANNEL.1,
            WHEEL_SLOT.0,
            WHEEL_SLOT.1,
            WHEEL.0,
            WHEEL.1,
            DPI_UP.0,
            DPI_UP.1,
            DPI_DOWN.0,
            DPI_DOWN.1,
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
