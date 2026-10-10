//! `--screenshot <dir>`: walk through the main views with demo devices and write PNGs.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Context, ViewportCommand};
use gearwise_core::device::{Capabilities, DeviceDescriptor, DeviceState, Setting};
use gearwise_core::model::{
    ButtonAction, ButtonCaps, DeviceKind, DpiFunction, EffectKind, LightingEffect,
    LightingZoneCaps, Modifiers, MouseButton, Rgb, Vendor, WaveDirection, ZoneState, media,
};
use gearwise_core::profile::{Library, Macro, MacroMode, MacroStep, Profile, Theme};

use crate::app::App;
use crate::pages::lighting::LightMode;
use crate::png;
use crate::state::{Action, Route, Tab, ToastKind};
use crate::worker::DeviceSnapshot;

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

struct Step {
    name: &'static str,
    route: Route,
    device: Option<&'static str>,
    tab: Tab,
    theme: Theme,
    prep: fn(&mut App),
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
    vec![
        dev("01-assignments-mouse", LOGITECH, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(0x56)
        }),
        dev("02-sensitivity", LOGITECH, Tab::Sensitivity, nothing),
        dev("03-lighting-mouse", LOGITECH, Tab::Lighting, |a| {
            a.pages.light.mode = LightMode::Effects
        }),
        dev("04-lighting-keyboard", RAZER, Tab::Lighting, |a| {
            a.pages.light.mode = LightMode::Effects
        }),
        dev("05-lighting-per-key", RAZER, Tab::Lighting, |a| {
            a.pages.light.mode = LightMode::PerKey
        }),
        dev("06-assignments-keyboard", RAZER, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(0x101)
        }),
        dev("07-performance", CORSAIR, Tab::Performance, nothing),
        page("08-profiles", Route::Profiles),
        page("09-macros", Route::Macros),
        page("10-settings", Route::Settings),
        Step {
            theme: Theme::Light,
            ..dev("11-light-assignments", LOGITECH, Tab::Assignments, |a| {
                a.pages.assign.selected = Some(0x5b)
            })
        },
        Step {
            theme: Theme::Light,
            ..dev("12-light-lighting", LOGITECH, Tab::Lighting, |a| {
                a.pages.light.zone = 1
            })
        },
        dev("13-g502-hero-callouts", HERO, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(5)
        }),
        dev(
            "14-superlight-callouts",
            SUPERLIGHT,
            Tab::Assignments,
            |a| a.pages.assign.selected = Some(3),
        ),
        dev("15-naga-button-grid", NAGA, Tab::Assignments, |a| {
            a.pages.assign.selected = Some(8)
        }),
        dev("16-deathadder-lighting", DEATHADDER, Tab::Lighting, nothing),
        dev("17-headset-lighting", HEADSET, Tab::Lighting, nothing),
        dev("20-k55-zone-lighting", K55, Tab::Lighting, |a| {
            a.pages.light.zone = 3
        }),
        dev("21-k55-all-zones", K55, Tab::Lighting, |a| {
            a.pages.light.zone = crate::pages::lighting::ALL_ZONES
        }),
        dev("18-info-permission-banner", LOGITECH, Tab::Info, |a| {
            a.show_sample_issue();
            a.toasts.push(ToastKind::Success, "Saved to G502 X Plus");
            a.toasts.push(
                ToastKind::Error,
                "Report rate: not supported by this device",
            );
        }),
        page("19-empty-state", Route::Device).with_prep(|a| {
            a.clear_issues();
            a.queue_action(Action::SetDemo(false));
        }),
    ]
}

impl Step {
    fn with_prep(mut self, prep: fn(&mut App)) -> Self {
        self.prep = prep;
        self
    }
}

