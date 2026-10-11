# Model-accurate mouse art

How to give a mouse its own drawing (developer notes, not user docs).

A mouse the app does not know is drawn with one of four generic silhouettes
(`illustration/mouse.rs`, `Kind`). A mouse with a **model** is drawn by its own
code and looks like the product. Models live here, one folder per brand
(`logitech/`, `razer/`, `corsair/`, `steelseries/`); when you add one you only
touch your brand's folder and your brand's file in `app/src/screenshot_brands/`
(demo devices and tour steps). Nothing shared is edited.

```
models/
  mod.rs           ModelArt trait, lookup(), all(), registry tests (shared; do not edit)
  kit.rs           Frame, Plastic ramps, facet, shaded_rect, shadow, tint, zone ring
  logitech/        mod.rs lists models(); g502.rs, mx_master.rs (+ *_shapes.rs) are examples
  razer/ corsair/ steelseries/    mod.rs with an empty models() until you fill it
```

Shared files (`kit.rs`, `roles.rs`, `callouts.rs`, `mouse.rs`, `screenshot.rs`,
`screenshot_demo.rs`) are meant to be enough as they are. If you really need something new there, say so instead of
editing it.

## 1. Reference photos (scratchpad only, never in the repo)

Download a top-down product photo (and a side one to understand the shell)
**for reference only** into the session scratchpad, e.g. `.../scratchpad/ref-img/<model>/`.
Never copy brand photos or logos into the repo; the art is an original vector
rendition.

```sh
UA='Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124 Safari/537.36'
curl -sL -A "$UA" -o page.html "<product page url>"
grep -o 'https://[^" ,)]*\.\(png\|jpg\)' page.html | sort -u | grep -i '<model>'
curl -sL -A "$UA" -o top.png "<image url>"        # prefer the "top view" gallery shot
```

View images with the Read tool. Product shots usually have an alpha channel,
so the silhouette is just the opaque pixels.

## 2. Measure in reference pixels

Work in the photo's pixels, like `logitech/g502_shapes.rs` and
`logitech/mx_master_shapes.rs`:

* Find the shell's bounding box: `ORIGIN` (top-left) and `SIZE`. `ASPECT = SIZE.0 / SIZE.1`.
* `pub const fn n(x, y) -> P` maps reference pixels to the 0-1 art box; polygons
  are stored through it. Rects and anchors may stay in raw pixels and go through
  `Frame::px` / `Frame::rect_px`.
* Sample the outline per row/column of the alpha mask (a few lines of PIL do it),
  keep 40-130 points clockwise and smooth them with `shapes::chaikin`. Keep each
  filled polygon under about 400 points: it is triangulated on every frame.
* Crop and enlarge regions with a labelled pixel grid to read off buttons, seams
  and wheel windows. Measure the dark seams and gaps too; they carry the look.
* Unit-test the geometry (see `*_shapes.rs`): the box maps to the unit square,
  every polygon stays in the box, facets triangulate cleanly.

## 3. Parts: one per role

`build(&self, rect) -> Anatomy` returns `parts`, the `wheel` rect and the `logo`
(centre, radius). Each `Part` has a `Role`, a polygon (hover tint and hit area),
an anchor (callout marker, must lie on the part) and an optional label side:

```rust
Part::new(Role::Left, f.pts(g::LEFT_BTN)).anchored(f.px(g::ANCHOR_LEFT))
Part::new(Role::ThumbWheel, poly).anchored(a).labelled(Side::Left)
```

