//! Outline and part geometry of the G502 X family (X, X Lightspeed, X Plus), measured off a
//! top-down product shot (416 x 700 px box) and redrawn as original vector art. Every constant
//! is in those reference pixels; [`n`] maps them into the 0-1 art box. The X is a smoother
//! shell than the HERO: one straight channel (wheel, a cycle key and G9) between two long
//! primary buttons, the G8 / G7 pair and a DPI-shift paddle on a dark flank, a ribbed thumb
//! grip, and on the Plus a light strip that sweeps round the palm.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (30.0, 30.0);
pub const SIZE: P = (416.0, 700.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

/// The silhouette, clockwise from the nose.
pub const OUTLINE: &[P] = &[
    n(298.2, 30.0),
    n(303.2, 31.0),
    n(328.3, 44.0),
    n(356.1, 61.0),
    n(374.5, 74.0),
    n(379.8, 79.0),
    n(382.1, 85.0),
    n(392.5, 90.0),
    n(397.8, 99.0),
    n(401.4, 108.0),
    n(411.3, 144.0),
    n(421.3, 187.0),
    n(431.1, 237.0),
    n(436.0, 270.0),
    n(438.0, 304.0),
    n(435.2, 370.0),
    n(435.3, 395.0),
    n(437.1, 425.0),
    n(444.0, 469.0),
    n(446.0, 488.0),
    n(446.0, 513.0),
    n(442.1, 542.0),
    n(432.5, 586.0),
    n(422.1, 618.0),
    n(415.8, 633.0),
    n(409.5, 645.0),
    n(403.3, 652.0),
    n(390.1, 657.0),
    n(383.9, 666.0),
    n(382.7, 676.0),
    n(378.9, 683.0),
    n(367.5, 694.0),
    n(352.0, 705.0),
    n(333.3, 715.0),
    n(318.3, 721.0),
    n(301.2, 726.0),
    n(281.4, 729.0),
    n(246.6, 729.0),
    n(229.7, 727.0),
    n(214.8, 724.0),
    n(193.8, 717.0),
    n(184.9, 713.0),
    n(173.1, 706.0),
    n(155.0, 691.0),
    n(143.5, 677.0),
    n(134.2, 662.0),
    n(127.2, 648.0),
    n(104.6, 594.0),
    n(97.7, 580.0),
    n(90.5, 568.0),
    n(81.9, 556.0),
    n(71.8, 545.0),
    n(60.4, 535.0),
    n(41.3, 520.0),
    n(35.5, 514.0),
    n(33.1, 510.0),
    n(30.7, 503.0),
    n(30.0, 491.0),
    n(40.0, 414.0),
    n(51.7, 343.0),
    n(55.3, 336.0),
    n(67.5, 327.0),
    n(70.7, 323.0),
    n(69.0, 313.0),
    n(69.0, 292.0),
    n(76.1, 252.0),
    n(84.1, 211.0),
    n(86.2, 206.0),
    n(86.7, 199.0),
    n(94.7, 164.0),
    n(109.8, 103.0),
    n(115.1, 88.0),
    n(121.3, 80.0),
    n(129.0, 75.0),
    n(181.8, 57.0),
    n(215.0, 47.0),
    n(267.7, 35.0),
    n(293.7, 30.0),
];

/// Left primary button: a long panel from the nose to the diagonal seam, beside the channel.
pub const LEFT_BTN: &[P] = &[
    n(141.0, 68.0),
    n(181.8, 57.0),
    n(215.0, 47.0),
    n(227.0, 44.0),
    n(229.0, 100.0),
    n(229.0, 200.0),
    n(229.0, 300.0),
    n(229.0, 376.0),
    n(190.0, 386.0),
    n(160.0, 395.0),
    n(130.0, 403.0),
    n(108.0, 413.0),
    n(114.0, 360.0),
    n(122.0, 300.0),
    n(126.0, 260.0),
    n(131.0, 190.0),
    n(135.0, 140.0),
    n(139.0, 95.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(306.0, 33.0),
    n(328.3, 44.0),
    n(356.1, 61.0),
    n(374.5, 74.0),
    n(382.0, 86.0),
    n(398.0, 108.0),
    n(412.0, 170.0),
    n(425.0, 230.0),
    n(433.0, 290.0),
    n(433.0, 318.0),
    n(420.0, 320.0),
    n(380.0, 333.0),
    n(340.0, 342.0),
    n(309.0, 350.0),
    n(308.0, 300.0),
    n(308.0, 200.0),
    n(306.0, 100.0),
];
/// The centre channel: wheel, cycle key and G9, rounded at its rear end.
pub const CHANNEL: &[P] = &[
    n(228.0, 45.0),
    n(304.0, 31.0),
    n(307.0, 100.0),
    n(308.0, 200.0),
    n(308.0, 350.0),
    n(306.0, 395.0),
    n(298.0, 410.0),
    n(280.0, 417.0),
    n(250.0, 418.0),
    n(236.0, 412.0),
    n(229.0, 395.0),
    n(229.0, 300.0),
    n(229.0, 100.0),
];
/// The lit chamfer across the nose of the channel.
pub const CHAMFER: &[P] = &[
    n(229.0, 48.0),
    n(303.0, 33.0),
    n(305.0, 62.0),
    n(230.0, 66.0),
];
/// The palm cover: from the diagonal seams and the channel's rear end down to the light strip.
pub const PALM_TOP: &[P] = &[
    n(108.0, 413.0),
    n(229.0, 377.0),
    n(229.0, 395.0),
    n(236.0, 412.0),
    n(250.0, 418.0),
    n(280.0, 417.0),
    n(298.0, 410.0),
    n(306.0, 395.0),
    n(307.0, 350.0),
    n(312.0, 350.0),
    n(340.0, 342.0),
    n(380.0, 333.0),
    n(420.0, 320.0),
    n(433.0, 318.0),
];

/// G8 (front) and G7 (rear): the stacked bright wedges on the flank, left of the left button.
pub const G8: &[P] = &[
    n(119.0, 88.0),
    n(125.0, 77.0),
    n(141.0, 68.0),
    n(139.0, 95.0),
    n(135.0, 140.0),
    n(131.0, 190.0),
    n(114.0, 190.0),
    n(101.0, 188.0),
    n(104.0, 165.0),
    n(111.0, 118.0),
];
pub const G7: &[P] = &[
    n(96.0, 213.0),
    n(130.0, 198.0),
    n(126.0, 260.0),
    n(122.0, 262.0),
    n(118.0, 300.0),
    n(113.0, 340.0),
    n(110.0, 357.0),
    n(96.0, 338.0),
    n(84.5, 321.0),
    n(81.0, 305.0),
    n(83.0, 280.0),
    n(87.0, 255.0),
    n(92.0, 230.0),
];
/// The DPI-shift paddle on the dark flank, below and beside G7.
pub const SNIPER: &[P] = &[
    n(70.0, 268.0),
    n(82.0, 262.0),
    n(86.0, 290.0),
    n(84.0, 321.0),
    n(96.0, 338.0),
    n(108.0, 354.0),
    n(94.0, 360.0),
    n(76.0, 350.0),
    n(68.0, 322.0),
    n(67.0, 290.0),
];
/// Thumb buttons on the flank: G5 (front) and G4 (rear).
pub const FORWARD: &[P] = &[
    n(53.0, 340.0),
    n(60.0, 330.0),
    n(66.0, 330.0),
    n(62.0, 350.0),
    n(58.0, 395.0),
    n(48.0, 395.0),
    n(42.0, 380.0),
    n(46.0, 355.0),
];
pub const BACK: &[P] = &[
    n(44.0, 402.0),
    n(56.0, 400.0),
    n(62.0, 430.0),
    n(60.0, 480.0),
    n(52.0, 492.0),
    n(40.0, 470.0),
    n(40.0, 430.0),
];
/// The ribbed thumb grip down the left flank.
pub const GRIP: &[P] = &[
    n(52.0, 343.0),
    n(67.0, 328.0),
    n(80.0, 345.0),
    n(96.0, 352.0),
    n(104.0, 380.0),
    n(101.0, 433.0),
    n(112.0, 488.0),
    n(125.0, 530.0),
    n(100.0, 546.0),
    n(71.8, 545.0),
    n(60.4, 535.0),
    n(41.3, 520.0),
    n(35.5, 514.0),
    n(30.7, 503.0),
    n(30.0, 491.0),
    n(40.0, 414.0),
];

/// The rows (reference y) the thumb grip's ribs span.
pub const GRIP_RIBS: (f32, f32) = (352.0, 540.0);

/// Wheel window, wheel and the tilt chevrons either side of the channel.
pub const WHEEL_SLOT: (P, P) = ((244.0, 143.0), (297.0, 261.0));
pub const WHEEL: (P, P) = ((251.0, 152.0), (289.0, 256.0));
pub const TILT_L: P = (212.0, 200.0);
pub const TILT_R: P = (317.0, 200.0);
/// The keys behind the wheel: the ring round both, the cycle key and G9.
pub const KEY_RING: (P, P) = ((253.0, 276.0), (289.0, 378.0));
pub const CYCLE_KEY: (P, P) = ((258.0, 283.0), (284.0, 320.0));
pub const G9_KEY: (P, P) = ((259.0, 337.0), (284.0, 372.0));
/// The status light in the nose of the channel (centre, half length, tilt in radians).
pub const STATUS: (P, f32, f32) = ((268.0, 103.0), 15.0, -0.2);

/// The light strip's centre line, from the thumb grip round the heel of the palm and up the
/// right flank, and the spur that branches off to the right heel.
pub const STRIP: &[P] = &[
    n(101.0, 433.0),
    n(106.0, 462.0),
    n(112.0, 488.0),
    n(125.0, 530.0),
    n(146.0, 556.0),
    n(178.0, 572.0),
    n(220.0, 583.0),
    n(272.0, 590.0),
    n(315.0, 590.0),
    n(351.0, 587.0),
    n(370.0, 575.0),
    n(388.0, 514.0),
    n(404.0, 467.0),
    n(417.0, 425.0),
    n(425.0, 383.0),
];
pub const SPUR: &[P] = &[
    n(368.0, 574.0),
    n(376.0, 612.0),
    n(383.0, 656.0),
    n(390.0, 670.0),
];

/// The palm mark (centre, radius).
pub const LOGO: (f32, f32, f32) = (176.0, 454.0, 36.0);

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: P = (180.0, 210.0);
pub const ANCHOR_RIGHT: P = (370.0, 200.0);
pub const ANCHOR_MIDDLE: P = (270.0, 200.0);
pub const ANCHOR_CYCLE: P = (271.0, 300.0);
pub const ANCHOR_G9: P = (271.0, 356.0);
pub const ANCHOR_G8: P = (123.0, 135.0);
pub const ANCHOR_G7: P = (105.0, 270.0);
pub const ANCHOR_SNIPER: P = (84.0, 335.0);
pub const ANCHOR_FORWARD: P = (54.0, 366.0);
pub const ANCHOR_BACK: P = (51.0, 440.0);

/// The smoothed outline, the palm cover and the strip, built once (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
    pub palm: Vec<Pos2>,
    pub strip: Vec<Pos2>,
    pub spur: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let strip = chaikin(&pos(STRIP), 3);
        // The palm cover: the seams and the channel's rear end, then back along the strip.
        let mut palm = pos(PALM_TOP);
        palm.extend(pos(&[n(433.0, 318.0)]));
        palm.extend(strip.iter().rev().copied());
        Shapes {
            outline: chaikin(&pos(OUTLINE), 1),
            palm,
            spur: chaikin(&pos(SPUR), 2),
            strip,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(30.0, 30.0), (0.0, 0.0));
        let (x, y) = n(446.0, 730.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.594).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let raw: [&[P]; 13] = [
            OUTLINE, LEFT_BTN, RIGHT_BTN, CHANNEL, CHAMFER, PALM_TOP, G8, G7, SNIPER, FORWARD,
            BACK, GRIP, STRIP,
        ];
        for (x, y) in raw.iter().flat_map(|p| p.iter().copied()) {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
        let s = shapes();
        for q in s
            .outline
            .iter()
            .chain(&s.palm)
            .chain(&s.strip)
            .chain(&s.spur)
        {
            assert!((-0.002..=1.002).contains(&q.x) && (-0.002..=1.002).contains(&q.y));
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let px = |poly: &[Pos2]| -> Vec<Pos2> {
            poly.iter()
                .map(|p| pos2(p.x * 416.0, p.y * 700.0))
                .collect()
        };
        let raw = |poly: &[P]| -> Vec<Pos2> { px(&pos(poly)) };
        let s = shapes();
        let mut all: Vec<(&str, Vec<Pos2>)> = vec![
            ("outline", px(&s.outline)),
            ("palm", px(&s.palm)),
            ("left", raw(LEFT_BTN)),
            ("right", raw(RIGHT_BTN)),
            ("channel", raw(CHANNEL)),
            ("g8", raw(G8)),
            ("g7", raw(G7)),
            ("sniper", raw(SNIPER)),
            ("forward", raw(FORWARD)),
            ("back", raw(BACK)),
            ("grip", raw(GRIP)),
        ];
        all.retain(|(_, p)| p.len() >= 3);
        for (name, pts) in all {
            assert_eq!(
                crate::illustration::shapes::triangulate(&pts).len(),
                pts.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