/// Profiles and macros so those pages have something to show.
pub fn sample_library() -> Library {
    let mut lib = Library::starter();
    lib.prefs.demo_devices = true;
    let mut fps = Profile::new("FPS competitive");
    fps.apps = vec!["valorant.exe".into(), "cs2".into()];
    let mut studio = Profile::new("Creative work");
    studio.apps = vec!["blender".into(), "photoshop.exe".into()];
    lib.profiles.extend([fps, studio]);

    let mut reload = Macro::new("Reload and inspect");
    reload.steps = vec![
        MacroStep::KeyTap {
            usage: 0x15,
            mods: Modifiers(0),
        },
        MacroStep::Delay(180),
        MacroStep::KeyTap {
            usage: 0x09,
            mods: Modifiers(0),
        },
        MacroStep::Delay(60),
        MacroStep::MouseDown(MouseButton::Left),
        MacroStep::Delay(40),
        MacroStep::MouseUp(MouseButton::Left),
    ];
    let mut rapid = Macro::new("Rapid click");
    rapid.mode = MacroMode::WhileHeld;
    rapid.steps = vec![
        MacroStep::MouseDown(MouseButton::Left),
        MacroStep::Delay(25),
        MacroStep::MouseUp(MouseButton::Left),
        MacroStep::Delay(25),
    ];
    let mut sig = Macro::new("Email signature");
    sig.steps = vec![
        MacroStep::Text("Best regards,".into()),
        MacroStep::KeyTap {
            usage: 0x28,
            mods: Modifiers(0),
        },
        MacroStep::Text("Alex".into()),
    ];
    lib.macros.extend([reload, rapid, sig]);
    lib
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
        self.lib.prefs.theme = step.theme;
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

// --- Simulated extras ---------------------------------------------------------------

fn btn(id: u16, name: &str, default: ButtonAction, hotspot: Option<(f32, f32)>) -> ButtonCaps {
    ButtonCaps {
        id,
        name: name.into(),
        remappable: true,
        default_action: default,
        hotspot,
    }
}

fn mouse_btn(b: MouseButton) -> ButtonAction {
    ButtonAction::Mouse(b)
}

/// Extra simulated devices (never sent to the worker) that show every silhouette.
pub(crate) fn extra_devices() -> Vec<DeviceSnapshot> {
    let mut demo = gearwise_core::demo::devices();
    let mut base = demo.remove(0);
    let (d0, c0) = (base.descriptor().clone(), base.capabilities().clone());
    let s0 = base.read_state().unwrap_or_default();
    let zone = |id: u8, name: &str| LightingZoneCaps {
        id,
        name: name.into(),
        effects: EffectKind::ALL[..5].to_vec(),
        brightness: true,
    };
    let lit = |color: Rgb| ZoneState {
        effect: LightingEffect::Static { color },
        brightness: 90,
    };
    let wave = ZoneState {
        effect: LightingEffect::Wave {
            period_ms: 3500,
            direction: WaveDirection::LeftToRight,
        },
        brightness: 80,
    };

    let make = |name: &str,
                vendor: Vendor,
                kind: DeviceKind,
                pid: u16,
                buttons: Vec<ButtonCaps>,
                zones: Vec<LightingZoneCaps>,
                lighting: Vec<(u8, ZoneState)>,
                custom: Vec<(u16, ButtonAction)>| {
        let mut descriptor = DeviceDescriptor {
            id: format!("demo:{}:{pid:04x}", vendor.key()),
            model_key: DeviceDescriptor::model_key_for(vendor, pid),
            vendor,
            kind,
            name: format!("{name} (demo)"),
            ..d0.clone()
        };
        descriptor.product_id = pid;
        let mut capabilities = Capabilities {
            buttons,
            lighting: zones,
            ..c0.clone()
        };
        if kind == DeviceKind::Headset {
            capabilities.dpi = None;
            capabilities.report_rates.clear();
            capabilities.onboard_profiles = 0;
        }
        let mut state = DeviceState {
            lighting: lighting.into_iter().collect(),
            ..s0.clone()
        };
        state.buttons = custom.into_iter().collect();
        DeviceSnapshot {
            descriptor,
            capabilities,
            state,
        }
    };

    let hero_buttons = vec![
        btn(
            0,
            "Left click",
            mouse_btn(MouseButton::Left),
            Some((0.30, 0.20)),
        ),
        btn(
            1,
            "Right click",
            mouse_btn(MouseButton::Right),
            Some((0.70, 0.20)),
        ),
        btn(
            2,
            "Middle click",
            mouse_btn(MouseButton::Middle),
            Some((0.50, 0.17)),
        ),
        btn(3, "Back", mouse_btn(MouseButton::Back), Some((0.10, 0.50))),
        btn(
            4,
            "Forward",
            mouse_btn(MouseButton::Forward),
            Some((0.11, 0.40)),
        ),
        btn(
            5,
            "G6 · DPI shift (1200)",
            ButtonAction::Dpi(DpiFunction::Shift(1200)),
            None,
        ),
        btn(
            6,
            "G7 · DPI down",
            ButtonAction::Dpi(DpiFunction::Down),
            None,
        ),
        btn(7, "G8 · DPI up", ButtonAction::Dpi(DpiFunction::Up), None),
        btn(8, "G9 · Profile cycle", ButtonAction::ProfileCycle, None),
        btn(
            9,
            "G10 · Scroll right",
            mouse_btn(MouseButton::WheelRight),
            None,
        ),
        btn(
            10,
            "G11 · Scroll left",
            mouse_btn(MouseButton::WheelLeft),
            None,
        ),
    ];
    let slim_buttons = vec![
        btn(0, "Left click", mouse_btn(MouseButton::Left), None),
        btn(1, "Right click", mouse_btn(MouseButton::Right), None),
        btn(2, "Middle click", mouse_btn(MouseButton::Middle), None),
        btn(3, "Back", mouse_btn(MouseButton::Back), None),
        btn(4, "Forward", mouse_btn(MouseButton::Forward), None),
        btn(5, "DPI button", ButtonAction::Dpi(DpiFunction::Cycle), None),
    ];
    let mut naga_buttons = slim_buttons[..3].to_vec();
    naga_buttons.push(btn(3, "DPI up", ButtonAction::Dpi(DpiFunction::Up), None));
    naga_buttons.push(btn(
        4,
        "DPI down",
        ButtonAction::Dpi(DpiFunction::Down),
        None,
    ));
    naga_buttons.extend((1..=12u16).map(|n| {
        btn(
            4 + n,
            &format!("Side button {n}"),
            ButtonAction::Key {
                usage: 0x1d + n as u8,
                mods: Modifiers(0),
            },
            None,
        )
    }));

    vec![
        make(
            "G502 HERO",
            Vendor::Logitech,
            DeviceKind::Mouse,
            0xc08b,
            hero_buttons,
            vec![zone(0, "Primary"), zone(1, "Logo")],
            vec![(0, wave), (1, lit(Rgb::new(0, 184, 252)))],
            vec![
                (
                    3,
                    ButtonAction::Key {
                        usage: 0x05,
                        mods: Modifiers(Modifiers::LCTRL),
                    },
                ),
                (9, ButtonAction::Media(media::NEXT)),
                (10, ButtonAction::Media(media::PREVIOUS)),
            ],
        ),
        make(
            "G Pro X Superlight",
            Vendor::Logitech,
            DeviceKind::Mouse,
            0xc094,
            slim_buttons.clone(),
            vec![zone(0, "Logo")],
            vec![(0, lit(Rgb::new(255, 77, 94)))],
            vec![(
                3,
                ButtonAction::Key {
                    usage: 0x06,
                    mods: Modifiers(Modifiers::LCTRL),
                },
            )],
        ),
        make(
            "Naga Pro",
            Vendor::Razer,
            DeviceKind::Mouse,
            0x0067,
            naga_buttons,
            vec![zone(0, "Logo"), zone(1, "Scroll wheel")],
            vec![
                (0, lit(Rgb::new(68, 214, 44))),
                (1, lit(Rgb::new(68, 214, 44))),
            ],
            vec![
                (8, ButtonAction::Disabled),
                (10, ButtonAction::Media(media::MUTE)),
            ],
        ),
        make(
            "DeathAdder V3",
            Vendor::Razer,
            DeviceKind::Mouse,
            0x00b6,
            slim_buttons,
            vec![zone(0, "Logo"), zone(1, "Scroll wheel")],
            vec![
                (0, lit(Rgb::new(139, 92, 246))),
                (
                    1,
                    ZoneState {
                        effect: LightingEffect::Breathing {
                            color: Rgb::new(139, 92, 246),
                            period_ms: 3000,
                        },
                        brightness: 100,
                    },
                ),
            ],
            vec![],
        ),
        make(
            "K55 RGB PRO",
            Vendor::Corsair,
            DeviceKind::Keyboard,
            0x1ba4,
            Vec::new(),
            std::iter::once((0, "Win-lock indicator".to_string()))
                .chain((1..=5).map(|i| (i, format!("Zone {i}"))))
                .map(|(id, name)| LightingZoneCaps {
                    id,
                    name,
                    effects: vec![EffectKind::Off, EffectKind::Static],
                    brightness: true,
                })
                .collect(),
            vec![
                (1, lit(Rgb::new(255, 40, 60))),
                (2, lit(Rgb::new(255, 160, 0))),
                (3, lit(Rgb::new(0, 212, 170))),
                (4, lit(Rgb::new(0, 140, 255))),
                (5, lit(Rgb::new(150, 70, 255))),
            ],
            vec![],
        ),
        make(
            "HS80 RGB Wireless",
            Vendor::Corsair,
            DeviceKind::Headset,
            0x0a64,
            Vec::new(),
            vec![zone(0, "Logo")],
            vec![(0, lit(Rgb::new(255, 204, 0)))],
            vec![],
        ),
    ]
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
