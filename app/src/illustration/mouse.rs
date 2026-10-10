//! Top-down mouse artwork. A silhouette is chosen per device, drawn as a layered
//! product render (rim, domed shell, split buttons, wheel, flank buttons, logo LED),
//! and exposes anatomical parts so buttons can be placed by role.

use std::f32::consts::PI;

use egui::{Color32, Pos2, Rect, pos2, vec2};
use gearwise_core::model::{ButtonAction, ButtonCaps, DpiFunction, MouseButton};

use super::shapes::{centroid, rot_rrect, rounded_poly};

/// Which part of the device a lighting zone lights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneRole {
    Logo,
    Wheel,
    Body,
}

pub fn zone_role(name: &str) -> ZoneRole {
    let n = name.to_ascii_lowercase();
    if n.contains("logo") {
        ZoneRole::Logo
    } else if n.contains("scroll") || n.contains("wheel") {
        ZoneRole::Wheel
    } else {
        ZoneRole::Body
    }
}

/// Colours a lighting preview paints on the mouse, per role.
#[derive(Clone, Copy, Default)]
pub struct MouseLook {
    pub logo: Option<Color32>,
    pub wheel: Option<Color32>,
    pub body: Option<Color32>,
    pub highlight: Option<ZoneRole>,
}

/// What a button is, anatomically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Left,
    Right,
    Middle,
    TiltLeft,
    TiltRight,
    Back,
    Forward,
    Sniper,
    DpiUp,
    DpiDown,
    DpiCycle,
    Profile,
    Grid(u8),
}

impl Role {
    /// Derive the role from the button's factory action, then from its name.
    pub fn of(b: &ButtonCaps) -> Option<Role> {
        let by_action = match &b.default_action {
            ButtonAction::Mouse(MouseButton::Left) => Some(Role::Left),
            ButtonAction::Mouse(MouseButton::Right) => Some(Role::Right),
            ButtonAction::Mouse(MouseButton::Middle) => Some(Role::Middle),
            ButtonAction::Mouse(MouseButton::Back) => Some(Role::Back),
            ButtonAction::Mouse(MouseButton::Forward) => Some(Role::Forward),
            ButtonAction::Mouse(MouseButton::WheelLeft) => Some(Role::TiltLeft),
            ButtonAction::Mouse(MouseButton::WheelRight) => Some(Role::TiltRight),
            ButtonAction::Dpi(DpiFunction::Up) => Some(Role::DpiUp),
            ButtonAction::Dpi(DpiFunction::Down) => Some(Role::DpiDown),
            ButtonAction::Dpi(DpiFunction::Cycle) => Some(Role::DpiCycle),
            ButtonAction::Dpi(DpiFunction::Shift(_)) => Some(Role::Sniper),
            ButtonAction::ProfileCycle => Some(Role::Profile),
            _ => None,
        };
        by_action.or_else(|| {
            let n = b.name.to_ascii_lowercase();
            let has = |s: &str| n.contains(s);
            Some(
                if has("scroll left") || has("tilt left") || has("wheel left") {
                    Role::TiltLeft
                } else if has("scroll right") || has("tilt right") || has("wheel right") {
                    Role::TiltRight
                } else if has("sniper") || has("dpi shift") {
                    Role::Sniper
                } else if has("dpi up") {
                    Role::DpiUp
                } else if has("dpi down") {
                    Role::DpiDown
                } else if has("dpi") {
                    Role::DpiCycle
                } else if has("profile") {
                    Role::Profile
                } else if has("middle") {
                    Role::Middle
                } else if has("left click") {
                    Role::Left
                } else if has("right click") {
                    Role::Right
                } else if n.ends_with("back") {
                    Role::Back
                } else if n.ends_with("forward") {
                    Role::Forward
                } else {
                    return None;
                },
            )
        })
    }
}

/// The silhouettes the artwork can draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Right-handed with a thumb rest and a sniper button (G502, Basilisk, M65...).
    Ergonomic,
    /// Mildly ergonomic (DeathAdder) and the fallback for unknown mice.
    ErgoLite,
    /// Symmetric lightweight shell (G Pro, Superlight, Viper...).
    Ambidextrous,
    /// Ergonomic shell with a 12-button thumb grid (Naga, Scimitar).
    Mmo,
}

