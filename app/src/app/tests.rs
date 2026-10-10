//! Integration tests for the app shell: the real worker thread with demo devices.

use super::*;
use periph_core::model::{DpiStage, DpiState, Rgb};
use periph_core::profile::new_id;

const LOGITECH: &str = "demo:logitech:c095";

fn test_app() -> App {
    let path = std::env::temp_dir()
        .join(format!("periph-studio-app-{}", new_id("t")))
        .join("library.json");
    let ctx = Context::default();
    let mut app = App::with_context(
        &ctx,
        Options {
            screenshot_dir: None,
            library_path: Some(path),
            hardware: false,
        },
    );
    app.dispatch(Action::SetDemo(true), &ctx);
    wait(&mut app, |a| a.devices.len() >= 3);
    app
}

/// Pump worker events until `cond` holds (the worker is a real thread).
fn wait(app: &mut App, cond: impl Fn(&App) -> bool) {
    for _ in 0..300 {
        app.pump_worker();
        if cond(app) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("condition not reached in time");
}

fn view<'a>(app: &'a App, id: &str) -> &'a DeviceView {
    app.devices.iter().find(|d| d.desc.id == id).unwrap()
}

#[test]
fn demo_devices_arrive_and_hardware_sorts_first() {
    let app = test_app();
    assert_eq!(app.devices.len(), 3);
    assert!(app.devices.iter().all(|d| d.desc.simulated));
    assert!(app.selected.is_some());
}

#[test]
fn apply_is_optimistic_then_confirmed_by_the_device() {
    let mut app = test_app();
    app.apply(LOGITECH, Setting::ReportRate(500), false);
    assert_eq!(
        view(&app, LOGITECH).state.report_rate,
        Some(500),
        "UI updates before the worker answers"
    );
    assert_eq!(view(&app, LOGITECH).pending, 1);
    wait(&mut app, |a| view(a, LOGITECH).pending == 0);

    // Scramble the local copy, then re-read: the device really holds 500.
    app.view_mut(LOGITECH).unwrap().state.report_rate = Some(125);
    app.worker.send(Command::Refresh {
        id: LOGITECH.into(),
    });
    wait(&mut app, |a| {
        view(a, LOGITECH).state.report_rate == Some(500)
    });
}

#[test]
fn rejected_setting_rolls_back_and_toasts() {
    let mut app = test_app();
    app.apply(LOGITECH, Setting::ReportRate(8000), false); // the demo mouse tops out at 1000 Hz
    assert_eq!(view(&app, LOGITECH).state.report_rate, Some(8000));
    wait(&mut app, |a| {
        view(a, LOGITECH).pending == 0 && view(a, LOGITECH).state.report_rate == Some(1000)
    });
    assert!(
        app.toasts
            .texts()
            .iter()
            .any(|(k, t)| *k == ToastKind::Error && t.contains("8000"))
    );
}

#[test]
fn slider_drags_are_debounced_into_one_command() {
    let mut app = test_app();
    for dpi in [1000, 1100, 1200] {
        let setting = Setting::Dpi(DpiState {
            stages: vec![DpiStage::uniform(dpi), DpiStage::uniform(2000)],
            active: 0,
        });
        app.apply(LOGITECH, setting, true);
    }
    assert_eq!(
        view(&app, LOGITECH).pending,
        0,
        "nothing is sent while the drag is in flight"
    );
    assert_eq!(
        view(&app, LOGITECH).state.dpi.as_ref().unwrap().stages[0].x,
        1200
    );
    std::thread::sleep(APPLY_DELAY + Duration::from_millis(30));
    app.flush_debounced(&Context::default());
    assert_eq!(
        view(&app, LOGITECH).pending,
        1,
        "one command for the whole drag"
    );
    wait(&mut app, |a| view(a, LOGITECH).pending == 0);
    app.view_mut(LOGITECH).unwrap().state.dpi = None;
    app.worker.send(Command::Refresh {
        id: LOGITECH.into(),
    });
    wait(&mut app, |a| {
        view(a, LOGITECH)
            .state
            .dpi
            .as_ref()
            .is_some_and(|d| d.stages[0].x == 1200 && d.stages.len() == 2)
    });
}

