//! Outline and part geometry of the G903 shell: an ambidextrous, angular mouse with a metal-look
//! centre spine (wheel, a key, two DPI keys and a bank of three LEDs) between two wing-shaped
//! primary buttons, thumb buttons on both flanks and a ridged, domed palm cover. Measured off a
//! top-down product shot (353 x 700 px box) and redrawn as original vector art; every constant
//! is in those reference pixels and [`n`] maps them into the 0-1 art box. The left half is
//! measured, the right half is its mirror image about the centre line.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (353.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose (the small steps are the thumb buttons).
pub const OUTLINE: &[P] = &[
    n(232.1, 30.0),
    n(242.3, 35.0),
    n(252.6, 39.0),
    n(271.9, 51.0),
    n(314.1, 80.0),
    n(335.7, 97.0),
    n(341.5, 104.0),
    n(345.8, 111.0),
    n(357.5, 139.0),
    n(365.5, 161.0),
    n(369.0, 176.0),
    n(369.0, 212.0),
    n(368.3, 221.0),
    n(375.0, 279.0),
    n(375.3, 289.0),
    n(376.0, 294.0),
    n(376.3, 323.0),
    n(377.0, 328.0),
    n(377.0, 379.0),
    n(376.0, 385.0),
    n(372.0, 390.0),
    n(371.6, 397.0),
    n(375.4, 403.0),
    n(375.0, 413.0),
    n(372.2, 433.0),
    n(371.7, 441.0),
    n(376.1, 459.0),
    n(381.0, 487.0),
    n(383.0, 506.0),
    n(383.0, 532.0),
    n(381.0, 548.0),
    n(377.3, 568.0),
    n(371.3, 590.0),
    n(365.7, 605.0),
    n(359.9, 619.0),
    n(347.9, 642.0),
    n(337.5, 658.0),
    n(326.1, 673.0),
    n(314.2, 686.0),
    n(302.3, 697.0),
    n(290.0, 706.0),
    n(278.1, 713.0),
    n(267.2, 718.0),
    n(250.2, 724.0),
    n(236.3, 727.0),
    n(221.4, 729.0),
    n(190.7, 729.0),
    n(176.8, 727.0),
    n(160.9, 723.0),
    n(146.6, 718.0),
    n(132.5, 711.0),
    n(115.9, 700.0),
    n(105.0, 691.0),
    n(93.0, 679.0),
    n(84.7, 669.0),
    n(72.5, 652.0),
    n(62.1, 635.0),
    n(57.2, 626.0),
    n(50.2, 611.0),
    n(44.5, 597.0),
    n(39.6, 582.0),
    n(36.9, 571.0),
    n(33.0, 551.0),
    n(30.0, 525.0),
    n(30.0, 518.0),
    n(32.0, 494.0),
    n(34.1, 477.0),
    n(37.6, 458.0),
    n(32.0, 428.0),
    n(30.0, 406.0),
    n(30.9, 403.0),
    n(36.5, 399.0),
    n(38.1, 397.0),
    n(38.0, 388.0),
    n(37.7, 387.0),
    n(30.1, 382.0),
    n(30.0, 359.0),
    n(30.7, 354.0),
    n(35.0, 281.0),
    n(37.0, 256.0),
    n(39.8, 231.0),
    n(39.7, 216.0),
    n(39.0, 211.0),
    n(39.0, 179.0),
    n(41.6, 167.0),
    n(51.5, 138.0),
    n(59.1, 119.0),
    n(65.5, 107.0),
    n(72.8, 98.0),
    n(77.3, 94.0),
    n(108.5, 71.0),
    n(145.1, 46.0),
    n(154.9, 40.0),
    n(164.5, 36.0),
    n(175.9, 30.0),
];

