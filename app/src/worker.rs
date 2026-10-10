//! Device worker thread: owns the `DeviceManager`, executes UI commands and
//! reports results. The UI never touches a device directly.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use periph_core::device::{Capabilities, DeviceDescriptor, DeviceEvent, DeviceState, Setting};
use periph_core::model::Battery;
use periph_core::{DeviceManager, Error, ScanIssue, demo};

use crate::drivers;

const TICK: Duration = Duration::from_millis(30);
const SCAN_INTERVAL: Duration = Duration::from_secs(2);

/// UI to worker.
pub enum Command {
    /// Scan for hot-plugged devices right now.
    Scan,
    Apply {
        id: String,
        setting: Setting,
    },
    ApplySettings {
        id: String,
        settings: Vec<Setting>,
    },
    Refresh {
        id: String,
    },
    SaveOnboard {
        id: String,
    },
    SetDemo(bool),
    Shutdown,
}

/// Everything the UI needs to show a device.
#[derive(Clone)]
pub struct DeviceSnapshot {
    pub descriptor: DeviceDescriptor,
    pub capabilities: Capabilities,
    pub state: DeviceState,
}

/// Worker to UI.
pub enum Event {
    DeviceAdded(Box<DeviceSnapshot>),
    DeviceRemoved(String),
    /// Authoritative re-read of a device.
    State {
        id: String,
        state: DeviceState,
    },
    /// An `Apply` / `ApplySettings` finished. On failure `state` holds a re-read so the
    /// UI can roll back its optimistic update.
    Applied {
        id: String,
        error: Option<String>,
        state: Option<DeviceState>,
    },
    Saved {
        id: String,
        error: Option<String>,
    },
    Battery {
        id: String,
        battery: Battery,
    },
    Online {
        id: String,
        online: bool,
    },
    ScanIssues(Vec<ScanIssue>),
    /// A button diverted to software changed state.
    Button {
        id: String,
        button: u16,
        down: bool,
    },
}

pub struct Worker {
    tx: Sender<Command>,
    pub rx: Receiver<Event>,
    join: Option<JoinHandle<()>>,
}

impl Worker {
    /// `hardware: false` runs without real devices (tests, screenshot tour).
    pub fn spawn(ctx: egui::Context, demo: bool, hardware: bool) -> Self {
        let (tx, cmd_rx) = mpsc::channel();
        let (evt_tx, rx) = mpsc::channel();
        let join = std::thread::Builder::new()
            .name("device-worker".into())
            .spawn(move || Runner::new(ctx, evt_tx, demo, hardware).run(cmd_rx))
            .expect("spawn device worker");
        Self {
            tx,
            rx,
            join: Some(join),
        }
    }

