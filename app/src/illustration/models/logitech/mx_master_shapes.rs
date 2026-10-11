//! Outline and part geometry of the MX Master (3S) shell, measured off a top-down product shot
//! (558 x 828 px box, origin at the shell's top-left) and redrawn as original vector art.
//! Every `*_PX` and run below is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::{Pos2, pos2};

use crate::illustration::models::kit::P;
use crate::illustration::shapes::chaikin;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (323.0, 101.0);
pub const SIZE: P = (558.0, 828.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Silhouette, clockwise from the left flank above the thumb housing, in three runs ---

/// Front: up the left flank, over the rounded nose and down the right flank to the grip.
pub const FRONT: &[P] = &[
    n(486.0, 340.0),
    n(486.0, 300.0),
    n(487.0, 260.0),
    n(488.0, 220.0),
    n(489.0, 190.0),
    n(491.0, 160.0),
    n(497.0, 140.0),
    n(503.0, 129.0),
    n(510.0, 121.0),
    n(519.0, 113.0),
    n(534.0, 105.0),
    n(549.0, 101.0),
    n(570.0, 101.0),
    n(594.0, 104.0),
    n(618.0, 107.0),
    n(642.0, 111.0),
    n(666.0, 115.0),
    n(690.0, 120.0),
    n(714.0, 126.0),
    n(738.0, 133.0),
    n(762.0, 141.0),
    n(786.0, 149.0),
    n(810.0, 159.0),
    n(822.0, 168.0),
    n(832.0, 180.0),
    n(842.0, 200.0),
    n(848.0, 220.0),
    n(851.0, 240.0),
    n(853.0, 260.0),
    n(855.0, 280.0),
    n(856.0, 300.0),
    n(858.0, 340.0),
    n(860.0, 380.0),
    n(860.0, 430.0),
    n(862.0, 456.0),
    n(865.0, 476.0),
    n(869.0, 510.0),
    n(873.0, 540.0),
    n(877.0, 580.0),
    n(880.0, 620.0),
    n(881.0, 660.0),
];

/// Rear: the skirt's outer edge, round the grip's heel and back along the thumb side.
pub const REAR: &[P] = &[
    n(881.0, 660.0),
    n(880.0, 680.0),
    n(879.0, 700.0),
    n(878.0, 720.0),
    n(875.0, 740.0),
    n(871.0, 760.0),
    n(865.0, 780.0),
    n(857.0, 800.0),
    n(848.0, 820.0),
    n(835.0, 840.0),
    n(818.0, 860.0),
    n(796.0, 880.0),
    n(768.0, 900.0),
    n(724.0, 920.0),
    n(682.0, 929.0),
    n(646.0, 929.0),
    n(603.0, 920.0),
    n(552.0, 900.0),
    n(515.0, 880.0),
    n(482.0, 860.0),
    n(450.0, 840.0),
    n(419.0, 820.0),
    n(390.0, 800.0),
    n(365.0, 780.0),
    n(346.0, 760.0),
    n(333.0, 740.0),
    n(326.0, 722.0),
];

/// The thumb shelf: bulging out to the left, then sweeping back in to the flank.
pub const SHELF_EDGE: &[P] = &[
    n(326.0, 722.0),
    n(323.0, 700.0),
    n(323.0, 680.0),
    n(324.0, 660.0),
    n(325.0, 640.0),
    n(328.0, 620.0),
    n(330.0, 600.0),
    n(333.0, 580.0),
    n(336.0, 560.0),
    n(339.0, 540.0),
    n(343.0, 520.0),
    n(346.0, 500.0),
    n(350.0, 484.0),
    n(355.0, 468.0),
    n(360.0, 460.0),
    n(365.0, 452.0),
    n(373.0, 444.0),
    n(384.0, 436.0),
    n(400.0, 428.0),
    n(419.0, 420.0),
    n(434.0, 412.0),
    n(445.0, 404.0),
    n(452.0, 396.0),
    n(457.0, 388.0),
    n(460.0, 380.0),
    n(460.0, 372.0),
    n(462.0, 364.0),
    n(474.0, 356.0),
    n(484.0, 348.0),
    n(486.0, 340.0),
];

// --- Facets ---

/// The ridged thumb panel: from the shelf's rim in to the housing and sweeping down to the skirt.
pub const SHELF: &[P] = &[
    n(384.0, 436.0),
    n(430.0, 446.0),
    n(452.0, 452.0),
    n(450.0, 520.0),
    n(450.0, 580.0),
    n(456.0, 640.0),
    n(468.0, 680.0),
    n(490.0, 716.0),
    n(520.0, 756.0),
    n(552.0, 796.0),
    n(584.0, 838.0),
    n(604.0, 880.0),
    n(603.0, 920.0),
    n(552.0, 900.0),
    n(515.0, 880.0),
    n(482.0, 860.0),
    n(450.0, 840.0),
    n(419.0, 820.0),
    n(390.0, 800.0),
    n(365.0, 780.0),
    n(346.0, 760.0),
    n(333.0, 740.0),
    n(326.0, 722.0),
    n(323.0, 700.0),
    n(323.0, 680.0),
    n(324.0, 660.0),
    n(325.0, 640.0),
    n(328.0, 620.0),
    n(330.0, 600.0),
    n(333.0, 580.0),
    n(336.0, 560.0),
    n(339.0, 540.0),
    n(343.0, 520.0),
    n(346.0, 500.0),
    n(350.0, 484.0),
    n(355.0, 468.0),
    n(360.0, 460.0),
    n(365.0, 452.0),
    n(373.0, 444.0),
];

/// Left button: a long panel from the nose to the diagonal seam above the thumb shelf.
pub const LEFT_BTN: &[P] = &[
    n(488.0, 432.0),
    n(487.0, 380.0),
    n(487.0, 340.0),
    n(486.0, 300.0),
    n(487.0, 260.0),
    n(488.0, 220.0),
    n(489.0, 190.0),
    n(491.0, 160.0),
    n(497.0, 140.0),
    n(503.0, 129.0),
    n(510.0, 121.0),
    n(519.0, 113.0),
    n(534.0, 105.0),
    n(549.0, 101.0),
    n(570.0, 101.0),
    n(594.0, 104.0),
    n(618.0, 107.0),
    n(642.0, 111.0),
    n(664.0, 115.0),
    n(664.0, 184.0),
    n(630.0, 186.0),
    n(622.0, 194.0),
    n(621.0, 260.0),
    n(621.0, 394.0),
    n(614.0, 402.0),
    n(560.0, 414.0),
    n(520.0, 423.0),
];

/// Right button: the mirror image about the rail, but its rear seam slopes the other way.
pub const RIGHT_BTN: &[P] = &[
    n(668.0, 115.0),
    n(690.0, 120.0),
    n(714.0, 126.0),
    n(738.0, 133.0),
    n(762.0, 141.0),
    n(786.0, 149.0),
    n(810.0, 159.0),
    n(822.0, 168.0),
    n(830.0, 178.0),
    n(833.0, 190.0),
    n(833.0, 300.0),
    n(833.0, 380.0),
    n(833.0, 436.0),
    n(828.0, 449.0),
    n(815.0, 451.0),
    n(780.0, 438.0),
    n(740.0, 424.0),
    n(714.0, 411.0),
    n(713.0, 186.0),
    n(668.0, 184.0),
];

/// The rail between the buttons that carries the wheel window and the mode-shift button.
pub const RAIL: &[P] = &[
    n(621.0, 184.0),
    n(713.0, 184.0),
    n(714.0, 410.0),
    n(706.0, 470.0),
    n(628.0, 470.0),
    n(621.0, 410.0),
];

/// Rects (two corners) and points below are in reference pixels.
///
/// Wheel window, the two metal bands of the MagSpeed wheel and the hub between them.
pub const WINDOW: (P, P) = ((640.0, 205.0), (696.0, 333.0));
pub const WHEEL_L: (P, P) = ((643.0, 216.0), (661.0, 330.0));
pub const WHEEL_R: (P, P) = ((671.0, 216.0), (689.0, 330.0));
pub const HUB: (P, P) = ((661.0, 218.0), (671.0, 328.0));
/// The mode-shift (SmartShift) button behind the wheel.
pub const MODE_BTN: (P, P) = ((647.0, 397.0), (686.0, 456.0));

/// The thumb pocket: a long housing on the left flank holding the thumb wheel and, below it,
/// the Forward and Back buttons.
pub const HOUSING: (P, P) = ((452.0, 434.0), (497.0, 658.0));
pub const THUMB_UPPER: (P, P) = ((459.0, 442.0), (489.0, 490.0));
pub const THUMB_LOWER: (P, P) = ((459.0, 501.0), (489.0, 552.0));
/// Hover area of the whole thumb wheel.
pub const THUMB_WHEEL: (P, P) = ((457.0, 440.0), (491.0, 554.0));
pub const FORWARD: (P, P) = ((464.0, 560.0), (494.0, 603.0));
pub const BACK: (P, P) = ((464.0, 609.0), (494.0, 652.0));
/// The gesture button's engraved mark (centre) and the area it presses.
pub const GESTURE_MARK: P = (362.0, 587.0);
pub const GESTURE_AREA: (P, P) = ((342.0, 556.0), (382.0, 618.0));

/// Ridges on the thumb panel: (start, end, bow) with the bow in reference pixels.
pub const RIDGES: &[(P, P, f32)] = &[
    ((455.0, 648.0), (326.0, 732.0), 16.0),
    ((458.0, 660.0), (338.0, 752.0), 20.0),
    ((462.0, 672.0), (358.0, 774.0), 24.0),
    ((468.0, 684.0), (384.0, 797.0), 28.0),
    ((476.0, 696.0), (416.0, 819.0), 30.0),
    ((486.0, 708.0), (450.0, 840.0), 30.0),
    ((498.0, 724.0), (486.0, 861.0), 28.0),
];

// --- Where each part's callout attaches (reference px) ---

pub const ANCHOR_LEFT: (f32, f32) = (556.0, 262.0);
pub const ANCHOR_RIGHT: (f32, f32) = (776.0, 262.0);
pub const ANCHOR_MIDDLE: (f32, f32) = (666.0, 262.0);
pub const ANCHOR_MODE: (f32, f32) = (666.0, 426.0);
pub const ANCHOR_THUMB: (f32, f32) = (474.0, 497.0);
pub const ANCHOR_FORWARD: (f32, f32) = (479.0, 578.0);
pub const ANCHOR_BACK: (f32, f32) = (479.0, 634.0);
pub const ANCHOR_GESTURE: (f32, f32) = (362.0, 596.0);

/// Smoothed outline and the curved facets that share its runs, built once (0-1 art box).
pub struct Shapes {
    pub outline: Vec<Pos2>,
    /// The lighter band round the heel, tapering out at both ends.
    pub skirt: Vec<Pos2>,
    /// A narrow strip just inside the right flank that catches the light.
    pub rim: Vec<Pos2>,
    pub shelf: Vec<Pos2>,
}

fn pos(q: &[P]) -> Vec<Pos2> {
    q.iter().map(|p| pos2(p.0, p.1)).collect()
}

/// `run` pulled `px` (reference pixels) towards `centre`, `taper`ing to nothing at both ends.
fn inset(run: &[Pos2], centre: Pos2, px: f32, taper: f32) -> Vec<Pos2> {
    let last = (run.len() - 1) as f32;
    run.iter()
        .enumerate()
        .map(|(i, q)| {
            let t = i as f32 / last;
            let k = (t.min(1.0 - t) / taper).clamp(0.0, 1.0);
            let k = k * k * (3.0 - 2.0 * k);
            let (dx, dy) = ((centre.x - q.x) * SIZE.0, (centre.y - q.y) * SIZE.1);
            let len = dx.hypot(dy).max(1e-3);
            pos2(
                q.x + dx / len * px * k / SIZE.0,
                q.y + dy / len * px * k / SIZE.1,
            )
        })
        .collect()
}

pub fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let front = chaikin(&pos(FRONT), 1);
        let rear = chaikin(&pos(REAR), 2);
        let shelf = chaikin(&pos(SHELF_EDGE), 1);
        let mut outline = front.clone();
        outline.extend(&rear[1..]);
        outline.extend(&shelf[1..]);

        let centre = pos2(0.5, 0.62);
        let mut skirt = rear.clone();
        skirt.extend(inset(&rear, centre, 27.0, 0.2).into_iter().rev());

        // The right flank from the shoulder down to where the skirt takes over.
        let flank: Vec<Pos2> = front
            .iter()
            .filter(|q| q.y > n(0.0, 250.0).1 && q.x > 0.5)
            .copied()
            .collect();
        let mut rim = flank.clone();
        rim.extend(inset(&flank, centre, 15.0, 0.25).into_iter().rev());

        let mut shelf_poly = pos(SHELF);
        shelf_poly.dedup();
        Shapes {
            outline,
            skirt,
            rim,
            shelf: shelf_poly,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_box_maps_to_the_unit_square() {
        assert_eq!(n(323.0, 101.0), (0.0, 0.0));
        let (x, y) = n(881.0, 929.0);
        assert!((x - 1.0).abs() < 1e-5 && (y - 1.0).abs() < 1e-5);
        assert!((ASPECT - 0.674).abs() < 0.002);
    }

    #[test]
    fn every_polygon_stays_inside_the_art_box() {
        let s = shapes();
        let raw: [&[P]; 6] = [LEFT_BTN, RIGHT_BTN, RAIL, SHELF, FRONT, REAR];
        let all = raw.iter().flat_map(|p| p.iter().copied()).chain(
            [&s.outline, &s.skirt, &s.rim, &s.shelf]
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
            ("skirt", &s.skirt),
            ("rim", &s.rim),
            ("shelf", &s.shelf),
        ] {
            let px: Vec<Pos2> = poly
                .iter()
                .map(|p| pos2(p.x * 558.0, p.y * 828.0))
                .collect();
            let tris = crate::illustration::shapes::triangulate(&px);
            assert_eq!(
                tris.len(),
                poly.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
        for (name, poly) in [("left", LEFT_BTN), ("right", RIGHT_BTN), ("rail", RAIL)] {
            let px: Vec<Pos2> = poly
                .iter()
                .map(|p| pos2(p.0 * 558.0, p.1 * 828.0))
                .collect();
            assert_eq!(
                crate::illustration::shapes::triangulate(&px).len(),
                poly.len() - 2,
                "{name} does not triangulate cleanly"
            );
        }
    }
}
