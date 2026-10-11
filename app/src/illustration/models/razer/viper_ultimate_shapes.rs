//! Outline and part geometry of the Viper Ultimate / V2 Pro shell, measured off a top-down product shot (a 280 x 515 px
//! box, origin at the shell's top-left) and redrawn as original vector art. Every number below
//! is in those reference pixels; [`n`] maps them into the 0-1 art box.

use std::sync::OnceLock;

use egui::Pos2;

use super::shared;
use crate::illustration::models::kit::P;

/// Top-left corner and size of the shell in the reference photo (px).
pub const ORIGIN: P = (820.0, 124.0);
pub const SIZE: P = (280.0, 515.0);
/// Width divided by height of the shell.
pub const ASPECT: f32 = SIZE.0 / SIZE.1;

/// Map a point of the reference box (px) into the 0-1 art box.
pub const fn n(x: f32, y: f32) -> P {
    ((x - ORIGIN.0) / SIZE.0, (y - ORIGIN.1) / SIZE.1)
}

// --- Geometry: sharp front corners, then the smoothed body, clockwise from the left front ---

/// Sharp: up the left front corner, over the two button tips and the wheel cover, down the right front corner.
pub const FRONT: &[P] = &[
    n(830.0, 250.0),
    n(832.0, 215.0),
    n(835.0, 188.0),
    n(841.0, 166.0),
    n(852.0, 154.0),
    n(882.0, 147.0),
    n(920.0, 135.0),
    n(938.0, 125.0),
    n(939.0, 130.0),
    n(944.0, 131.0),
    n(976.0, 131.0),
    n(981.0, 130.0),
    n(982.0, 125.0),
    n(1000.0, 135.0),
    n(1038.0, 147.0),
    n(1068.0, 154.0),
    n(1079.0, 166.0),
    n(1085.0, 188.0),
    n(1088.0, 215.0),
    n(1090.0, 250.0),
];

/// Smoothed: the right flank, the round tail and back up the left flank.
pub const BODY: &[P] = &[
    n(1090.0, 250.0),
    n(1091.0, 272.0),
    n(1091.0, 330.0),
    n(1094.0, 390.0),
    n(1097.0, 430.0),
    n(1099.0, 465.0),
    n(1100.0, 502.0),
    n(1098.0, 530.0),
    n(1088.0, 562.0),
    n(1073.0, 585.0),
    n(1050.0, 610.0),
    n(1023.0, 627.0),
    n(993.0, 635.0),
    n(960.0, 638.0),
    n(927.0, 635.0),
    n(897.0, 627.0),
    n(870.0, 610.0),
    n(847.0, 585.0),
    n(832.0, 562.0),
    n(822.0, 530.0),
    n(820.0, 502.0),
    n(821.0, 465.0),
    n(823.0, 430.0),
    n(826.0, 390.0),
    n(829.0, 330.0),
    n(829.0, 272.0),
    n(830.0, 250.0),
];

/// The slit between the primary buttons, below the wheel channel.
pub const SPINE: &[P] = &[n(960.0, 250.0), n(960.0, 282.0)];

/// The V-shaped seam where the primary buttons meet the palm.
pub const CHEVRON: &[P] = &[
    n(829.0, 330.0),
    n(861.0, 369.0),
    n(910.0, 330.0),
    n(930.0, 289.0),
    n(942.0, 282.0),
    n(978.0, 282.0),
    n(990.0, 289.0),
    n(1010.0, 330.0),
    n(1059.0, 369.0),
    n(1091.0, 330.0),
];

/// Left primary button: from the nose to the V-shaped rear seam.
pub const LEFT_BTN: &[P] = &[
    n(938.0, 125.0),
    n(920.0, 135.0),
    n(882.0, 147.0),
    n(852.0, 154.0),
    n(841.0, 166.0),
    n(835.0, 188.0),
    n(833.0, 215.0),
    n(832.0, 250.0),
    n(831.0, 290.0),
    n(831.0, 326.0),
    n(860.0, 364.0),
    n(907.0, 328.0),
    n(927.0, 287.0),
    n(940.0, 280.0),
    n(958.0, 280.0),
    n(958.0, 248.0),
    n(940.0, 246.0),
    n(939.0, 130.0),
];

/// Right primary button.
pub const RIGHT_BTN: &[P] = &[
    n(982.0, 125.0),
    n(1000.0, 135.0),
    n(1038.0, 147.0),
    n(1068.0, 154.0),
    n(1079.0, 166.0),
    n(1085.0, 188.0),
    n(1087.0, 215.0),
    n(1088.0, 250.0),
    n(1089.0, 290.0),
    n(1089.0, 326.0),
    n(1060.0, 364.0),
    n(1013.0, 328.0),
    n(993.0, 287.0),
    n(980.0, 280.0),
    n(962.0, 280.0),
    n(962.0, 248.0),
    n(980.0, 246.0),
    n(981.0, 130.0),
];

/// Thumb buttons on the left flank (Forward in front, Back behind it) and the same pair on the right.
pub const FORWARD: &[P] = &[
    n(826.0, 268.0),
    n(836.0, 270.0),
    n(837.0, 336.0),
    n(829.0, 334.0),
];

pub const BACK: &[P] = &[
    n(828.0, 342.0),
    n(837.0, 344.0),
    n(838.0, 408.0),
    n(830.0, 406.0),
];

pub const RIGHT_FORWARD: &[P] = &[
    n(1094.0, 268.0),
    n(1084.0, 270.0),
    n(1083.0, 336.0),
    n(1091.0, 334.0),
];

pub const RIGHT_BACK: &[P] = &[
    n(1092.0, 342.0),
    n(1083.0, 344.0),
    n(1082.0, 408.0),
    n(1090.0, 406.0),
];

/// The wheel channel, the dark window the wheel turns in, the wheel and the cover (with the status LED) in front of it, as rect corners.
pub const CHANNEL: (P, P) = ((938.0, 124.0), (984.0, 252.0));

pub const WHEEL_SLOT: (P, P) = ((940.0, 131.0), (982.0, 250.0));

pub const WHEEL: (P, P) = ((945.0, 156.0), (975.0, 214.0));

pub const COVER: (P, P) = ((944.0, 131.0), (976.0, 156.0));

pub const LED: P = (960.0, 239.0);

pub const LOGO: P = (952.0, 543.0);

pub const ANCHOR_LEFT: P = (885.0, 215.0);

pub const ANCHOR_RIGHT: P = (1035.0, 215.0);

pub const ANCHOR_MIDDLE: P = (960.0, 184.0);

pub const ANCHOR_FORWARD: P = (832.0, 302.0);

pub const ANCHOR_BACK: P = (833.0, 375.0);

pub const ANCHOR_RIGHT_FORWARD: P = (1088.0, 302.0);

pub const ANCHOR_RIGHT_BACK: P = (1087.0, 375.0);

pub const LOGO_RADIUS: f32 = 28.0;

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
        let raw: [&[P]; 10] = [
            FRONT,
            BODY,
            SPINE,
            CHEVRON,
            LEFT_BTN,
            RIGHT_BTN,
            FORWARD,
            BACK,
            RIGHT_FORWARD,
            RIGHT_BACK,
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
            LED,
            LOGO,
            ANCHOR_LEFT,
            ANCHOR_RIGHT,
            ANCHOR_MIDDLE,
            ANCHOR_FORWARD,
            ANCHOR_BACK,
            ANCHOR_RIGHT_FORWARD,
            ANCHOR_RIGHT_BACK,
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
