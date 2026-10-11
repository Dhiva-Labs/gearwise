//! Outline and part geometry of the G502-style mouse, in a 0-1 box (x across, y front to
//! back). The numbers were measured off a top-down product shot so proportions and part
//! layout match the real shell; everything is redrawn as original vector art.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use super::shapes::chaikin;

pub type P = (f32, f32);

/// Map a point of the 353 x 609 reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - 271.0) / 353.0, (y - 95.0) / 609.0)
}

/// Width divided by height of the shell (the cable sticks out above it).
pub const ASPECT: f32 = 353.0 / 609.0;
/// How far the cable reaches above the shell, as a fraction of its height.
pub const CABLE: f32 = 0.075;

// --- Silhouette, clockwise from the left nose tip, as runs that are rounded or kept sharp ---

/// The split nose: left tip, down the channel, across, up to the right tip.
pub const NOSE: &[P] = &[
    n(443.0, 97.0),
    n(446.0, 141.0),
    n(499.0, 141.0),
    n(497.0, 95.0),
];
/// Sharp bevel from the right tip to the shoulder.
pub const SHOULDER: &[P] = &[n(497.0, 95.0), n(576.0, 148.0)];
/// Right flank, tail corner (smoothed).
pub const RIGHT: &[P] = &[
    n(576.0, 148.0),
    n(592.0, 165.0),
    n(604.0, 187.0),
    n(613.0, 215.0),
    n(619.0, 250.0),
    n(623.0, 290.0),
    n(624.0, 340.0),
    n(624.0, 420.0),
    n(624.0, 500.0),
    n(624.0, 545.0),
    n(620.0, 575.0),
    n(612.0, 606.0),
    n(598.0, 640.0),
    n(578.0, 668.0),
    n(552.0, 687.0),
    n(520.0, 699.0),
    n(497.0, 704.0),
];
/// The notch where the tail groove meets the table.
pub const TAIL: &[P] = &[
    n(497.0, 704.0),
    n(491.0, 703.0),
    n(489.0, 697.0),
    n(461.0, 697.0),
    n(458.0, 703.0),
    n(452.0, 702.0),
];
/// Left flank: under the thumb rest and up its flared side (smoothed).
pub const LEFT: &[P] = &[
    n(452.0, 702.0),
    n(420.0, 692.0),
    n(398.0, 681.0),
    n(380.0, 664.0),
    n(368.0, 642.0),
    n(355.0, 618.0),
    n(342.0, 596.0),
    n(328.0, 575.0),
    n(313.0, 556.0),
    n(298.0, 540.0),
    n(284.0, 522.0),
    n(274.0, 502.0),
    n(271.0, 480.0),
    n(271.0, 440.0),
    n(274.0, 404.0),
    n(281.0, 378.0),
    n(291.0, 362.0),
    n(298.0, 352.0),
];
/// Sharp: the DPI-shift button's nose, the G7/G8 pair and the left button's front edge.
pub const FRONT_LEFT: &[P] = &[
    n(298.0, 352.0),
    n(304.0, 336.0),
    n(318.0, 322.0),
    n(322.0, 298.0),
    n(316.0, 262.0),
    n(315.0, 252.0),
    n(323.0, 232.0),
    n(340.0, 190.0),
    n(358.0, 144.0),
    n(365.0, 142.0),
];

// --- Facets, painted back to front ---

