//! `--screenshot <dir>`: walk through the main views with demo devices and write PNGs.

use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use egui::{Context, ViewportCommand};
use gearwise_core::device::Setting;
use gearwise_core::model::{ButtonAction, Modifiers, media};
use gearwise_core::profile::Theme;

use crate::app::App;
use crate::pages::lighting::LightMode;
use crate::png;
use crate::screenshot_demo::extra_devices;
pub use crate::screenshot_demo::sample_library;
use crate::state::{Action, Route, Tab, ToastKind};

const LOGITECH: &str = "logitech:c095";
const RAZER: &str = "razer:028d";
const CORSAIR: &str = "corsair:1b9e";
// Simulated extras for the silhouette close-ups (see `extra_devices`).
const HERO: &str = "logitech:c08b";
const SUPERLIGHT: &str = "logitech:c094";
const NAGA: &str = "razer:0067";
const DEATHADDER: &str = "razer:00b6";
const HEADSET: &str = "corsair:0a64";
const K55: &str = "corsair:1ba4";

/// Frames to let a view settle (layout height matching, animations) before the shot.
const SETTLE_FRAMES: u32 = 16;
const SETTLE_TIME: Duration = Duration::from_millis(500);

/// `--only a,b`: restrict the tour to steps whose name contains one of these.
static ONLY: OnceLock<Vec<String>> = OnceLock::new();
/// `--light`: run every step in the light theme.
static LIGHT: AtomicBool = AtomicBool::new(false);

pub fn force_light() {
    LIGHT.store(true, Ordering::Relaxed);
}

pub fn set_only(list: &str) {
    let _ = ONLY.set(
        list.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    );
}

pub(crate) struct Step {
    name: &'static str,
    route: Route,
    device: Option<&'static str>,
    tab: Tab,
    theme: Theme,
    prep: fn(&mut App),
}

impl Step {
    /// A device page (`model_key`, e.g. `"logitech:b034"`) at `tab`, in the dark theme.
    pub(crate) fn device(
        name: &'static str,
        model_key: &'static str,
        tab: Tab,
        prep: fn(&mut App),
    ) -> Self {
        Self {
            name,
            route: Route::Device,
            device: Some(model_key),
            tab,
            theme: Theme::Dark,
            prep,
        }
    }

    /// The same step in the light theme.
    pub(crate) fn light(self) -> Self {
        Self {
            theme: Theme::Light,
            ..self
        }
    }

    #[cfg(test)]
    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    /// Does this step's device appear among `devices`?
    #[cfg(test)]
    pub(crate) fn targets(&self, devices: &[crate::worker::DeviceSnapshot]) -> bool {
        self.device
            .is_none_or(|k| devices.iter().any(|d| d.descriptor.model_key == k))
    }
}

fn nothing(_: &mut App) {}