#[test]
fn state_replies_wait_while_commands_are_in_flight() {
    let mut app = test_app();
    let mut stale = view(&app, LOGITECH).state.clone();
    stale.report_rate = Some(125);
    app.view_mut(LOGITECH).unwrap().pending = 1;
    app.on_event(Event::State {
        id: LOGITECH.into(),
        state: stale,
    });
    assert_eq!(
        view(&app, LOGITECH).state.report_rate,
        Some(1000),
        "an older snapshot must not clobber optimistic state"
    );
    assert!(view(&app, LOGITECH).stale);
    app.on_event(Event::Applied {
        id: LOGITECH.into(),
        error: None,
        state: None,
    });
    assert_eq!(view(&app, LOGITECH).pending, 0);
    assert!(
        !view(&app, LOGITECH).stale,
        "a refresh was requested once idle"
    );
}

#[test]
fn hiding_demo_devices_removes_them() {
    let mut app = test_app();
    app.dispatch(Action::SetDemo(false), &Context::default());
    wait(&mut app, |a| a.devices.is_empty());
    assert!(!app.lib.prefs.demo_devices);
    assert_eq!(app.current_device(), None);
    app.dispatch(Action::SetDemo(true), &Context::default());
    wait(&mut app, |a| a.devices.len() == 3);
}

#[test]
fn edits_follow_the_active_profile_and_profiles_swap_settings() {
    let mut app = test_app();
    let ctx = Context::default();
    app.apply(LOGITECH, Setting::ReportRate(250), false);
    wait(&mut app, |a| view(a, LOGITECH).pending == 0);
    let first = app.lib.active_profile.clone().unwrap();
    let recorded = &app.lib.profile(&first).unwrap().devices["logitech:c095"];
    assert_eq!(
        recorded,
        &vec![Setting::ReportRate(250)],
        "edits are recorded, one entry per setting"
    );

    let mut other = periph_core::profile::Profile::new("Other");
    other
        .devices
        .insert("logitech:c095".into(), vec![Setting::ReportRate(500)]);
    let other_id = other.id.clone();
    app.lib.profiles.push(other);
    app.dispatch(Action::ActivateProfile(other_id), &ctx);
    assert_eq!(view(&app, LOGITECH).state.report_rate, Some(500));
    wait(&mut app, |a| view(a, LOGITECH).pending == 0);

    app.dispatch(Action::ActivateProfile(first.clone()), &ctx);
    assert_eq!(view(&app, LOGITECH).state.report_rate, Some(250));
    wait(&mut app, |a| view(a, LOGITECH).pending == 0);
    assert_eq!(
        app.lib.profile(&first).unwrap().devices["logitech:c095"],
        vec![Setting::ReportRate(250)]
    );
}

#[test]
fn reconnect_restores_only_what_differs() {
    let mut app = test_app();
    let active = app.lib.active_profile.clone().unwrap();
    // The logo zone already matches the device; only the rate differs.
    let logo = view(&app, LOGITECH).state.lighting[&0];
    app.lib.profile_mut(&active).unwrap().devices.insert(
        "logitech:c095".into(),
        vec![
            Setting::ReportRate(500),
            Setting::Lighting {
                zone: 0,
                state: logo,
            },
        ],
    );
    let id = LOGITECH.to_string();
    app.restore_profile(&id);
    assert_eq!(view(&app, LOGITECH).state.report_rate, Some(500));
    assert_eq!(view(&app, LOGITECH).pending, 1);
    wait(&mut app, |a| view(a, LOGITECH).pending == 0);
    // Already matching: nothing is sent.
    app.restore_profile(&id);
    assert_eq!(view(&app, LOGITECH).pending, 0);
}

#[test]
fn per_key_strokes_accumulate_until_flushed() {
    let mut app = test_app();
    let razer = "demo:razer:028d";
    let c = |r| Setting::PerKey([(0x04, Rgb::new(r, 0, 0))].into());
    app.apply(razer, c(10), true);
    app.apply(
        razer,
        Setting::PerKey([(0x05, Rgb::new(0, 20, 0))].into()),
        true,
    );
    assert_eq!(view(&app, razer).pending, 0);
    app.apply(razer, Setting::PerKey(Default::default()), false);
    assert_eq!(
        view(&app, razer).pending,
        1,
        "an empty update flushes the queued strokes"
    );
    wait(&mut app, |a| view(a, razer).pending == 0);
    app.view_mut(razer).unwrap().state.per_key.clear();
    app.worker.send(Command::Refresh { id: razer.into() });
    wait(&mut app, |a| {
        let k = &view(a, razer).state.per_key;
        k.get(&0x04) == Some(&Rgb::new(10, 0, 0)) && k.get(&0x05) == Some(&Rgb::new(0, 20, 0))
    });
}
