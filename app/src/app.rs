//! The eframe application: owns all state, routes between pages and talks to the worker.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Align, Color32, Context, CornerRadius, Frame, Layout, Margin, Panel, RichText, ScrollArea, Stroke, Ui};
use periph_core::ScanIssue;
use periph_core::device::Setting;
use periph_core::model::ButtonAction;
use periph_core::profile::{Library, Theme, default_library_path, settings_from_state};

use crate::debounce::Debouncer;
use crate::icons::Icon;
use crate::input::MacroEngine;
use crate::keymap::{self, KeyEv};
use crate::pages::{self, PageStates};
use crate::persist;
use crate::screenshot::{self, Capture};
use crate::sidebar::{self, Nav};
use crate::state::{Action, DeviceView, Route, SettingKey, Tab, ToastKind, merge_settings, setting_key};
use crate::theme::{self, Palette, palette, with_alpha};
use crate::toasts::Toasts;
use crate::widgets;
use crate::worker::{Command, Event, Worker};

const SAVE_DELAY: Duration = Duration::from_millis(500);
const APPLY_DELAY: Duration = Duration::from_millis(150);

pub struct Options {
    /// Run the screenshot tour (demo devices, nothing persisted) and exit.
    pub screenshot_dir: Option<PathBuf>,
    /// Library file; `None` means the default location.
    pub library_path: Option<PathBuf>,
}