impl Kind {
    /// Pick a silhouette from the product name.
    pub fn for_name(name: &str) -> Kind {
        let n = name.to_ascii_lowercase();
        let any = |list: &[&str]| list.iter().any(|s| n.contains(s));
        if any(&["naga", "scimitar", "mmo", "g600", "g604"]) {
            Kind::Mmo
        } else if any(&[
            "g502",
            "basilisk",
            "m65",
            "dark core",
            "darkcore",
            "g402",
            "ironclaw",
            "nightsword",
            "g700",
            "g500",
        ]) {
            Kind::Ergonomic
        } else if any(&[
            "g pro",
            "gpro",
            "g305",
            "g304",
            "superlight",
            "viper",
            "g102",
            "g203",
            "g403",
            "harpoon",
            "sabre",
            "katar",
            "glaive",
            "lightweight",
        ]) {
            Kind::Ambidextrous
        } else {
            Kind::ErgoLite
        }
    }

    /// Width divided by height of the artwork box.
    pub fn aspect(self) -> f32 {
        match self {
            Kind::Ergonomic => 0.66,
            Kind::ErgoLite => 0.62,
            Kind::Ambidextrous => 0.56,
            Kind::Mmo => 0.70,
        }
    }

    fn profile(self) -> Profile {
        match self {
            Kind::Ergonomic => Profile {
                a: 0.46,
                p: 2.4,
                k: 1.12,
                bulge: 0.13,
                notch: 0.05,
                rshrink: 0.07,
                waist: 0.0,
            },
            Kind::ErgoLite => Profile {
                a: 0.47,
                p: 2.45,
                k: 1.15,
                bulge: 0.06,
                notch: 0.02,
                rshrink: 0.04,
                waist: 0.02,
            },
            Kind::Ambidextrous => Profile {
                a: 0.5,
                p: 2.6,
                k: 1.2,
                bulge: 0.0,
                notch: 0.0,
                rshrink: 0.0,
                waist: 0.09,
            },
            Kind::Mmo => Profile {
                a: 0.45,
                p: 2.4,
                k: 1.1,
                bulge: 0.2,
                notch: 0.03,
                rshrink: 0.06,
                waist: 0.0,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Profile {
    a: f32,
    p: f32,
    k: f32,
    bulge: f32,
    notch: f32,
    rshrink: f32,
    waist: f32,
}

/// Depth of the rear edge of the primary buttons at across-fraction `f`: it bows forward in the middle.
pub(super) fn seam_t(f: f32) -> f32 {
    0.37 - 0.05 * (1.0 - (2.0 * f - 1.0).powi(2))
}

fn bump(t: f32, centre: f32, width: f32) -> f32 {
    (-((t - centre) / width).powi(2)).exp()
}

impl Profile {
    /// Left and right edge offsets from the centre line at depth `t` (0 nose, 1 tail).
    fn edges(&self, t: f32) -> (f32, f32) {
        let u = t.clamp(0.0, 1.0).powf(self.k);
        let base = self.a
            * (1.0 - (2.0 * u - 1.0).abs().powf(self.p))
                .max(0.0)
                .powf(1.0 / self.p);
        let waist = 1.0 - self.waist * bump(t, 0.40, 0.14);
        let left = base
            * (1.0 + self.bulge * bump(t, 0.60, 0.17) - self.notch * bump(t, 0.33, 0.07))
            * waist;
        let right = base * (1.0 - self.rshrink * bump(t, 0.50, 0.28)) * waist;
        (-left, right)
    }
}

/// Maps (depth, across) coordinates of the shell onto a screen rect.
#[derive(Clone, Copy)]
pub(super) struct Geo {
    pub(super) rect: Rect,
    profile: Profile,
    xmin: f32,
    xmax: f32,
}

impl Geo {
    pub(super) fn new(rect: Rect, profile: Profile) -> Self {
        let (mut xmin, mut xmax) = (f32::MAX, f32::MIN);
        for i in 0..=100 {
            let (l, r) = profile.edges(i as f32 / 100.0);
            xmin = xmin.min(l);
            xmax = xmax.max(r);
        }
        Self {
            rect,
            profile,
            xmin,
            xmax,
        }
    }

    fn x(&self, raw: f32) -> f32 {
        self.rect.min.x + (raw - self.xmin) / (self.xmax - self.xmin) * self.rect.width()
    }

    /// Point at depth `t` and across-fraction `f` (0 left edge, 1 right edge).
    pub(super) fn pt(&self, t: f32, f: f32) -> Pos2 {
        let (l, r) = self.profile.edges(t);
        pos2(
            self.x(l + (r - l) * f),
            self.rect.min.y + t * self.rect.height(),
        )
    }

    /// Depths 0..=end, denser near both ends so the nose and tail stay round.
    pub(super) fn depths(n: usize, end: f32) -> impl Iterator<Item = f32> {
        (0..=n).map(move |i| end * 0.5 * (1.0 - (PI * i as f32 / n as f32).cos()))
    }

    /// The same shape shrunk towards its centre by `px` on every side.
    pub(super) fn inset(&self, px: f32) -> Geo {
        Geo {
            rect: self.rect.shrink(px),
            ..*self
        }
    }

    pub(super) fn outline(&self) -> Vec<Pos2> {
        const N: usize = 72;
        let left: Vec<Pos2> = Self::depths(N, 1.0).map(|t| self.pt(t, 0.0)).collect();
        let right: Vec<Pos2> = Self::depths(N, 1.0).map(|t| self.pt(t, 1.0)).collect();
        left.into_iter()
            .chain(right.into_iter().rev().skip(1).take(N - 1))
            .collect()
    }

    /// Angle that rotates an upright rect to lie along the left flank at depth `t`.
    fn flank_angle(&self, t: f32) -> f32 {
        let d = self.pt(t + 0.02, 0.0) - self.pt(t - 0.02, 0.0);
        (-d.x).atan2(d.y)
    }
}

/// Pieces of the mouse a button can be tied to.
pub struct Part {
    pub role: Role,
    pub poly: Vec<Pos2>,
    pub anchor: Pos2,
}

/// Where a button's callout attaches.
pub struct Placement {
    pub id: u16,
    pub role: Option<Role>,
    pub anchor: Pos2,
}

pub struct MouseArt {
    pub rect: Rect,
    pub kind: Kind,
    pub(super) geo: Geo,
    pub(super) parts: Vec<Part>,
    pub(super) wheel: Rect,
    pub(super) logo: (Pos2, f32),
}

/// Largest rect with the silhouette's proportions that fits in `avail`.
pub fn art_rect(avail: Rect, kind: Kind) -> Rect {
    let h = avail.height().min(avail.width() / kind.aspect());
    Rect::from_center_size(avail.center(), vec2(h * kind.aspect(), h))
}

impl MouseArt {
    pub fn new(rect: Rect, kind: Kind) -> Self {
        let geo = Geo::new(rect, kind.profile());
        let (w, h) = (rect.width(), rect.height());
        let mut parts = Vec::new();
        let mut add = |role, poly: Vec<Pos2>, anchor: Option<Pos2>| {
            let anchor = anchor.unwrap_or_else(|| centroid(&poly));
            parts.push(Part { role, poly, anchor });
        };

        // Primary buttons: front edge of the shell down to a curved rear seam.
        let gap = 0.014;
        for (role, lo, hi) in [(Role::Left, 0.0, 0.5 - gap), (Role::Right, 0.5 + gap, 1.0)] {
            let mut poly: Vec<Pos2> = Geo::depths(14, seam_t(if lo == 0.0 { 0.0 } else { 1.0 }))
                .map(|t| geo.pt(t, if lo == 0.0 { 0.0 } else { 1.0 }))
                .collect();
            let arc: Vec<Pos2> = (0..=12)
                .map(|i| lo + (hi - lo) * i as f32 / 12.0)
                .map(|f| geo.pt(seam_t(f), f))
                .collect();
            if lo == 0.0 {
                poly.extend(arc);
            } else {
                poly.extend(arc.into_iter().rev());
            }
            let inner = if lo == 0.0 { hi } else { lo };
            poly.push(geo.pt(0.006, inner));
            let anchor = geo.pt(0.17, if lo == 0.0 { 0.27 } else { 0.73 });
            add(role, poly, Some(anchor));
        }

        // Scroll wheel and its tilt halves.
        let wheel = Rect::from_center_size(geo.pt(0.165, 0.5), vec2(w * 0.095, h * 0.15));
        let wheel_poly = rounded_poly(wheel, wheel.width() / 2.0, 5);
        let left_half = Rect::from_min_max(wheel.min, pos2(wheel.center().x, wheel.max.y));
        let right_half = Rect::from_min_max(pos2(wheel.center().x, wheel.min.y), wheel.max);
        add(Role::Middle, wheel_poly, Some(wheel.center()));
        add(
            Role::TiltLeft,
            rounded_poly(left_half, 4.0, 3),
            Some(pos2(wheel.min.x - 1.0, wheel.center().y - h * 0.025)),
        );
        add(
            Role::TiltRight,
            rounded_poly(right_half, 4.0, 3),
            Some(pos2(wheel.max.x + 1.0, wheel.center().y + h * 0.025)),
        );

        // DPI rocker and profile button behind the wheel.
        let dpi = |t: f32| {
            rounded_poly(
                Rect::from_center_size(geo.pt(t, 0.5), vec2(w * 0.085, h * 0.04)),
                4.0,
                3,
            )
        };
        let (up, down) = (dpi(0.355), dpi(0.405));
        let merged = rounded_poly(
            Rect::from_center_size(geo.pt(0.38, 0.5), vec2(w * 0.085, h * 0.095)),
            5.0,
            3,
        );
        add(Role::DpiUp, up, None);
        add(Role::DpiDown, down, None);
        add(Role::DpiCycle, merged, None);
        let pc = geo.pt(0.475, 0.5);
        let circle: Vec<Pos2> = (0..16)
            .map(|i| {
                pc + vec2((i as f32 * PI / 8.0).cos(), (i as f32 * PI / 8.0).sin()) * (w * 0.03)
            })
            .collect();
        add(Role::Profile, circle, Some(pc));

        // Flank buttons: tangent to the left edge so they follow the shell's curve.
        let flank = |t: f32, f: f32, across: f32, along: f32| {
            let c = geo.pt(t, f);
            rot_rrect(c, w * across, h * along, geo.flank_angle(t), 4.0)
        };
        match kind {
            Kind::Ergonomic => {
                add(Role::Sniper, flank(0.265, 0.075, 0.085, 0.07), None);
                add(Role::Back, flank(0.395, 0.085, 0.07, 0.078), None);
                add(Role::Forward, flank(0.485, 0.09, 0.07, 0.078), None);
            }
            Kind::ErgoLite => {
                add(Role::Back, flank(0.37, 0.08, 0.065, 0.075), None);
                add(Role::Forward, flank(0.455, 0.08, 0.065, 0.075), None);
            }
            Kind::Ambidextrous => {
                add(Role::Back, flank(0.35, 0.075, 0.06, 0.07), None);
                add(Role::Forward, flank(0.43, 0.075, 0.06, 0.07), None);
            }
            Kind::Mmo => {
                add(Role::Back, flank(0.25, 0.075, 0.06, 0.05), None);
                add(Role::Forward, flank(0.31, 0.08, 0.06, 0.05), None);
                for i in 0..12u8 {
                    let (row, col) = (f32::from(i / 3), f32::from(i % 3));
                    add(
                        Role::Grid(i),
                        flank(0.385 + row * 0.082, 0.065 + col * 0.075, 0.052, 0.054),
                        None,
                    );
                }
            }
        }

        let logo = (geo.pt(0.70, 0.5), w * 0.07);
        Self {
            rect,
            kind,
            geo,
            parts,
            wheel,
            logo,
        }
    }

    pub fn part(&self, role: Role) -> Option<&Part> {
        self.parts.iter().find(|p| p.role == role)
    }

    /// Tie each button to a part by role, else to its driver hotspot; MMO grids
    /// take the remaining buttons in order. Buttons that fit nowhere are omitted.
    pub fn place(&self, buttons: &[ButtonCaps]) -> Vec<Placement> {
        let mut taken: Vec<Role> = Vec::new();
        let mut out = Vec::new();
        let mut next_cell = 0u8;
        for b in buttons {
            let role = Role::of(b).filter(|r| self.part(*r).is_some() && !taken.contains(r));
            let role = role.or_else(|| {
                if self.kind == Kind::Mmo && b.hotspot.is_none() && next_cell < 12 {
                    next_cell += 1;
                    Some(Role::Grid(next_cell - 1))
                } else {
                    None
                }
            });
            match (role, b.hotspot) {
                (Some(r), _) => {
                    taken.push(r);
                    let anchor = self.part(r).map_or(self.rect.center(), |p| p.anchor);
                    out.push(Placement {
                        id: b.id,
                        role: Some(r),
                        anchor,
                    });
                }
                (None, Some(h)) => out.push(Placement {
                    id: b.id,
                    role: None,
                    anchor: super::at(self.rect, h),
                }),
                (None, None) => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gearwise_core::model::ButtonAction;

    fn caps(id: u16, name: &str, default: ButtonAction) -> ButtonCaps {
        ButtonCaps {
            id,
            name: name.into(),
            remappable: true,
            default_action: default,
            hotspot: None,
        }
    }

    #[test]
    fn silhouettes_follow_the_product_name() {
        assert_eq!(Kind::for_name("G502 X Plus (demo)"), Kind::Ergonomic);
        assert_eq!(Kind::for_name("Corsair M65 RGB Ultra"), Kind::Ergonomic);
        assert_eq!(Kind::for_name("Razer Basilisk V3"), Kind::Ergonomic);
        assert_eq!(Kind::for_name("Corsair Dark Core RGB"), Kind::Ergonomic);
        assert_eq!(Kind::for_name("G Pro X Superlight"), Kind::Ambidextrous);
        assert_eq!(Kind::for_name("Razer Viper Ultimate"), Kind::Ambidextrous);
        assert_eq!(Kind::for_name("Razer Naga Pro"), Kind::Mmo);
        assert_eq!(Kind::for_name("Corsair Scimitar Elite"), Kind::Mmo);
        assert_eq!(Kind::for_name("DeathAdder V3"), Kind::ErgoLite);
        assert_eq!(Kind::for_name("Some Unknown Mouse"), Kind::ErgoLite);
    }

    #[test]
    fn roles_come_from_the_factory_action_then_the_name() {
        assert_eq!(
            Role::of(&caps(
                1,
                "Left click",
                ButtonAction::Mouse(MouseButton::Left)
            )),
            Some(Role::Left)
        );
        assert_eq!(
            Role::of(&caps(
                2,
                "G6 · DPI shift (1200)",
                ButtonAction::Dpi(DpiFunction::Shift(1200))
            )),
            Some(Role::Sniper)
        );
        assert_eq!(
            Role::of(&caps(3, "G9 · Profile cycle", ButtonAction::ProfileCycle)),
            Some(Role::Profile)
        );
        assert_eq!(
            Role::of(&caps(4, "G11 · Scroll left", ButtonAction::Default)),
            Some(Role::TiltLeft)
        );
        assert_eq!(
            Role::of(&caps(5, "Back", ButtonAction::Default)),
            Some(Role::Back)
        );
        assert_eq!(Role::of(&caps(6, "Button 7", ButtonAction::Default)), None);
    }

    /// The G502 HERO as drivers report it: eleven buttons, five with hotspots.
    fn g502_hero() -> Vec<ButtonCaps> {
        let mouse = |b| ButtonAction::Mouse(b);
        let mut v = vec![
            caps(0, "Left click", mouse(MouseButton::Left)),
            caps(1, "Right click", mouse(MouseButton::Right)),
            caps(2, "Middle click", mouse(MouseButton::Middle)),
            caps(3, "Back", mouse(MouseButton::Back)),
            caps(4, "Forward", mouse(MouseButton::Forward)),
            caps(
                5,
                "G6 · DPI shift (1200)",
                ButtonAction::Dpi(DpiFunction::Shift(1200)),
            ),
            caps(6, "G7 · DPI down", ButtonAction::Dpi(DpiFunction::Down)),
            caps(7, "G8 · DPI up", ButtonAction::Dpi(DpiFunction::Up)),
            caps(8, "G9 · Profile cycle", ButtonAction::ProfileCycle),
            caps(9, "G10 · Scroll right", mouse(MouseButton::WheelRight)),
            caps(10, "G11 · Scroll left", mouse(MouseButton::WheelLeft)),
        ];
        for (b, h) in
            v.iter_mut()
                .zip([(0.3, 0.2), (0.7, 0.2), (0.5, 0.17), (0.1, 0.5), (0.11, 0.4)])
        {
            b.hotspot = Some(h);
        }
        v
    }

    #[test]
    fn every_g502_hero_button_gets_an_anatomical_anchor() {
        let art = MouseArt::new(
            Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 455.0)),
            Kind::Ergonomic,
        );
        let placed = art.place(&g502_hero());
        assert_eq!(placed.len(), 11);
        assert!(
            placed.iter().all(|p| p.role.is_some()),
            "roles beat raw hotspots"
        );
        assert!(
            placed
                .iter()
                .all(|p| art.rect.expand(2.0).contains(p.anchor))
        );
        // Back sits on the left flank, below the sniper button; the right button is right of centre.
        let at = |r: Role| placed.iter().find(|p| p.role == Some(r)).unwrap().anchor;
        assert!(at(Role::Back).x < art.rect.center().x && at(Role::Back).y > at(Role::Sniper).y);
        assert!(at(Role::Right).x > art.rect.center().x);
    }

    #[test]
    fn buttons_without_a_matching_part_fall_back_to_hotspots_or_the_grid() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
        // Sniper has no part on an ambidextrous mouse, so its hotspot is used.
        let mut sniper = caps(1, "Sniper", ButtonAction::Dpi(DpiFunction::Shift(400)));
        sniper.hotspot = Some((0.1, 0.3));
        let placed = MouseArt::new(rect, Kind::Ambidextrous).place(&[sniper.clone()]);
        assert_eq!(
            (placed[0].role, placed[0].anchor),
            (None, pos2(30.0, 129.0))
        );
        // Without a hotspot it is left out of the callouts.
        sniper.hotspot = None;
        assert!(
            MouseArt::new(rect, Kind::Ambidextrous)
                .place(&[sniper])
                .is_empty()
        );
        // MMO mice give unknown buttons a grid cell each, in order.
        let unknown: Vec<ButtonCaps> = (0..14)
            .map(|i| caps(i, &format!("Button {i}"), ButtonAction::Default))
            .collect();
        let placed = MouseArt::new(rect, Kind::Mmo).place(&unknown);
        assert_eq!(placed.len(), 12);
        assert_eq!(placed[0].role, Some(Role::Grid(0)));
        assert_eq!(placed[11].role, Some(Role::Grid(11)));
    }

    #[test]
    fn each_role_is_used_once() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 430.0));
        let twice = vec![
            caps(1, "Left click", ButtonAction::Mouse(MouseButton::Left)),
            caps(
                2,
                "Left click again",
                ButtonAction::Mouse(MouseButton::Left),
            ),
        ];
        assert_eq!(MouseArt::new(rect, Kind::ErgoLite).place(&twice).len(), 1);
    }

    #[test]
    fn parts_stay_inside_the_shell_box() {
        for kind in [
            Kind::Ergonomic,
            Kind::ErgoLite,
            Kind::Ambidextrous,
            Kind::Mmo,
        ] {
            let rect =
                Rect::from_min_size(pos2(50.0, 40.0), vec2(300.0 * kind.aspect() / 0.66, 455.0));
            let art = MouseArt::new(rect, kind);
            for part in &art.parts {
                assert!(
                    part.poly.iter().all(|q| rect.expand(3.0).contains(*q)),
                    "{kind:?} {:?} leaves the box",
                    part.role
                );
            }
            let outline = art.geo.outline();
            let (min, max) = outline
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q.x), b.max(q.x)));
            assert!(
                (min - rect.min.x).abs() < 1.0 && (max - rect.max.x).abs() < 1.0,
                "{kind:?} spans the full width"
            );
        }
    }

    #[test]
    fn ergonomic_shell_is_wider_on_the_thumb_side() {
        let p = Kind::Ergonomic.profile();
        let (l, r) = p.edges(0.6);
        assert!(-l > r * 1.1, "thumb rest bulges on the left");
        let (l, r) = Kind::Ambidextrous.profile().edges(0.6);
        assert!((-l - r).abs() < 1e-4, "ambidextrous is symmetric");
    }

    #[test]
    fn zone_names_map_to_roles() {
        assert_eq!(zone_role("Logo"), ZoneRole::Logo);
        assert_eq!(zone_role("Scroll wheel"), ZoneRole::Wheel);
        assert_eq!(zone_role("Primary"), ZoneRole::Body);
        assert_eq!(zone_role("Underglow"), ZoneRole::Body);
    }
}