fn steps() -> Vec<Step> {
    let dev = |name, device, tab, prep| Step {
        name,
        route: Route::Device,
        device: Some(device),
        tab,
        theme: Theme::Dark,
        prep,
    };
    let page = |name, route| Step {
        name,
        route,
        device: None,
        tab: Tab::Info,
        theme: Theme::Dark,
        prep: nothing,
    };
    let light = |s: Step| Step {
        theme: Theme::Light,
        ..s
    };
    let mut all = vec![
        dev("01-g502-hero-assignments", HERO, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(5)
        }),
        dev("02-g502-hero-lighting", HERO, Tab::Lighting, |a| {
            a.pages.light.zone = 1
        }),
        dev("03-sensitivity", LOGITECH, Tab::Sensitivity, nothing),
        dev("04-k55-zone-lighting", K55, Tab::Lighting, |a| {
            a.pages.light.zone = 3
        }),
        dev("05-k55-all-zones", K55, Tab::Lighting, |a| {
            a.pages.light.zone = crate::pages::lighting::ALL_ZONES
        }),
        dev("06-lighting-per-key", RAZER, Tab::Lighting, |a| {
            a.pages.light.mode = LightMode::PerKey
        }),
        dev("07-lighting-keyboard", RAZER, Tab::Lighting, |a| {
            a.pages.light.mode = LightMode::Effects
        }),
        dev("08-assignments-keyboard", RAZER, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(0x101)
        }),
        dev("09-performance", CORSAIR, Tab::Performance, nothing),
        page("10-profiles", Route::Profiles),
        page("11-macros", Route::Macros),
        page("12-settings", Route::Settings),
        light(dev("13-light-assignments", HERO, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(8)
        })),
        light(dev("14-light-k55-lighting", K55, Tab::Lighting, |a| {
            a.pages.light.zone = 2
        })),
        light(dev(
            "15-light-lighting-mouse",
            LOGITECH,
            Tab::Lighting,
            |a| a.pages.light.zone = 1,
        )),
        dev(
            "16-g502-x-plus-assignments",
            LOGITECH,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(0x56),
        ),
        dev(
            "17-superlight-callouts",
            SUPERLIGHT,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(3),
        ),
        dev("18-naga-button-grid", NAGA, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(8)
        }),
        dev("19-deathadder-lighting", DEATHADDER, Tab::Lighting, nothing),
        dev("20-headset-lighting", HEADSET, Tab::Lighting, nothing),
    ];
    // The per-brand model steps come first: the next step leaves a banner and toasts behind
    // and the one after drops every device.
    all.extend(crate::screenshot_brands::steps());
    all.push(dev("21-info-permission-banner", LOGITECH, Tab::Info, |a| {
        a.show_sample_issue();
        a.toasts.push(ToastKind::Success, "Saved to G502 X Plus");
        a.toasts.push(
            ToastKind::Error,
            "Report rate: not supported by this device",
        );
    }));
    all.push(page("22-empty-state", Route::Device).with_prep(|a| {
        a.clear_issues();
        a.toasts.clear();
        a.queue_action(Action::SetDemo(false));
        a.clear_devices();
    }));
    match ONLY.get() {
        Some(only) if !only.is_empty() => all
            .into_iter()
            .filter(|s| only.iter().any(|o| s.name.contains(o.as_str())))
            .collect(),
        _ => all,
    }
}

impl Step {
    fn with_prep(mut self, prep: fn(&mut App)) -> Self {
        self.prep = prep;
        self
    }
}

pub struct Capture {
    dir: PathBuf,
    step: usize,
    frames: u32,
    entered: Instant,
    requested: bool,
    seeded: bool,
    prepped: bool,
}

impl Capture {
    pub fn new(dir: PathBuf) -> Self {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("cannot create {}: {e}", dir.display());
        }
        Self {
            dir,
            step: 0,
            frames: 0,
            entered: Instant::now(),
            requested: false,
            seeded: false,
            prepped: false,
        }
    }
}

/// Average `factor` x `factor` blocks into one RGB pixel.
fn to_rgb(image: &egui::ColorImage, factor: usize) -> (u32, u32, Vec<u8>) {
    let [w, h] = image.size;
    let (ow, oh) = (w / factor, h / factor);
    let mut out = Vec::with_capacity(ow * oh * 3);
    for y in 0..oh {
        for x in 0..ow {
            let mut sum = [0u32; 3];
            for dy in 0..factor {
                for dx in 0..factor {
                    let p = image.pixels[(y * factor + dy) * w + x * factor + dx];
                    sum[0] += u32::from(p.r());
                    sum[1] += u32::from(p.g());
                    sum[2] += u32::from(p.b());
                }
            }
            let n = (factor * factor) as u32;
            out.extend(sum.map(|s| (s / n) as u8));
        }
    }
    (ow as u32, oh as u32, out)
}

impl App {
    pub(crate) fn queue_action(&mut self, a: Action) {
        let ctx = egui::Context::default();
        self.dispatch(a, &ctx);
    }

