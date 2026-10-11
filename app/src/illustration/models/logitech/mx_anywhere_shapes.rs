//! Outline and part geometry of the MX Anywhere 3S / 3, measured off a top-down product shot
//! (450 x 700 px box) and redrawn as original vector art. Every constant is in those reference
//! pixels; [`n`] maps them into the 0-1 art box. A compact, symmetric shell: a domed lid inside
//! a lighter skirt, one long rounded channel down the middle with the MagSpeed wheel, the mode
//! button and a status light, and two thumb buttons on the left flank.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (450.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose (the small step on the left is the thumb buttons).
pub const OUTLINE: &[P] = &[
    n(277.3, 30.0),
    n(306.3, 33.0),
    n(336.0, 38.0),
    n(366.7, 45.0),
    n(389.4, 52.0),
    n(405.7, 60.0),
    n(419.0, 70.0),
    n(427.3, 80.0),
    n(434.7, 93.0),
    n(440.4, 108.0),
    n(448.3, 139.0),
    n(457.0, 186.0),
    n(462.0, 225.0),
    n(465.0, 269.0),
    n(465.0, 335.0),
    n(464.3, 341.0),
    n(464.1, 358.0),
    n(474.0, 420.0),
    n(478.0, 453.0),
    n(480.0, 477.0),
    n(480.0, 514.0),
    n(478.0, 535.0),
    n(473.3, 562.0),
    n(469.4, 577.0),
    n(460.9, 601.0),
    n(451.9, 620.0),
    n(440.4, 639.0),
    n(430.1, 653.0),
    n(417.2, 667.0),
    n(401.9, 681.0),
    n(386.9, 692.0),
    n(370.2, 702.0),
    n(347.0, 713.0),
    n(329.9, 719.0),
    n(310.3, 724.0),
    n(292.3, 727.0),
    n(268.4, 729.0),
    n(241.6, 729.0),
    n(218.6, 727.0),
    n(199.7, 724.0),
    n(180.1, 719.0),
    n(163.0, 713.0),
    n(139.8, 702.0),
    n(123.1, 692.0),
    n(105.7, 679.0),
    n(92.8, 667.0),
    n(80.1, 653.0),
    n(67.5, 636.0),
    n(58.1, 620.0),
    n(49.1, 601.0),
    n(41.6, 580.0),
    n(36.7, 562.0),
    n(32.0, 535.0),
    n(30.0, 514.0),
    n(30.0, 477.0),
    n(31.0, 463.0),
    n(35.0, 427.0),
    n(42.8, 377.0),
    n(41.0, 353.0),
    n(40.0, 322.0),
    n(43.5, 307.0),
    n(45.5, 302.0),
    n(44.8, 294.0),
    n(45.0, 269.0),
    n(48.0, 225.0),
    n(53.0, 186.0),
    n(58.7, 153.0),
    n(67.7, 114.0),
    n(75.3, 93.0),
    n(82.7, 80.0),
    n(91.0, 70.0),
    n(104.3, 60.0),
    n(120.6, 52.0),
    n(143.3, 45.0),
    n(174.1, 38.0),
    n(203.7, 33.0),
    n(232.7, 30.0),
];

/// The lid's seam, left half from the nose down to the tail; the right half is its mirror
/// image about the centre line.
pub const LID_L: &[P] = &[
    n(253.0, 37.0),
    n(204.0, 39.0),
    n(150.0, 46.0),
    n(108.0, 62.0),
    n(80.0, 90.0),
    n(66.0, 130.0),
    n(59.0, 180.0),
    n(56.0, 240.0),
    n(54.0, 290.0),
    n(56.0, 350.0),
    n(59.0, 400.0),
    n(62.0, 450.0),
    n(66.0, 500.0),
    n(72.0, 545.0),
    n(85.0, 590.0),
    n(105.0, 630.0),
    n(135.0, 662.0),
    n(175.0, 685.0),
    n(220.0, 698.0),
    n(255.0, 702.0),
];
/// The channel: a long pill (corner radius `CHANNEL_R`) that carries wheel, button and light.
pub const CHANNEL: (P, P) = ((208.0, 72.0), (299.0, 472.0));
pub const CHANNEL_R: f32 = 46.0;
/// Wheel window, the metal wheel in it and the mode button below.
pub const WHEEL_SLOT: (P, P) = ((225.0, 103.0), (284.0, 213.0));
pub const WHEEL: (P, P) = ((233.0, 109.0), (277.0, 207.0));
pub const MODE_BTN: (P, P) = ((238.0, 238.0), (272.0, 312.0));
/// The status light under the mode button (centre, half height).
pub const LED: (P, f32) = ((255.0, 338.0), 8.0);