/// Left primary button: a long, narrow wedge from the nose down to the groove.
pub const LEFT_BTN: &[P] = &[
    n(443.0, 97.0),
    n(365.0, 142.0),
    n(361.0, 158.0),
    n(358.0, 300.0),
    n(354.0, 325.0),
    n(357.0, 436.0),
    n(455.0, 397.0),
    n(450.0, 300.0),
    n(446.0, 141.0),
];
pub const RIGHT_BTN: &[P] = &[
    n(497.0, 95.0),
    n(576.0, 148.0),
    n(588.0, 180.0),
    n(592.0, 260.0),
    n(594.0, 330.0),
    n(595.0, 397.0),
    n(505.0, 378.0),
    n(501.0, 300.0),
    n(499.0, 141.0),
];
/// Right grip wall: its inner edge (the outer edge follows the silhouette).
pub const WALL_INNER: &[P] = &[
    n(603.0, 502.0),
    n(595.0, 480.0),
    n(595.0, 397.0),
    n(594.0, 330.0),
    n(592.0, 260.0),
    n(588.0, 180.0),
];
/// Dark pocket where the grip wall meets the palm.
pub const POCKET: &[P] = &[
    n(595.0, 498.0),
    n(606.0, 502.0),
    n(620.0, 522.0),
    n(624.0, 546.0),
    n(612.0, 541.0),
    n(597.0, 522.0),
];
/// Centre channel that carries the wheel and the two small buttons.
pub const RAIL: &[P] = &[
    n(446.0, 141.0),
    n(499.0, 141.0),
    n(501.0, 300.0),
    n(505.0, 378.0),
    n(503.0, 421.0),
    n(456.0, 421.0),
    n(455.0, 397.0),
    n(450.0, 300.0),
];
/// Dark diagonal gap between the primary buttons and the palm cover.
pub const GROOVE: &[P] = &[
    n(340.0, 449.0),
    n(354.0, 433.0),
    n(456.0, 397.0),
    n(503.0, 400.0),
    n(503.0, 421.0),
    n(456.0, 421.0),
    n(345.0, 458.0),
];
/// Palm cover (right of the swoosh, with the logo): its top edge, then (after the
/// silhouette's lower right) the tail groove side and the swoosh.
pub const PALM_HEAD: &[P] = &[
    n(344.0, 456.0),
    n(456.0, 421.0),
    n(503.0, 421.0),
    n(596.0, 420.0),
    n(596.0, 498.0),
    n(603.0, 502.0),
];
pub const PALM_TAIL: &[P] = &[
    n(492.0, 703.0),
    n(490.0, 650.0),
    n(470.0, 600.0),
    n(455.0, 588.0),
    n(420.0, 580.0),
    n(390.0, 562.0),
    n(362.0, 534.0),
    n(350.0, 515.0),
    n(343.0, 490.0),
];
/// Lighter wedge under the right button.
pub const PALM_WEDGE: &[P] = &[
    n(505.0, 380.0),
    n(596.0, 398.0),
    n(596.0, 421.0),
    n(503.0, 421.0),
];
/// Lower-left lobe (the underside of the thumb rest): the inner edge, after the silhouette's
/// left flank.
pub const LOBE_INNER: &[P] = &[
    n(457.0, 702.0),
    n(457.0, 650.0),
    n(468.0, 600.0),
    n(455.0, 588.0),
    n(420.0, 580.0),
    n(390.0, 562.0),
    n(362.0, 534.0),
    n(350.0, 515.0),
    n(343.0, 490.0),
    n(344.0, 456.0),
    n(338.0, 447.0),
    n(322.0, 426.0),
    n(316.0, 405.0),
    n(311.0, 380.0),
    n(306.0, 362.0),
];
/// Dark curved band that separates the lobe from the palm.
pub const SWOOSH: &[P] = &[
    n(345.0, 455.0),
    n(343.0, 490.0),
    n(350.0, 515.0),
    n(362.0, 534.0),
    n(390.0, 562.0),
    n(420.0, 580.0),
    n(455.0, 588.0),
    n(472.0, 599.0),
];
/// The rear groove (dark, with a lighter inner strip).
pub const TAIL_GROOVE: &[P] = &[
    n(457.0, 590.0),
    n(470.0, 598.0),
    n(492.0, 703.0),
    n(457.0, 703.0),
    n(456.0, 650.0),
];
pub const TAIL_STRIP: &[P] = &[
    n(462.0, 600.0),
    n(481.0, 603.0),
    n(486.0, 648.0),
    n(462.0, 650.0),
];

/// Slanted panel carrying the DPI indicator strip.
pub const LED_PANEL: &[P] = &[
    n(321.0, 326.0),
    n(352.0, 322.0),
    n(359.0, 436.0),
    n(341.0, 447.0),
];
/// The three DPI indicator segments (centres).
pub const LEDS: [P; 3] = [n(337.0, 376.0), n(342.0, 394.0), n(346.0, 415.0)];
/// Thumb buttons: G5 (front) and G4 (rear), long slivers along the left edge.
pub const G5: &[P] = &[
    n(312.0, 347.0),
    n(323.0, 343.0),
    n(331.0, 404.0),
    n(322.0, 414.0),
];
pub const G4: &[P] = &[
    n(318.0, 421.0),
    n(332.0, 418.0),
    n(346.0, 508.0),
    n(336.0, 513.0),
];
/// DPI-shift ("sniper") button at the front of the thumb rest.
pub const SNIPER: &[P] = &[
    n(318.0, 322.0),
    n(304.0, 336.0),
    n(298.0, 352.0),
    n(304.0, 371.0),
    n(319.0, 361.0),
];
/// G8 (front) and G7 (rear), the stacked wedges left of the left button.
pub const G8: &[P] = &[
    n(358.0, 144.0),
    n(359.0, 240.0),
    n(316.0, 251.0),
    n(323.0, 232.0),
    n(340.0, 190.0),
];
pub const G7: &[P] = &[
    n(316.0, 258.0),
    n(359.0, 244.0),
    n(359.0, 306.0),
    n(323.0, 298.0),
    n(316.0, 262.0),
];

// --- Centre channel parts (rect corners in reference px) ---

