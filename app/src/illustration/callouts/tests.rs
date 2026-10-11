use super::*;

fn valid(ys: &[f32], gap: f32, lo: f32, hi: f32) {
    assert!(
        ys.windows(2).all(|w| w[1] - w[0] >= gap - 1e-3),
        "{ys:?} overlap"
    );
    assert!(
        ys.iter().all(|y| *y >= lo - 1e-3 && *y <= hi + 1e-3),
        "{ys:?} outside {lo}..{hi}"
    );
}

#[test]
fn spaced_labels_stay_where_they_want_to_be() {
    let ys = stack(&[50.0, 150.0, 300.0], 48.0, 20.0, 400.0);
    assert_eq!(ys, vec![50.0, 150.0, 300.0]);
}

#[test]
fn crowded_labels_are_pushed_apart_in_order() {
    let ys = stack(&[100.0, 104.0, 106.0, 110.0], 48.0, 20.0, 400.0);
    valid(&ys, 48.0, 20.0, 400.0);
    assert!(ys[0] <= 100.0 + 1e-3 || ys[0] >= 20.0);
    assert!(ys.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn labels_near_the_bottom_shift_up_instead_of_leaving_the_box() {
    let ys = stack(&[380.0, 390.0, 395.0], 48.0, 20.0, 400.0);
    valid(&ys, 48.0, 20.0, 400.0);
    assert_eq!(*ys.last().unwrap(), 400.0);
}

#[test]
fn too_many_labels_are_spread_evenly() {
    let ys = stack(&[10.0; 6], 48.0, 0.0, 100.0);
    assert_eq!(ys.len(), 6);
    assert!((ys[5] - 100.0).abs() < 1e-3 && ys[0].abs() < 1e-3);
    assert!(ys.windows(2).all(|w| (w[1] - w[0] - 20.0).abs() < 1e-3));
    assert!(stack(&[], 48.0, 0.0, 100.0).is_empty());
}

#[test]
fn labels_go_to_the_side_the_part_lives_on() {
    let art = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 300.0));
    assert_eq!(side_for(Some(Role::Back), pos2(10.0, 0.0), art), Side::Left);
    assert_eq!(
        side_for(Some(Role::Right), pos2(10.0, 0.0), art),
        Side::Right
    );
    assert_eq!(side_for(None, pos2(40.0, 0.0), art), Side::Left);
    assert_eq!(side_for(None, pos2(160.0, 0.0), art), Side::Right);
    // A DPI wedge far left of the centre line (G502) is labelled on the left.
    assert_eq!(
        side_for(Some(Role::DpiUp), pos2(40.0, 0.0), art),
        Side::Left
    );
    assert_eq!(
        side_for(Some(Role::DpiUp), pos2(98.0, 0.0), art),
        Side::Right
    );
}

fn item(id: u16, x: f32, y: f32, side: Side) -> Callout {
    Callout {
        id,
        anchor: pos2(x, y),
        side,
        title: format!("Button {id}"),
        detail: "Default".into(),
        custom: false,
        enabled: true,
        compact: false,
    }
}

fn untangled(laid: &[Laid], items: &[Callout]) -> bool {
    for (i, a) in laid.iter().enumerate() {
        for b in &laid[i + 1..] {
            if lines_cross(&a.line, &b.line) {
                return false;
            }
        }
        if laid
            .iter()
            .any(|o| o.item != a.item && runs_over(&a.line, items[o.item].anchor))
        {
            return false;
        }
    }
    true
}