pub struct App {
    pub(crate) worker: Worker,
    macros: MacroEngine,
    pub(crate) devices: Vec<DeviceView>,
    issues: Vec<ScanIssue>,
    banner_dismissed: bool,
    pub(crate) route: Route,
    pub(crate) selected: Option<String>,
    pub(crate) tab: Tab,
    pub(crate) pages: PageStates,
    keys: Vec<KeyEv>,
    debounce: Debouncer<(String, SettingKey), Setting>,
    pub(crate) toasts: Toasts,
    pub(crate) lib: Library,
    /// What was last written (or loaded), to notice edits.
    saved: Library,
    save_path: Option<PathBuf>,
    save_due: Option<Instant>,
    look: (Theme, String),
    pub(crate) capture: Option<Capture>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: Options) -> Self {
        Self::with_context(&cc.egui_ctx, opts)
    }

    pub fn with_context(ctx: &Context, opts: Options) -> Self {
        let mut toasts = Toasts::default();
        let (lib, save_path) = if opts.screenshot_dir.is_some() {
            (screenshot::sample_library(), None)
        } else {
            let path = opts.library_path.unwrap_or_else(default_library_path);
            let loaded = persist::load(&path);
            if let Some(n) = loaded.notice {
                toasts.push(ToastKind::Error, n);
            }
            (loaded.lib, Some(path))
        };
        let look = (lib.prefs.theme, lib.prefs.accent.clone());
        theme::apply(ctx, &Palette::new(look.0, theme::parse_hex(&look.1).unwrap_or(Color32::from_rgb(0, 0xd4, 0xaa))));
        let worker = Worker::spawn(ctx.clone(), lib.prefs.demo_devices);
        Self {
            worker,
            macros: MacroEngine,
            devices: Vec::new(),
            issues: Vec::new(),
            banner_dismissed: false,
            route: Route::Device,
            selected: None,
            tab: Tab::Assignments,
            pages: PageStates::default(),
            keys: Vec::new(),
            debounce: Debouncer::new(APPLY_DELAY),
            toasts,
            saved: lib.clone(),
            lib,
            save_path,
            save_due: None,
            look,
            capture: opts.screenshot_dir.map(Capture::new),
        }
    }

    // --- Worker events ---------------------------------------------------------

    fn pump_worker(&mut self) {
        while let Ok(ev) = self.worker.rx.try_recv() {
            self.on_event(ev);
        }
    }

    pub(crate) fn dismiss_issues(&mut self) {
        self.issues.clear();
    }

    fn view_mut(&mut self, id: &str) -> Option<&mut DeviceView> {
        self.devices.iter_mut().find(|d| d.desc.id == id)
    }

    fn has_queued(&self, dev: &str) -> bool {
        self.debounce.count_where(|k| k.0 == dev) > 0
    }

    pub(crate) fn on_event(&mut self, ev: Event) {
        match ev {
            Event::DeviceAdded(snap) => {
                let announce = !snap.descriptor.simulated;
                let name = snap.descriptor.name.clone();
                let view = DeviceView::new(*snap);
                match self.devices.iter().position(|d| d.desc.id == view.desc.id) {
                    Some(i) => self.devices[i] = view,
                    None => {
                        self.devices.push(view);
                        // Real hardware first, simulated devices after.
                        self.devices.sort_by_key(|d| d.desc.simulated);
                    }
                }
                if announce {
                    self.toasts.push(ToastKind::Success, format!("{name} connected"));
                }
                if self.selected.is_none() {
                    self.selected = self.devices.first().map(|d| d.desc.id.clone());
                }
            }
            Event::DeviceRemoved(id) => {
                if let Some(i) = self.devices.iter().position(|d| d.desc.id == id) {
                    let d = self.devices.remove(i);
                    if !d.desc.simulated {
                        self.toasts.push(ToastKind::Info, format!("{} disconnected", d.desc.name));
                    }
                }
                self.debounce.retain(|k| k.0 != id);
                if self.selected.as_deref() == Some(id.as_str()) {
                    self.selected = self.devices.first().map(|d| d.desc.id.clone());
                }
            }
            Event::State { id, state } => {
                let queued = self.has_queued(&id);
                if let Some(d) = self.view_mut(&id) {
                    if d.pending > 0 || queued {
                        d.stale = true;
                    } else {
                        d.state = state;
                    }
                }
            }
            Event::Applied { id, error, state } => {
                let queued = self.has_queued(&id);
                let mut refresh = false;
                let mut failure = None;
                if let Some(d) = self.view_mut(&id) {
                    d.pending = d.pending.saturating_sub(1);
                    if let Some(e) = error {
                        failure = Some(format!("{}: {e}", d.desc.name));
                    }
                    let idle = d.pending == 0 && !queued;
                    match state {
                        Some(s) if idle => {
                            d.state = s;
                            d.stale = false;
                        }
                        Some(_) => d.stale = true,
                        None => {}
                    }
                    if idle && d.stale {
                        d.stale = false;
                        refresh = true;
                    }
                }
                if let Some(m) = failure {
                    self.toasts.push(ToastKind::Error, m);
                }
                if refresh {
                    self.worker.send(Command::Refresh { id });
                }
            }
            Event::Saved { id, error } => {
                let name = self.devices.iter().find(|d| d.desc.id == id).map_or_else(|| "device".to_string(), |d| d.display_name().to_string());
                match error {
                    None => self.toasts.push(ToastKind::Success, format!("Saved to {name}")),
                    Some(e) => self.toasts.push(ToastKind::Error, format!("Could not save to {name}: {e}")),
                }
            }
            Event::Battery { id, battery } => {
                if let Some(d) = self.view_mut(&id) {
                    d.state.battery = Some(battery);
                }
            }
            Event::Online { id, online } => {
                if let Some(d) = self.view_mut(&id) {
                    d.online = online;
                }
            }
            Event::ScanIssues(issues) => {
                for i in issues {
                    if i.permission_denied {
                        self.banner_dismissed = false;
                    }
                    if !self.issues.iter().any(|e| e.path == i.path) {
                        self.issues.push(i);
                    }
                }
            }
            Event::Button { id, button, down } => {
                let action = self.devices.iter().find(|d| d.desc.id == id).and_then(|d| d.state.buttons.get(&button)).cloned();
                if let Some(ButtonAction::Macro(mid)) = action {
                    if down {
                        self.macros.trigger_down(&mid, button);
                    } else {
                        self.macros.trigger_up(&mid, button);
                    }
                }
            }
        }
    }

    // --- Applying settings -----------------------------------------------------

    fn send_apply(&mut self, dev: &str, setting: Setting) {
        if let Some(d) = self.view_mut(dev) {
            d.pending += 1;
            self.worker.send(Command::Apply { id: dev.to_string(), setting });
        }
    }

    /// Update the UI state right away, then send now or after the debounce delay.
    pub(crate) fn apply(&mut self, dev: &str, setting: Setting, debounce: bool) {
        let key = (dev.to_string(), setting_key(&setting));
        let Some(d) = self.view_mut(dev) else { return };
        setting.apply_to(&mut d.state);
        if debounce {
            self.debounce.push(key, setting, Instant::now(), merge_settings);
            return;
        }
        let merged = match self.debounce.take(&key) {
            Some(old) => merge_settings(old, setting),
            None => setting,
        };
        // An empty per-key update only exists to flush what was queued.
        if !matches!(&merged, Setting::PerKey(m) if m.is_empty()) {
            self.send_apply(dev, merged);
        }
    }

    pub(crate) fn apply_many(&mut self, dev: &str, settings: Vec<Setting>) {
        if settings.is_empty() {
            return;
        }
        let Some(d) = self.view_mut(dev) else { return };
        for s in &settings {
            s.apply_to(&mut d.state);
        }
        d.pending += 1;
        self.worker.send(Command::ApplySettings { id: dev.to_string(), settings });
    }

    fn flush_debounced(&mut self, ctx: &Context) {
        for ((dev, _), setting) in self.debounce.take_due(Instant::now()) {
            self.send_apply(&dev, setting);
        }
        if let Some(wait) = self.debounce.next_due(Instant::now()) {
            ctx.request_repaint_after(wait);
        }
    }

    /// Send a profile's settings to every connected device it covers.
    fn apply_profile(&mut self, id: &str, activating: bool) {
        let Some(profile) = self.lib.profile(id).cloned() else { return };
        let targets: Vec<(String, Vec<Setting>)> = self
            .devices
            .iter()
            .filter_map(|d| profile.devices.get(&d.desc.model_key).filter(|s| !s.is_empty()).map(|s| (d.desc.id.clone(), s.clone())))
            .collect();
        let n = targets.len();
        for (dev, settings) in targets {
            self.apply_many(&dev, settings);
        }
        let msg = match (n, activating) {
            (0, true) => format!("“{}” is now active", profile.name),
            (0, false) => format!("“{}” has nothing saved for the connected devices", profile.name),
            (n, _) => format!("“{}” applied to {n} device{}", profile.name, if n == 1 { "" } else { "s" }),
        };
        self.toasts.push(ToastKind::Success, msg);
    }

    fn save_to_profile(&mut self, dev: &str) {
        let Some(active) = self.lib.active_profile.clone().filter(|id| self.lib.profile(id).is_some()) else {
            self.toasts.push(ToastKind::Error, "No active profile. Create one on the Profiles page first.");
            return;
        };
        let Some(d) = self.devices.iter().find(|d| d.desc.id == dev) else { return };
        let settings = settings_from_state(&d.caps, &d.state);
        let (key, n) = (d.desc.model_key.clone(), settings.len());
        if let Some(p) = self.lib.profile_mut(&active) {
            p.devices.insert(key, settings);
            let name = p.name.clone();
            self.toasts.push(ToastKind::Success, format!("Saved {n} settings to “{name}”"));
        }
    }

    pub(crate) fn dispatch(&mut self, action: Action, ctx: &Context) {
        match action {
            Action::Apply { dev, setting, debounce } => self.apply(&dev, setting, debounce),
            Action::ApplyMany { dev, settings } => self.apply_many(&dev, settings),
            Action::SaveOnboard(id) => self.worker.send(Command::SaveOnboard { id }),
            Action::SaveToProfile(dev) => self.save_to_profile(&dev),
            Action::ActivateProfile(id) => {
                self.lib.active_profile = Some(id.clone());
                self.apply_profile(&id, true);
            }
            Action::ApplyProfile(id) => self.apply_profile(&id, false),
            Action::SelectDevice(id) => {
                self.selected = Some(id);
                self.route = Route::Device;
            }
            Action::Navigate(r) => self.route = r,
            Action::SetDemo(on) => {
                self.lib.prefs.demo_devices = on;
                self.worker.send(Command::SetDemo(on));
            }
            Action::Rescan => self.worker.send(Command::Scan),
            Action::CopyText(t) => ctx.copy_text(t),
            Action::Toast(kind, text) => self.toasts.push(kind, text),
        }
    }

    // --- Per-frame housekeeping --------------------------------------------------

    fn sync_look(&mut self, ctx: &Context) {
        let want = (self.lib.prefs.theme, self.lib.prefs.accent.clone());
        if want != self.look {
            let accent = theme::parse_hex(&want.1).unwrap_or(Color32::from_rgb(0, 0xd4, 0xaa));
            theme::apply(ctx, &Palette::new(want.0, accent));
            self.look = want;
        }
    }

    fn autosave(&mut self, ctx: &Context) {
        if self.lib != self.saved {
            self.saved = self.lib.clone();
            self.save_due = Some(Instant::now() + SAVE_DELAY);
        }
        if let Some(due) = self.save_due {
            let now = Instant::now();
            if now >= due {
                self.save_due = None;
                self.write_library();
            } else {
                ctx.request_repaint_after(due - now);
            }
        }
    }

    fn write_library(&mut self) {
        if let Some(path) = &self.save_path
            && let Err(e) = self.lib.save(path) {
                self.toasts.push(ToastKind::Error, format!("Could not save settings: {e}"));
            }
    }

    fn current_device(&self) -> Option<usize> {
        let by_id = self.selected.as_deref().and_then(|id| self.devices.iter().position(|d| d.desc.id == id));
        by_id.or(if self.devices.is_empty() { None } else { Some(0) })
    }

    // --- UI ------------------------------------------------------------------------

    fn banner(&mut self, ui: &mut Ui, pal: &Palette, out: &mut Vec<Action>) {
        let denied = self.issues.iter().filter(|i| i.permission_denied).count();
        if denied == 0 || self.banner_dismissed {
            return;
        }
        Frame::new()
            .fill(with_alpha(pal.warning, 26))
            .stroke(Stroke::new(1.0, with_alpha(pal.warning, 130)))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    crate::icons::show(ui, Icon::Warning, 18.0, pal.warning);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;
                        let what = format!("{denied} device{} could not be opened. ", if denied == 1 { "" } else { "s" });
                        let how = if cfg!(windows) {
                            "Close Logitech G HUB, Razer Synapse or Corsair iCUE, then reconnect the device."
                        } else {
                            "Install the udev rules, then reconnect the device:"
                        };
                        ui.label(RichText::new(format!("{what}{how}")).color(pal.text));
                        if !cfg!(windows) {
                            pages::settings::code_box(ui, pages::settings::UDEV_COMMAND);
                        }
                    });
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if widgets::icon_button(ui, Icon::Close, "Dismiss").clicked() {
                            self.banner_dismissed = true;
                        }
                        if !cfg!(windows) && widgets::secondary(ui, Some(Icon::Copy), "Copy command").clicked() {
                            out.push(Action::CopyText(pages::settings::UDEV_COMMAND.to_string()));
                            out.push(Action::Toast(ToastKind::Success, "Command copied".into()));
                        }
                    });
                });
            });
        ui.add_space(12.0);
    }

    fn central(&mut self, ui: &mut Ui, pal: &Palette, out: &mut Vec<Action>) {
        self.banner(ui, pal, out);
        let scroll = |ui: &mut Ui, add: &mut dyn FnMut(&mut Ui)| {
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.set_width(ui.available_width());
                add(ui);
                ui.add_space(24.0);
            });
        };
        match self.route {
            Route::Device => match self.current_device() {
                Some(i) => pages::device::show(ui, &mut self.pages, &self.devices[i], &self.lib, &self.keys, &mut self.tab, out),
                None => pages::empty::show(ui, out),
            },
            Route::Profiles => scroll(ui, &mut |ui| pages::profiles::show(ui, &mut self.pages.profiles, &mut self.lib, &self.devices, out)),
            Route::Macros => scroll(ui, &mut |ui| pages::macros::show(ui, &mut self.pages.macros, &mut self.lib, &self.devices, &self.keys, out)),
            Route::Settings => scroll(ui, &mut |ui| pages::settings::show(ui, &mut self.pages.settings, &mut self.lib, out)),
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.pump_worker();
        // Hide key presses from widgets while a page is capturing them.
        self.keys = if self.pages.wants_keys() { keymap::take_key_events(ctx) } else { Vec::new() };
        self.flush_debounced(ctx);
        self.sync_look(ctx);
        self.autosave(ctx);
        self.drive_capture(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let pal = palette(&ctx);
        let mut out: Vec<Action> = Vec::new();

        let nav = Nav { devices: &self.devices, selected: self.selected.as_deref(), route: self.route, lib: &self.lib };
        Panel::left("sidebar")
            .exact_size(252.0)
            .resizable(false)
            .frame(Frame::new().fill(pal.sidebar).inner_margin(Margin::same(14)))
            .show(ui, |ui| sidebar::show(ui, &nav, &mut out));
        egui::CentralPanel::default()
            .frame(Frame::new().fill(pal.window).inner_margin(Margin::symmetric(28, 18)))
            .show(ui, |ui| self.central(ui, &pal, &mut out));
        self.toasts.show(&ctx);

        for a in out {
            self.dispatch(a, &ctx);
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.macros.stop_all();
        if self.lib != self.saved || self.save_due.is_some() {
            self.write_library();
        }
    }
}

#[cfg(test)]
mod tests;