/// Wheel slot and wheel.
pub const WHEEL_SLOT: ((f32, f32), (f32, f32)) = ((451.0, 185.0), (501.0, 306.0));
pub const WHEEL: ((f32, f32), (f32, f32)) = ((461.0, 192.0), (492.0, 290.0));
/// Tilt arrow marks (centres).
pub const ARROW_L: (f32, f32) = (433.0, 240.0);
pub const ARROW_R: (f32, f32) = (520.0, 238.0);
/// DPI-cycle button and the G9 button behind the wheel.
pub const CYCLE_KEY: ((f32, f32), (f32, f32)) = ((461.0, 317.0), (493.0, 341.0));
pub const G9_KEY: ((f32, f32), (f32, f32)) = ((463.0, 361.0), (495.0, 397.0));
/// Palm logo centre and radius.
pub const LOGO: (f32, f32, f32) = (418.0, 490.0, 29.0);
/// Cable boot centre x.
pub const CABLE_X: f32 = 476.0;

// --- Where each button's callout attaches (reference px) ---

pub const ANCHOR_LEFT: (f32, f32) = (402.0, 168.0);
pub const ANCHOR_RIGHT: (f32, f32) = (546.0, 168.0);
pub const ANCHOR_MIDDLE: (f32, f32) = (476.0, 272.0);
pub const ANCHOR_G8: (f32, f32) = (337.0, 192.0);
pub const ANCHOR_G7: (f32, f32) = (337.0, 278.0);
pub const ANCHOR_CYCLE: (f32, f32) = (477.0, 329.0);
pub const ANCHOR_G9: (f32, f32) = (479.0, 379.0);
pub const ANCHOR_G5: (f32, f32) = (320.0, 376.0);
pub const ANCHOR_G4: (f32, f32) = (331.0, 468.0);
pub const ANCHOR_SNIPER: (f32, f32) = (306.0, 346.0);

/// The outline and the facets that share its curved runs, built once.
pub struct Shapes {
    pub outline: Vec<Pos2>,
    pub wall: Vec<Pos2>,
    pub palm: Vec<Pos2>,
    pub lobe: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

/// Points of `run` whose y lies in `lo..=hi` (reference px).
fn between(run: &[Pos2], lo: f32, hi: f32) -> Vec<Pos2> {
    let (lo, hi) = (n(0.0, lo).1, n(0.0, hi).1);
    run.iter()
        .filter(|p| p.y >= lo - 1e-4 && p.y <= hi + 1e-4)
        .copied()
        .collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let right = chaikin(&pos(RIGHT), 3);
        let left = chaikin(&pos(LEFT), 3);
        let mut outline = pos(NOSE);
        outline.extend(&pos(SHOULDER)[1..]);
        outline.extend(&right[1..]);
        outline.extend(&pos(TAIL)[1..]);
        outline.extend(&left[1..]);
        outline.extend(&pos(FRONT_LEFT)[1..]);

        let mut wall = between(&right, 148.0, 500.0);
        wall.extend(pos(WALL_INNER));

        let mut palm = pos(PALM_HEAD);
        palm.extend(between(&right, 503.0, 704.0));
        palm.extend(pos(PALM_TAIL));

        let mut lobe: Vec<Pos2> = left.iter().rev().copied().collect();
        lobe.extend(pos(LOBE_INNER));
        Shapes {
            outline,
            wall,
            palm,
            lobe,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(271.0, 95.0), (0.0, 0.0));
        let (x, y) = n(624.0, 704.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.58).abs() < 0.01);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let s = shapes();
        let raw: [&[P]; 17] = [
            LEFT_BTN,
            RIGHT_BTN,
            POCKET,
            RAIL,
            GROOVE,
            PALM_WEDGE,
            SWOOSH,
            TAIL_GROOVE,
            TAIL_STRIP,
            LED_PANEL,
            G5,
            G4,
            SNIPER,
            G8,
            G7,
            WALL_INNER,
            LOBE_INNER,
        ];
        let all = raw.iter().flat_map(|p| p.iter().copied()).chain(
            [&s.outline, &s.wall, &s.palm, &s.lobe]
                .into_iter()
                .flat_map(|v| v.iter().map(|p| (p.x, p.y))),
        );
        for (x, y) in all {
            assert!(
                (-0.002..=1.002).contains(&x) && (-0.002..=1.002).contains(&y),
                "({x}, {y})"
            );
        }
    }

    #[test]
    fn facets_are_well_formed_simple_polygons() {
        let s = shapes();
        for (name, poly) in [
            ("outline", &s.outline),
            ("wall", &s.wall),
            ("palm", &s.palm),
            ("lobe", &s.lobe),
        ] {
            // Work at pixel scale, like the painter does.
            let px: Vec<Pos2> = poly
                .iter()
                .map(|p| pos2(p.x * 400.0, p.y * 690.0))
                .collect();
            let tris = crate::illustration::shapes::triangulate(&px);
            assert_eq!(
                tris.len(),
                poly.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