Roles (`roles.rs`; `Role::of` maps a driver's button names and factory actions):

| Role | What | Driver names that map to it |
|---|---|---|
| `Left` `Right` `Middle` | primary buttons, wheel click | "Left click", "Wheel click" |
| `TiltLeft` `TiltRight` | wheel tilt / sideways scroll | "Scroll left", "Tilt right" |
| `Back` `Forward` | thumb buttons, left flank | "Back", "G4 · Back", "Left side rear" |
| `RightBack` `RightForward` | thumb buttons, right flank (ambidextrous) | "Right side rear", "Forward (right)" |
| `Sniper` | DPI shift / sniper / clutch | "Sniper", "DPI clutch", "G6 · DPI shift (1200)" |
| `DpiUp` `DpiDown` `DpiCycle` | DPI keys | "DPI up", "DPI stage -", "DPI button", "DPI switch" |
| `Profile` | profile cycling | "Profile cycle" |
| `WheelMode` | SmartShift / ratchet toggle, wheel unlock | "Smart shift", "Mode shift" |
| `ThumbWheel` | thumb wheel (a part, not a button) | "Thumb wheel" |
| `Gesture` | gesture button | "Gesture button" |
| `GKey(n)` | extra key printed `Gn` | "G12", "G12 · Default" |
| `Grid(i)` | thumb grid / plate cell, 0-based, row by row | "Side button 7"; unnamed buttons fill free cells in order |

Only draw roles the real mouse has: a role without a part falls back to the
driver's hotspot (or is left out), which is right. One part per role.

Side hints: leave `side` unset and the callouts pick from the role and the anchor
(left/right of the art's centre). Set it when a part is awkward (a thumb wheel,
a key on the centre line). The callouts never learn about models.

## 4. Painting

`paint(&self, art, p, pal, look, tint)` draws into `art.rect`; everything is the
egui painter, no textures.

* **Frame**: `Frame::new(rect, ORIGIN, SIZE)`; `f.pts` (normalised polygons),
  `f.px` (reference px), `f.s()` (screen px per reference px, for line widths and radii).
* **Colour**: take a `Plastic` ramp (`black`, `graphite`, `white`; each has dark and
  light-theme steps from `pal.dark`) rather than hard-coded colours, and `pal.accent`
  for UI. Metal and glass get their own colours.
* **Flat-shaded facets**: `kit::facet(p, pts, dir, from, to, hi, edge)`: a linear
  gradient plus a rim light on edges facing `LIGHT` (up-left). Paint back to front:
  shadow, base shell, lower facets, seams (dark polygon under a slightly inset lighter
  one), buttons, wheel, keys, logo; then an all-round faint rim and
  `edge_lights` on the outline. `kit::shadow` for the contact shadow.
* **Gotcha**: `shapes::fill_poly` colours only the *vertices*, so it reproduces linear
  gradients exactly but cannot show shading that peaks mid-shape. For wheels,
  domes and metal use `kit::shaded_rect` (grid mesh) or several facets.
* **Concave polygons**: never `Shape::convex_polygon`; use `fill_poly` / `kit::fill_flat`.
* **Lighting**: `MouseLook` carries `logo`, `wheel` and `body` zone colours. Glow a
  LED with `shapes::glow` / `glow_line` driven by those colours; leave them unlit
  (plain dark/grey) when `None`. Mice without LEDs ignore `look` except for
  `kit::zone_ring`, which draws the selection ring.
* **Hover/selection**: end with `kit::paint_tint(p, art, pal, tint)`.
* Keep the file under ~600 lines (geometry in `*_shapes.rs`), no brand logos
  (draw a neutral mark or leave the spot blank), and stay cheap per frame.

## 5. Register, demo, tour

1. `impl ModelArt` (skeleton below) and add `&your_model::YourModel` to
   `models()` in `<brand>/mod.rs`, most specific first.
2. `matches(name)` gets the **lower-cased** product name; use `contains_any`.
   `priority()` (default 0) breaks ties: give a variant that needs its own art
   a higher number than its family. `examples()` lists real product names
   (drivers, demos); the registry test requires each to resolve to your model.
3. Add a simulated device to `devices()` in `app/src/screenshot_brands/<brand>.rs`
   (copy `mx_master_3s()` in `logitech.rs`: `mouse_like(Spec { .. })` with the
   button names and ids the driver reports, its hotspots, and the zones the
   real mouse has) and tour steps to that file's `steps()`:
   `Step::device("NN-<model>-assignments", "<vendor>:<pid>", Tab::Assignments, |a| ...)`,
   with `.light()` for a light-theme variant. Step numbers: Logitech 3x, Razer 4x,
   Corsair 5x, SteelSeries 6x. The `<vendor>:<pid>` model key must be unique.
4. Add geometry tests in `<brand>/tests.rs` (`#[cfg(test)] mod tests;` in the
   brand's `mod.rs`): each control sits where the real one does, every demo
   button lands on a part (see `logitech/tests.rs`).

## 6. Checks

```sh
cargo fmt -p gearwise
cargo clippy -p gearwise --all-targets -- -D warnings
cargo test -p gearwise            # registry tests cover every model automatically
cargo check -p gearwise --target x86_64-pc-windows-gnu
cargo build --release -p gearwise && \
  target/release/gearwise --screenshot <dir> --only <step-name-substring> [--light] [--size 1280x800]
```

Compare the tour shot with the reference (crop the art and put both side by side),
in dark and light. GNOME pauses rendering of covered windows: if the tour stalls,
rerun. Do not run `--hw-screenshot` and do not open real devices. Keep
`app/screenshots` under 4.5 MB in total.

## Skeleton

```rust
// <brand>/foo_bar.rs
use egui::{Painter, Rect};

use super::foo_bar_shapes::{self as g, shapes};
use crate::illustration::models::kit::{self, Frame, Plastic};
use crate::illustration::models::{ModelArt, contains_any};
use crate::illustration::mouse::{Anatomy, MouseArt, Part};
use crate::illustration::roles::{MouseLook, Role};
use crate::theme::Palette;

pub struct FooBar;

impl ModelArt for FooBar {
    fn id(&self) -> &'static str { "foo-bar" }
    fn matches(&self, name: &str) -> bool { contains_any(name, &["foo bar"]) }
    fn examples(&self) -> &'static [&'static str] { &["Foo Bar Pro", "Foo Bar Pro (demo)"] }
    fn aspect(&self) -> f32 { g::ASPECT }
    // fn overhang(&self) -> f32 { 0.07 }   // cable above the box, fraction of height

    fn build(&self, rect: Rect) -> Anatomy {
        let f = Frame::new(rect, g::ORIGIN, g::SIZE);
        let part = |role, poly, anchor| Part::new(role, poly).anchored(f.px(anchor));
        Anatomy {
            parts: vec![
                part(Role::Left, f.pts(g::LEFT_BTN), g::ANCHOR_LEFT),
                part(Role::Right, f.pts(g::RIGHT_BTN), g::ANCHOR_RIGHT),
                // ...one part per control the real mouse has
            ],
            wheel: f.rect_px(g::WHEEL),
            logo: (f.px(g::LOGO), g::LOGO_RADIUS * f.s()),
        }
    }

    fn paint(&self, art: &MouseArt, p: &Painter, pal: &Palette, look: &MouseLook, tint: &[(Role, f32)]) {
        let f = Frame::new(art.rect, g::ORIGIN, g::SIZE);
        let sk = Plastic::black(pal);
        let outline = f.norm(&shapes().outline);
        kit::shadow(p, &outline, art.rect, pal);
        kit::facet(p, &outline, kit::DOWN, sk.mid, sk.low, sk.hi, 0.5);
        // ...facets back to front, wheel, keys, logo glow from `look`
        kit::paint_tint(p, art, pal, tint);
        kit::zone_ring(p, art, pal, look, |p, ring| { /* outline what "body" lights */ });
    }
}
```