/// Left primary button: a big wing from the nose out to the shoulder and down the flank.
pub const LEFT_BTN: &[P] = &[
    n(172.0, 31.0),
    n(164.5, 36.0),
    n(154.9, 40.0),
    n(145.1, 46.0),
    n(108.5, 71.0),
    n(77.3, 94.0),
    n(72.8, 98.0),
    n(65.5, 107.0),
    n(59.1, 119.0),
    n(51.5, 138.0),
    n(41.6, 167.0),
    n(39.0, 179.0),
    n(39.0, 211.0),
    n(40.0, 231.0),
    n(43.0, 260.0),
    n(46.0, 300.0),
    n(49.0, 325.0),
    n(56.0, 335.0),
    n(80.0, 322.0),
    n(105.0, 308.0),
    n(125.0, 296.0),
    n(140.0, 255.0),
    n(150.0, 205.0),
    n(158.0, 150.0),
    n(162.0, 100.0),
    n(166.0, 60.0),
];
/// The dark gap under the wing and the thumb-button bay beneath it.
pub const BAY: &[P] = &[
    n(46.0, 300.0),
    n(49.0, 325.0),
    n(56.0, 335.0),
    n(64.0, 350.0),
    n(61.0, 397.0),
    n(160.0, 398.0),
    n(166.0, 300.0),
    n(125.0, 296.0),
    n(105.0, 308.0),
    n(80.0, 322.0),
];
/// The ridges cut into the bay.
pub const RIDGES: &[[P; 2]] = &[
    [n(80.0, 335.0), n(92.0, 384.0)],
    [n(98.0, 333.0), n(109.0, 377.0)],
    [n(114.0, 337.0), n(124.0, 373.0)],
];
/// The dark walls round the spine, and the spine itself (a lit metal column).
pub const WALL: &[P] = &[
    n(160.0, 30.0),
    n(253.0, 30.0),
    n(255.0, 100.0),
    n(263.0, 200.0),
    n(263.0, 300.0),
    n(256.0, 395.0),
    n(246.0, 446.0),
    n(167.0, 446.0),
    n(157.0, 395.0),
    n(150.0, 300.0),
    n(150.0, 200.0),
    n(158.0, 100.0),
];
pub const SPINE: &[P] = &[
    n(176.0, 30.0),
    n(237.0, 30.0),
    n(238.0, 106.0),
    n(241.0, 230.0),
    n(245.0, 300.0),
    n(246.0, 395.0),
    n(236.0, 431.0),
    n(217.0, 442.0),
    n(196.0, 442.0),
    n(177.0, 431.0),
    n(167.0, 395.0),
    n(168.0, 300.0),
    n(172.0, 230.0),
    n(175.0, 106.0),
];
/// The bright bridge at the nose of the spine, in front of the wheel window.
pub const BRIDGE: &[P] = &[
    n(176.0, 30.0),
    n(237.0, 30.0),
    n(238.0, 106.0),
    n(175.0, 106.0),
];
/// The seam under the primary buttons where the palm cover starts.
pub const COVER_Y: f32 = 397.0;

/// Wheel window, wheel and the tilt marks either side of it.
pub const WHEEL_SLOT: (P, P) = ((181.0, 109.0), (231.0, 227.0));
pub const WHEEL: (P, P) = ((187.0, 118.0), (225.0, 217.0));
pub const TILT_L: P = (150.0, 150.0);
pub const TILT_R: P = (262.0, 150.0);
/// The cycle key behind the wheel: its ring, pad and the bar under it.
pub const KEY_RING: (P, P) = ((181.0, 240.0), (231.0, 284.0));
pub const KEY_PAD: (P, P) = ((186.0, 245.0), (226.0, 266.0));
pub const KEY_BAR: (P, P) = ((188.0, 270.0), (219.0, 276.0));
/// The two DPI keys (left: down, right: up), tapering towards the rear.
pub const DPI_DOWN: &[P] = &[
    n(172.0, 301.0),
    n(203.0, 301.0),
    n(204.0, 335.0),
    n(194.0, 350.0),
    n(178.0, 346.0),
    n(171.0, 330.0),
];
pub const DPI_UP: &[P] = &[
    n(209.0, 301.0),
    n(240.0, 301.0),
    n(241.0, 330.0),
    n(234.0, 346.0),
    n(218.0, 350.0),
    n(209.0, 335.0),
];
/// The three DPI indicator bars (centres), slanted.
pub const BARS: [P; 3] = [(187.0, 376.0), (206.0, 376.0), (225.0, 376.0)];
/// The thumb tail of the spine over the palm cover.
pub const TAIL: &[P] = &[
    n(166.0, 395.0),
    n(246.0, 395.0),
    n(236.0, 431.0),
    n(217.0, 442.0),
    n(196.0, 442.0),
    n(177.0, 431.0),
];