    pub fn send(&self, cmd: Command) {
        // The worker only goes away on shutdown, when nothing matters any more.
        let _ = self.tx.send(cmd);
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

struct Runner {
    ctx: egui::Context,
    out: Sender<Event>,
    mgr: DeviceManager,
    demo_on: bool,
    demo_wanted: bool,
    sent: bool,
}

impl Runner {
    fn new(ctx: egui::Context, out: Sender<Event>, demo_wanted: bool, hardware: bool) -> Self {
        let mgr = if hardware {
            DeviceManager::new(drivers::backend(), drivers::drivers())
        } else {
            DeviceManager::new(drivers::offline(), Vec::new())
        };
        Self {
            ctx,
            out,
            mgr,
            demo_on: false,
            demo_wanted,
            sent: false,
        }
    }

    fn emit(&mut self, ev: Event) {
        self.sent = true;
        let _ = self.out.send(ev);
    }

    fn run(mut self, rx: Receiver<Command>) {
        let mut last_scan = Instant::now() - SCAN_INTERVAL;
        loop {
            match rx.recv_timeout(TICK) {
                Ok(Command::Shutdown) | Err(RecvTimeoutError::Disconnected) => return,
                Ok(cmd) => self.handle(cmd, &mut last_scan),
                Err(RecvTimeoutError::Timeout) => {}
            }
            while let Ok(cmd) = rx.try_recv() {
                if matches!(cmd, Command::Shutdown) {
                    return;
                }
                self.handle(cmd, &mut last_scan);
            }
            self.poll_devices();
            if last_scan.elapsed() >= SCAN_INTERVAL {
                last_scan = Instant::now();
                self.scan();
            }
            if std::mem::take(&mut self.sent) {
                self.ctx.request_repaint();
            }
        }
    }

    fn handle(&mut self, cmd: Command, last_scan: &mut Instant) {
        match cmd {
            Command::Scan => {
                *last_scan = Instant::now();
                self.scan();
            }
            Command::SetDemo(on) => {
                self.demo_wanted = on;
                self.sync_demo();
            }
            Command::Apply { id, setting } => {
                let result = self.with_device(&id, |d| d.apply(&setting));
                self.finish_apply(id, result, false);
            }
            Command::ApplySettings { id, settings } => {
                let result = self.with_device(&id, |d| {
                    let mut first_err = None;
                    for s in &settings {
                        if let Err(e) = d.apply(s) {
                            if matches!(e, Error::Disconnected) {
                                return Err(e);
                            }
                            log::warn!("{}: {}: {e}", d.descriptor().name, s.label());
                            first_err.get_or_insert_with(|| {
                                Error::protocol(format!("{}: {e}", s.label()))
                            });
                        }
                    }
                    first_err.map_or(Ok(()), Err)
                });
                self.finish_apply(id, result, true);
            }
            Command::Refresh { id } => match self.with_device(&id, |d| d.read_state()) {
                Ok(state) => self.emit(Event::State { id, state }),
                Err(e) => self.device_error(&id, &e),
            },
            Command::SaveOnboard { id } => {
                let result = self.with_device(&id, |d| d.save_onboard());
                if matches!(result, Err(Error::Disconnected)) {
                    self.drop_device(&id);
                } else {
                    self.emit(Event::Saved {
                        id,
                        error: result.err().map(|e| e.to_string()),
                    });
                }
            }
            Command::Shutdown => {}
        }
    }

    fn with_device<R>(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut Box<dyn periph_core::Device>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        match self.mgr.device_mut(id) {
            Some(d) => f(d),
            None => Err(Error::Disconnected),
        }
    }

    /// Reply to an apply command: re-read on failure (or always for batches).
    fn finish_apply(&mut self, id: String, result: Result<(), Error>, always_read: bool) {
        if matches!(result, Err(Error::Disconnected)) {
            // The UI counts one reply per command, so answer before dropping.
            self.emit(Event::Applied {
                id: id.clone(),
                error: Some(Error::Disconnected.to_string()),
                state: None,
            });
            self.drop_device(&id);
            return;
        }
        let error = result.err().map(|e| e.to_string());
        let state = if error.is_some() || always_read {
            self.with_device(&id, |d| d.read_state()).ok()
        } else {
            None
        };
        self.emit(Event::Applied { id, error, state });
    }

    fn device_error(&mut self, id: &str, e: &Error) {
        if matches!(e, Error::Disconnected) {
            self.drop_device(id);
        } else {
            log::warn!("{id}: {e}");
        }
    }

    fn drop_device(&mut self, id: &str) {
        self.mgr.drop_device(id);
        self.emit(Event::DeviceRemoved(id.to_string()));
    }

    fn poll_devices(&mut self) {
        let mut events = Vec::new();
        let mut refresh = Vec::new();
        for d in self.mgr.devices_mut() {
            let id = d.descriptor().id.clone();
            for ev in d.poll_events() {
                match ev {
                    DeviceEvent::ButtonDown(b) => events.push(Event::Button {
                        id: id.clone(),
                        button: b,
                        down: true,
                    }),
                    DeviceEvent::ButtonUp(b) => events.push(Event::Button {
                        id: id.clone(),
                        button: b,
                        down: false,
                    }),
                    DeviceEvent::BatteryChanged(battery) => events.push(Event::Battery {
                        id: id.clone(),
                        battery,
                    }),
                    DeviceEvent::StateChanged => refresh.push(id.clone()),
                    DeviceEvent::Disconnected => events.push(Event::Online {
                        id: id.clone(),
                        online: false,
                    }),
                    DeviceEvent::Reconnected => {
                        events.push(Event::Online {
                            id: id.clone(),
                            online: true,
                        });
                        refresh.push(id.clone());
                    }
                }
            }
        }
        for e in events {
            self.emit(e);
        }
        refresh.dedup();
        for id in refresh {
            if let Ok(state) = self.with_device(&id, |d| d.read_state()) {
                self.emit(Event::State { id, state });
            }
        }
    }

    fn scan(&mut self) {
        let report = self.mgr.scan();
        for id in report.removed {
            self.emit(Event::DeviceRemoved(id));
        }
        self.announce(&report.added);
        if !report.issues.is_empty() {
            self.emit(Event::ScanIssues(report.issues));
        }
        self.sync_demo();
    }

    fn sync_demo(&mut self) {
        if self.demo_wanted && !self.demo_on {
            self.demo_on = true;
            let ids = self.mgr.add_virtual(demo::devices());
            self.announce(&ids);
        } else if !self.demo_wanted && self.demo_on {
            self.demo_on = false;
            for id in self.mgr.remove_virtual() {
                self.emit(Event::DeviceRemoved(id));
            }
        }
    }

    /// Read and report freshly opened devices.
    fn announce(&mut self, ids: &[String]) {
        let mut snaps = Vec::new();
        for d in self.mgr.devices_mut() {
            if !ids.iter().any(|i| *i == d.descriptor().id) {
                continue;
            }
            let state = d.read_state().unwrap_or_else(|e| {
                log::warn!("{}: cannot read state: {e}", d.descriptor().name);
                DeviceState::default()
            });
            snaps.push(DeviceSnapshot {
                descriptor: d.descriptor().clone(),
                capabilities: d.capabilities().clone(),
                state,
            });
        }
        for s in snaps {
            self.emit(Event::DeviceAdded(Box::new(s)));
        }
    }
}
