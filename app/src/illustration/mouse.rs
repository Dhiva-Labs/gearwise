//! Top-down mouse artwork. A product is drawn either by a model-accurate drawing from the
//! registry in `models/` or, failing that, by one of the generic silhouettes below (a layered
//! render: rim, domed shell, split buttons, wheel, flank buttons, logo LED). Either way the
//! result is a [`MouseArt`]: anatomical parts that buttons are placed on by role.

use std::f32::consts::PI;

use egui::{Pos2, Rect, pos2, vec2};
use gearwise_core::model::ButtonCaps;

use super::models::{self, ModelArt};
pub use super::roles::{MouseLook, Role, Side, ZoneRole};
use super::shapes::{centroid, rot_rrect, rounded_poly};

/// The generic silhouettes, used for mice that have no model-accurate drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Right-handed with a thumb rest and a sniper button (Basilisk, M65, G402...).
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

    pub(super) fn profile(self) -> Profile {
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
    /// Outline, for the hover tint.
    pub poly: Vec<Pos2>,
    /// Where the callout marker sits.
    pub anchor: Pos2,
    /// Margin this part's label prefers; `None` lets the callouts decide from the role and
    /// where the anchor is.
    pub side: Option<Side>,
}

impl Part {
    /// A part whose anchor is the centre of its outline.
    pub fn new(role: Role, poly: Vec<Pos2>) -> Self {
        let anchor = centroid(&poly);
        Self {
            role,
            poly,
            anchor,
            side: None,
        }
    }

    pub fn anchored(mut self, anchor: Pos2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Ask for this part's label to go in `side`'s margin.
    pub fn labelled(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }
}

/// Where a button's callout attaches.
pub struct Placement {
    pub id: u16,
    pub role: Option<Role>,
    pub anchor: Pos2,
    /// The part's preferred label side (see [`Part::side`]).
    pub side: Option<Side>,
}

/// What a drawing contributes beyond its looks: the hit areas and the places lighting refers to.
pub struct Anatomy {
    pub parts: Vec<Part>,
    /// The scroll wheel (lit by the wheel zone, ringed when that zone is selected).
    pub wheel: Rect,
    /// The logo mark: centre and radius.
    pub logo: (Pos2, f32),
}

/// What draws a mouse: a model-accurate drawing when one matches the product name, else a
/// generic silhouette.
#[derive(Clone, Copy)]
pub enum Design {
    Model(&'static dyn ModelArt),
    Generic(Kind),
}

impl std::fmt::Debug for Design {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Design::Model(m) => write!(f, "Model({})", m.id()),
            Design::Generic(k) => write!(f, "Generic({k:?})"),
        }
    }
}

impl Design {
    pub fn for_name(name: &str) -> Design {
        models::lookup(name).map_or_else(|| Design::Generic(Kind::for_name(name)), Design::Model)
    }

    /// Width divided by height of the artwork box.
    pub fn aspect(self) -> f32 {
        match self {
            Design::Model(m) => m.aspect(),
            Design::Generic(k) => k.aspect(),
        }
    }

    /// How far the artwork reaches above its box (a cable), as a fraction of the height.
    pub fn overhang(self) -> f32 {
        match self {
            Design::Model(m) => m.overhang(),
            Design::Generic(_) => 0.0,
        }
    }
}

pub struct MouseArt {
    pub rect: Rect,
    pub design: Design,
    pub parts: Vec<Part>,
    pub wheel: Rect,
    pub logo: (Pos2, f32),
}

/// Largest rect with the design's proportions that fits in `avail`, leaving room above it
/// for whatever the artwork draws outside its box (a cable).
pub fn art_rect(avail: Rect, design: Design) -> Rect {
    let (aspect, over) = (design.aspect(), design.overhang());
    let h = (avail.height() / (1.0 + over)).min(avail.width() / aspect);
    let size = vec2(h * aspect, h);
    let top = avail.center().y - h * (1.0 + over) / 2.0 + h * over;
    Rect::from_min_size(pos2(avail.center().x - size.x / 2.0, top), size)
}

impl MouseArt {
    pub fn new(rect: Rect, design: Design) -> Self {
        let Anatomy { parts, wheel, logo } = match design {
            Design::Model(m) => m.build(rect),
            Design::Generic(kind) => generic_anatomy(rect, kind),
        };
        Self {
            rect,
            design,
            parts,
            wheel,
            logo,
        }
    }

    pub fn part(&self, role: Role) -> Option<&Part> {
        self.parts.iter().find(|p| p.role == role)
    }

    /// Tie each button to a part by role, else to its driver hotspot; buttons with neither
    /// take the free cells of a thumb grid in order, if the drawing has one. Buttons that fit
    /// nowhere are omitted.
    pub fn place(&self, buttons: &[ButtonCaps]) -> Vec<Placement> {
        // Roles first, so a button named after a grid cell keeps it.
        let mut taken: Vec<Role> = Vec::new();
        let mut roles: Vec<Option<Role>> = buttons
            .iter()
            .map(|b| {
                let role = Role::of(b).filter(|r| self.part(*r).is_some() && !taken.contains(r))?;
                taken.push(role);
                Some(role)
            })
            .collect();
        let cells = self
            .parts
            .iter()
            .filter(|p| matches!(p.role, Role::Grid(_)))
            .count() as u8;
        let mut free = (0..cells).map(Role::Grid).filter(|r| !taken.contains(r));
        for (role, b) in roles.iter_mut().zip(buttons) {
            if role.is_none() && b.hotspot.is_none() {
                *role = free.next();
            }
        }
        buttons
            .iter()
            .zip(roles)
            .filter_map(|(b, role)| match (role, b.hotspot) {
                (Some(r), _) => {
                    let part = self.part(r);
                    Some(Placement {
                        id: b.id,
                        role,
                        anchor: part.map_or(self.rect.center(), |p| p.anchor),
                        side: part.and_then(|p| p.side),
                    })
                }
                (None, Some(h)) => Some(Placement {
                    id: b.id,
                    role: None,
                    anchor: super::at(self.rect, h),
                    side: None,
                }),
                (None, None) => None,
            })
            .collect()
    }
}

/// Parts of a generic silhouette in `rect`.
fn generic_anatomy(rect: Rect, kind: Kind) -> Anatomy {
    let geo = Geo::new(rect, kind.profile());
    let (w, h) = (rect.width(), rect.height());
    let mut parts = Vec::new();
    let mut add = |role, poly: Vec<Pos2>, anchor: Option<Pos2>| {
        let mut part = Part::new(role, poly);
        if let Some(a) = anchor {
            part = part.anchored(a);
        }
        parts.push(part);
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
        .map(|i| pc + vec2((i as f32 * PI / 8.0).cos(), (i as f32 * PI / 8.0).sin()) * (w * 0.03))
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

    Anatomy {
        parts,
        wheel,
        logo: (geo.pt(0.70, 0.5), w * 0.07),
    }
}

#[cfg(test)]
mod tests;