/// Left and right primary buttons: the lid either side of the channel, down to the palm.
pub const LEFT_BTN: &[P] = &[
    n(206.0, 40.0),
    n(150.0, 46.0),
    n(108.0, 62.0),
    n(80.0, 90.0),
    n(66.0, 130.0),
    n(59.0, 180.0),
    n(56.0, 240.0),
    n(54.0, 290.0),
    n(56.0, 350.0),
    n(61.0, 400.0),
    n(70.0, 440.0),
    n(207.0, 440.0),
    n(207.0, 120.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(300.0, 120.0),
    n(300.0, 440.0),
    n(440.0, 440.0),
    n(449.0, 400.0),
    n(454.0, 350.0),
    n(456.0, 290.0),
    n(454.0, 240.0),
    n(451.0, 180.0),
    n(444.0, 130.0),
    n(430.0, 90.0),
    n(402.0, 62.0),
    n(360.0, 46.0),
    n(304.0, 40.0),
];
/// The two thumb buttons on the left flank: Forward (front) and Back (rear).
pub const FORWARD: &[P] = &[
    n(40.0, 300.0),
    n(49.0, 297.0),
    n(58.0, 302.0),
    n(58.0, 338.0),
    n(48.0, 342.0),
    n(41.0, 336.0),
];
pub const BACK: &[P] = &[
    n(41.0, 346.0),
    n(48.0, 344.0),
    n(58.0, 346.0),
    n(60.0, 386.0),
    n(50.0, 392.0),
    n(42.0, 380.0),
];

/// The badge position; the mouse has no light there, so nothing is drawn.
pub const LOGO: (f32, f32, f32) = (255.0, 400.0, 14.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (130.0, 230.0);
pub const ANCHOR_RIGHT: P = (380.0, 230.0);
pub const ANCHOR_MIDDLE: P = (255.0, 160.0);
pub const ANCHOR_MODE: P = (255.0, 275.0);
pub const ANCHOR_FORWARD: P = (49.0, 320.0);
pub const ANCHOR_BACK: P = (50.0, 366.0);

/// The smoothed outline and the closed lid (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
    pub lid: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let half = chaikin(&pos(LID_L), 2);
        // Left half top to bottom, then the right half back up (the box is centred on x = 0.5).
        let mut lid = half.clone();
        lid.extend(half.iter().rev().skip(1).map(|q| pos2(1.0 - q.x, q.y)));
        Shapes {
            outline: chaikin(&pos(OUTLINE), 1),
            lid,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(30.0, 30.0), (0.0, 0.0));
        let (x, y) = n(480.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.643).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let raw: [&[P]; 6] = [OUTLINE, LID_L, LEFT_BTN, RIGHT_BTN, FORWARD, BACK];
        for (x, y) in raw.iter().flat_map(|p| p.iter().copied()) {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
        let s = shapes();
        for q in s.outline.iter().chain(&s.lid) {
            assert!((-0.002..=1.002).contains(&q.x) && (-0.002..=1.002).contains(&q.y));
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let px = |poly: &[Pos2]| -> Vec<Pos2> {
            poly.iter()
                .map(|p| pos2(p.x * 450.0, p.y * 700.0))
                .collect()
        };
        let s = shapes();
        let raw = |poly: &[P]| -> Vec<Pos2> { px(&pos(poly)) };
        let all: Vec<(&str, Vec<Pos2>)> = vec![
            ("outline", px(&s.outline)),
            ("lid", px(&s.lid)),
            ("left", raw(LEFT_BTN)),
            ("right", raw(RIGHT_BTN)),
            ("forward", raw(FORWARD)),
            ("back", raw(BACK)),
        ];
        for (name, pts) in all {
            assert_eq!(
                crate::illustration::shapes::triangulate(&pts).len(),
                pts.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
