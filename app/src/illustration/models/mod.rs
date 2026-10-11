//! Model-accurate mouse artwork. Each mouse that has its own drawing implements [`ModelArt`]
//! and is listed in its brand's `models()`; [`lookup`] finds the drawing for a product name and
//! everything else (`Design`, `MouseArt`, the callouts) goes through the trait, so adding a model
//! never touches shared code. See `README.md` in this folder for how to add one.

pub mod corsair;
pub mod kit;
pub mod logitech;
pub mod razer;
pub mod steelseries;

use egui::{Painter, Rect};

use super::mouse::{Anatomy, MouseArt};
use super::roles::{MouseLook, Role};
use crate::theme::Palette;

/// One mouse (or family) with its own drawing.
pub trait ModelArt: Sync {
    /// Short stable id, e.g. `"g502"`.
    fn id(&self) -> &'static str;

    /// Does this drawing fit the product called `name`? `name` is lower-case; match on
    /// substrings (see [`contains_any`]).
    fn matches(&self, name: &str) -> bool;

    /// When several drawings match, the highest priority wins (ties go to the one registered
    /// first). Give a family a 0 and a variant that needs its own drawing a higher number.
    fn priority(&self) -> i32 {
        0
    }

    /// Product names this drawing is for: the demo and test devices, real names from the
    /// drivers. Each must resolve back to this model through [`lookup`] (tested).
    #[cfg_attr(not(test), allow(dead_code))] // read by the registry tests
    fn examples(&self) -> &'static [&'static str];

    /// Width divided by height of the art box.
    fn aspect(&self) -> f32;

    /// How far the art reaches above its box (a cable), as a fraction of the box height.
    fn overhang(&self) -> f32 {
        0.0
    }

    /// The parts of the drawing in `rect`: one per role the model places, each with a polygon
    /// (hover tint, hit test) and an anchor (callout marker), plus the wheel and logo zones.
    fn build(&self, rect: Rect) -> Anatomy;

    /// Paint the mouse into `art.rect`. `tint` lists parts to light up (hover / selection) with
    /// a strength 0-1; `look` carries the lighting zone colours and the selected zone.
    fn paint(
        &self,
        art: &MouseArt,
        p: &Painter,
        pal: &Palette,
        look: &MouseLook,
        tint: &[(Role, f32)],
    );
}

/// Does `name` contain any of `needles`? For [`ModelArt::matches`].
pub fn contains_any(name: &str, needles: &[&str]) -> bool {
    needles.iter().any(|s| name.contains(s))
}

/// Every registered drawing, brand by brand, in registration order.
pub fn all() -> impl Iterator<Item = &'static dyn ModelArt> {
    [
        logitech::models(),
        razer::models(),
        corsair::models(),
        steelseries::models(),
    ]
    .into_iter()
    .flatten()
    .copied()
}

/// The drawing for a product name: the highest-priority match, the earliest registered on a tie.
pub fn lookup(name: &str) -> Option<&'static dyn ModelArt> {
    best_match(all(), &name.to_ascii_lowercase())
}

fn best_match<'a>(
    models: impl Iterator<Item = &'a dyn ModelArt>,
    name: &str,
) -> Option<&'a dyn ModelArt> {
    let mut best: Option<&dyn ModelArt> = None;
    for m in models.filter(|m| m.matches(name)) {
        if best.is_none_or(|b| m.priority() > b.priority()) {
            best = Some(m);
        }
    }
    best
}

#[cfg(test)]
mod tests;