    /// Advance the screenshot run; called every frame from `logic`.
    pub(crate) fn drive_capture(&mut self, ctx: &Context) {
        let Some(mut cap) = self.capture.take() else {
            return;
        };
        ctx.request_repaint();
        let all = steps();
        if cap.step >= all.len() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            self.capture = Some(cap);
            return;
        }
        // Wait for the worker to report the three demo devices, then seed some customisation.
        if !cap.seeded {
            if self.devices.len() >= 3 {
                self.seed();
                cap.seeded = true;
                self.enter(&all[0]);
                cap.entered = Instant::now();
            }
            self.capture = Some(cap);
            return;
        }
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        let step = &all[cap.step];
        if let Some(image) = shot {
            let factor = (image.size[0] as f32 / ctx.content_rect().width())
                .round()
                .max(1.0) as usize;
            let (w, h, rgb) = to_rgb(&image, factor);
            let path = cap.dir.join(format!("{}.png", step.name));
            match std::fs::write(&path, png::encode_rgb(w, h, &rgb)) {
                Ok(()) => println!("wrote {} ({w}x{h})", path.display()),
                Err(e) => eprintln!("cannot write {}: {e}", path.display()),
            }
            cap.step += 1;
            cap.frames = 0;
            cap.requested = false;
            cap.prepped = false;
            cap.entered = Instant::now();
            if let Some(next) = all.get(cap.step) {
                self.enter(next);
            }
        } else {
            cap.frames += 1;
            if cap.frames == 4 && !cap.prepped {
                cap.prepped = true;
                (step.prep)(self);
            }
            if !cap.requested && cap.frames >= SETTLE_FRAMES && cap.entered.elapsed() >= SETTLE_TIME
            {
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
                cap.requested = true;
            }
        }
        self.capture = Some(cap);
    }

    fn enter(&mut self, step: &Step) {
        if step
            .device
            .is_some_and(|k| !self.devices.iter().any(|d| d.desc.model_key == k))
        {
            for snap in extra_devices() {
                self.on_event(crate::worker::Event::DeviceAdded(Box::new(snap)));
            }
        }
        self.route = step.route;
        if let Some(key) = step.device {
            self.selected = self
                .devices
                .iter()
                .find(|d| d.desc.model_key == key)
                .map(|d| d.desc.id.clone());
        }
        self.tab = step.tab;
        self.lib.prefs.theme = if LIGHT.load(Ordering::Relaxed) {
            Theme::Light
        } else {
            step.theme
        };
    }

    /// A fake "permission denied" scan result, to show the banner.
    fn show_sample_issue(&mut self) {
        self.on_event(crate::worker::Event::ScanIssues(vec![
            gearwise_core::ScanIssue {
                path: "/dev/hidraw3".into(),
                vendor_id: 0x046d,
                product_id: 0xc547,
                product: Some("USB Receiver".into()),
                permission_denied: true,
                message: "permission denied opening /dev/hidraw3".into(),
            },
        ]));
    }

    fn clear_issues(&mut self) {
        self.dismiss_issues();
    }

    /// Drop every device, including the simulated extras the worker does not know about.
    fn clear_devices(&mut self) {
        self.devices.clear();
        self.selected = None;
    }

    /// Give the demo mouse a few non-default buttons so the lists look lived in.
    fn seed(&mut self) {
        let Some(dev) = self
            .devices
            .iter()
            .find(|d| d.desc.model_key == LOGITECH)
            .map(|d| d.desc.id.clone())
        else {
            return;
        };
        let macro_id = self.lib.macros.first().map(|m| m.id.clone());
        let mut buttons = vec![
            (
                0x56,
                ButtonAction::Key {
                    usage: 0x06,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            ),
            (0x5b, ButtonAction::Media(media::VOLUME_DOWN)),
            (0x5d, ButtonAction::Media(media::VOLUME_UP)),
        ];
        if let Some(id) = macro_id {
            buttons.push((0x53, ButtonAction::Macro(id)));
        }
        let settings = buttons
            .into_iter()
            .map(|(id, action)| Setting::Button { id, action })
            .collect();
        self.apply_many(&dev, settings);
    }
}