/// Left thumb buttons: Forward (front, long) and Back (rear). The right pair mirrors them.
pub const FORWARD: &[P] = &[
    n(30.0, 236.0),
    n(38.0, 232.0),
    n(46.0, 262.0),
    n(48.0, 330.0),
    n(45.0, 384.0),
    n(36.0, 388.0),
    n(30.0, 380.0),
    n(30.0, 300.0),
];
pub const BACK: &[P] = &[
    n(32.0, 404.0),
    n(40.0, 400.0),
    n(50.0, 420.0),
    n(52.0, 450.0),
    n(46.0, 472.0),
    n(38.0, 460.0),
    n(31.0, 430.0),
];

/// Grip ridges on the palm cover's lobes (left: a few strokes; the right ones mirror them).
pub const GRIP: &[[P; 2]] = &[
    [n(70.0, 538.0), n(60.0, 640.0)],
    [n(96.0, 548.0), n(86.0, 650.0)],
    [n(130.0, 580.0), n(114.0, 680.0)],
];

/// The palm mark (centre, radius).
pub const LOGO: (f32, f32, f32) = (206.0, 527.0, 43.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (100.0, 190.0);
pub const ANCHOR_RIGHT: P = (313.0, 190.0);
pub const ANCHOR_MIDDLE: P = (206.0, 160.0);
pub const ANCHOR_CYCLE: P = (206.0, 256.0);
pub const ANCHOR_DOWN: P = (188.0, 322.0);
pub const ANCHOR_UP: P = (225.0, 322.0);
pub const ANCHOR_FORWARD: P = (38.0, 310.0);
pub const ANCHOR_BACK: P = (41.0, 436.0);

/// The smoothed outline and the palm cover (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
    /// The part of the silhouette below the cover seam, closed along the seam.
    pub cover: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

/// The shell is symmetric about x = 0.5 (the box is centred on the centre line).
pub fn mirrored(q: &[P]) -> Vec<P> {
    q.iter().rev().map(|p| (1.0 - p.0, p.1)).collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let outline = chaikin(&pos(OUTLINE), 1);
        let seam = n(0.0, COVER_Y).1;
        // Everything of the outline at or below the seam, starting where the right flank
        // crosses it and ending where the left one does.
        let cover: Vec<Pos2> = outline.iter().filter(|q| q.y >= seam).copied().collect();
        Shapes { outline, cover }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(30.0, 30.0), (0.0, 0.0));
        let (x, y) = n(383.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.504).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let raw: [&[P]; 11] = [
            OUTLINE,
            LEFT_BTN,
            BAY,
            WALL,
            SPINE,
            DPI_DOWN,
            DPI_UP,
            TAIL,
            FORWARD,
            BACK,
            &[n(206.0, 527.0)],
        ];
        for (x, y) in raw.iter().flat_map(|p| p.iter().copied()) {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
        for q in &shapes().outline {
            assert!((-0.002..=1.002).contains(&q.x) && (-0.002..=1.002).contains(&q.y));
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let px = |poly: &[P]| -> Vec<Pos2> {
            poly.iter()
                .map(|p| pos2(p.0 * 353.0, p.1 * 700.0))
                .collect()
        };
        let outline: Vec<Pos2> = shapes()
            .outline
            .iter()
            .map(|p| pos2(p.x * 353.0, p.y * 700.0))
            .collect();
        assert_eq!(
            crate::illustration::shapes::triangulate(&outline).len(),
            outline.len() - 2
        );
        for (name, poly) in [
            ("left", LEFT_BTN),
            ("bay", BAY),
            ("wall", WALL),
            ("spine", SPINE),
            ("down", DPI_DOWN),
            ("up", DPI_UP),
            ("tail", TAIL),
            ("forward", FORWARD),
            ("back", BACK),
        ] {
            let pts = px(poly);
            assert_eq!(
                crate::illustration::shapes::triangulate(&pts).len(),
                pts.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
        let right = mirrored(LEFT_BTN);
        let pts = px(&right);
        assert_eq!(
            crate::illustration::shapes::triangulate(&pts).len(),
            pts.len() - 2
        );
    }
}