/// The user's G502 HERO, laid out at several sizes: no leader crosses another or runs
/// over a foreign marker, and the pills never overlap or leave the area.
#[test]
fn real_g502_labels_stay_untangled_at_every_size() {
    use super::super::mouse::{Kind, MouseArt, art_rect};
    let hero = crate::screenshot_demo::extra_devices()
        .into_iter()
        .find(|d| d.descriptor.product_id == 0xc08b)
        .unwrap();
    for h in [330.0f32, 350.0, 380.0, 430.0, 460.0, 520.0, 580.0] {
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(900.0, h.max(height_needed(7))));
        let rect = art_rect(area.shrink2(vec2(250.0, 6.0)), Kind::G502);
        let art = MouseArt::new(rect, Kind::G502);
        let items: Vec<Callout> = art
            .place(&hero.capabilities.buttons)
            .iter()
            .map(|pl| Callout {
                side: side_for(pl.role, pl.anchor, rect),
                ..item(pl.id, pl.anchor.x, pl.anchor.y, Side::Left)
            })
            .collect();
        assert_eq!(items.len(), 11);
        let ys = (area.min.y + PILL_H / 2.0, area.max.y - PILL_H / 2.0);
        let mut all: Vec<Laid> = Vec::new();
        for (side, col, bend) in [
            (
                Side::Left,
                Rect::from_min_max(
                    pos2(rect.min.x - LEAD - 160.0, area.min.y),
                    pos2(rect.min.x - LEAD, area.max.y),
                ),
                rect.min.x - LEAD * 0.45,
            ),
            (
                Side::Right,
                Rect::from_min_max(
                    pos2(rect.max.x + LEAD, area.min.y),
                    pos2(rect.max.x + LEAD + 160.0, area.max.y),
                ),
                rect.max.x + LEAD * 0.45,
            ),
        ] {
            let idx: Vec<usize> = (0..items.len())
                .filter(|i| items[*i].side == side)
                .collect();
            let laid = layout_side(&items, &idx, side, col, bend, ys, &[]);
            assert!(
                untangled(&laid, &items),
                "h={h} {side:?}: {:?}",
                laid.iter()
                    .map(|l| (items[l.item].id, l.line))
                    .collect::<Vec<_>>()
            );
            let mut pills: Vec<Rect> = laid.iter().map(|l| l.pill).collect();
            pills.sort_by(|a, b| a.min.y.total_cmp(&b.min.y));
            assert!(pills.windows(2).all(|w| w[0].max.y <= w[1].min.y + 1e-3));
            assert!(
                pills
                    .iter()
                    .all(|p| p.min.y >= area.min.y - 1e-3 && p.max.y <= area.max.y + 1e-3)
            );
            all.extend(laid);
        }
        // Left and right leaders never cross either.
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert!(!lines_cross(&a.line, &b.line), "h={h}");
            }
        }
    }
}

#[test]
fn crossing_leaders_get_their_slots_swapped() {
    // Two markers whose natural order would make the lines cross.
    let items = vec![
        item(0, 100.0, 50.0, Side::Left),
        item(1, 20.0, 52.0, Side::Left),
    ];
    let col = Rect::from_min_max(pos2(-200.0, 0.0), pos2(-60.0, 200.0));
    let laid = layout_side(&items, &[0, 1], Side::Left, col, -20.0, (20.0, 180.0), &[]);
    assert!(!lines_cross(&laid[0].line, &laid[1].line));
}

#[test]
fn segment_tests_behave() {
    let (a, b) = (pos2(0.0, 0.0), pos2(10.0, 10.0));
    assert!(segments_cross(a, b, pos2(0.0, 10.0), pos2(10.0, 0.0)));
    assert!(!segments_cross(a, b, pos2(0.0, 10.0), pos2(4.0, 6.0)));
    assert!((dist_to_segment(pos2(5.0, 0.0), a, b) - 3.5355).abs() < 1e-3);
}

#[test]
fn pill_columns_fit_the_longest_label() {
    let ctx = egui::Context::default();
    let mut widths = (0.0, 0.0);
    let mut out = ctx.run_ui(Default::default(), |ui| {
        let mut a = item(1, 0.0, 0.0, Side::Left);
        a.title = "G6 · DPI shift (1200)".into();
        a.detail = "Macro: Reload and inspect".into();
        let b = item(2, 0.0, 0.0, Side::Right);
        widths = column_widths(ui.painter(), &[a, b], 400.0);
    });
    out.textures_delta.clear();
    assert!(widths.0 > widths.1, "{widths:?}");
    assert_eq!(widths.1, MIN_PILL_W);
    assert!(widths.0 < 400.0);
}