// --- `--hw-screenshot <dir>`: the real devices, as connected ----------------------

/// How long to wait for hardware to show up before giving up.
const HW_WAIT: Duration = Duration::from_secs(8);

pub struct HwCapture {
    dir: PathBuf,
    started: Instant,
    /// (file name, device id, tab), built once devices have arrived.
    shots: Vec<(String, String, Tab)>,
    step: usize,
    frames: u32,
    entered: Instant,
    requested: bool,
}

impl HwCapture {
    pub fn new(dir: PathBuf) -> Self {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("cannot create {}: {e}", dir.display());
        }
        Self {
            dir,
            started: Instant::now(),
            shots: Vec::new(),
            step: 0,
            frames: 0,
            entered: Instant::now(),
            requested: false,
        }
    }
}

impl App {
    /// Screenshot every connected device's tabs; called every frame from `logic`.
    pub(crate) fn drive_hw_capture(&mut self, ctx: &Context) {
        let Some(mut cap) = self.hw_capture.take() else {
            return;
        };
        ctx.request_repaint();
        if cap.shots.is_empty() {
            // Give receivers a moment to report every paired device.
            let settled = cap.started.elapsed() >= Duration::from_secs(3);
            if settled && !self.devices.is_empty() {
                cap.shots = self.hw_shot_list();
                cap.entered = Instant::now();
                if let Some((_, dev, tab)) = cap.shots.first().cloned() {
                    self.route = Route::Device;
                    self.selected = Some(dev);
                    self.tab = tab;
                }
            } else if cap.started.elapsed() >= HW_WAIT {
                eprintln!("no devices found");
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            self.hw_capture = Some(cap);
            return;
        }
        if cap.step >= cap.shots.len() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            self.hw_capture = Some(cap);
            return;
        }
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = shot {
            let factor = (image.size[0] as f32 / ctx.content_rect().width())
                .round()
                .max(1.0) as usize;
            let (w, h, rgb) = to_rgb(&image, factor);
            let path = cap.dir.join(format!("{}.png", cap.shots[cap.step].0));
            match std::fs::write(&path, png::encode_rgb(w, h, &rgb)) {
                Ok(()) => println!("wrote {} ({w}x{h})", path.display()),
                Err(e) => eprintln!("cannot write {}: {e}", path.display()),
            }
            cap.step += 1;
            cap.frames = 0;
            cap.requested = false;
            cap.entered = Instant::now();
            if let Some((_, dev, tab)) = cap.shots.get(cap.step).cloned() {
                self.selected = Some(dev);
                self.tab = tab;
            }
        } else {
            cap.frames += 1;
            if !cap.requested && cap.frames >= SETTLE_FRAMES && cap.entered.elapsed() >= SETTLE_TIME
            {
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
                cap.requested = true;
            }
        }
        self.hw_capture = Some(cap);
    }

    fn hw_shot_list(&self) -> Vec<(String, String, Tab)> {
        let mut out = Vec::new();
        for (n, d) in self
            .devices
            .iter()
            .filter(|d| !d.desc.simulated)
            .enumerate()
        {
            let slug: String = d
                .desc
                .name
                .to_ascii_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect();
            let mut tabs = Vec::new();
            if !d.caps.buttons.is_empty() {
                tabs.push((Tab::Assignments, "assignments"));
            }
            if d.caps.dpi.is_some() {
                tabs.push((Tab::Sensitivity, "sensitivity"));
            }
            if !d.caps.lighting.is_empty() || d.caps.per_key.is_some() {
                tabs.push((Tab::Lighting, "lighting"));
            }
            tabs.push((Tab::Info, "info"));
            for (tab, name) in tabs {
                out.push((format!("hw-{n}-{slug}-{name}"), d.desc.id.clone(), tab));
            }
        }
        out
    }
}
