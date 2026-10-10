//! Software actions for buttons a device hands to the host (diverted
//! Logitech controls, G-keys): macros through the macro player, plain
//! keystrokes / mouse buttons / media keys through the virtual input sink.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use periph_core::model::{ButtonAction, Modifiers};
use periph_core::profile::Macro;
use periph_input::{MacroPlayer, SharedSink};

/// Modifier bits paired with the HID usage of the key that produces them.
const MOD_KEYS: [(u8, u8); 8] = [
    (Modifiers::LCTRL, 0xe0),
    (Modifiers::LSHIFT, 0xe1),
    (Modifiers::LALT, 0xe2),
    (Modifiers::LGUI, 0xe3),
    (Modifiers::RCTRL, 0xe4),
    (Modifiers::RSHIFT, 0xe5),
    (Modifiers::RALT, 0xe6),
    (Modifiers::RGUI, 0xe7),
];

#[derive(Default)]
pub struct MacroEngine {
    sink: Option<SharedSink>,
    player: Option<MacroPlayer>,
    /// The sink could not be created; don't retry on every press.
    unavailable: Option<String>,
    /// Actions currently held down, by trigger, so release mirrors press.
    held: HashMap<String, ButtonAction>,
}

impl MacroEngine {
    /// The shared sink, created on first use (opening `/dev/uinput` is not
    /// free, and most users never need it). Returns the error once.
    fn sink(&mut self) -> Result<SharedSink, Option<String>> {
        if let Some(s) = &self.sink {
            return Ok(s.clone());
        }
        if self.unavailable.is_some() {
            return Err(None);
        }
        match periph_input::create_sink() {
            Ok(b) => {
                let s: SharedSink = Arc::new(Mutex::new(b));
                self.player = Some(MacroPlayer::new(s.clone()));
                self.sink = Some(s.clone());
                Ok(s)
            }
            Err(e) => {
                let msg = format!("Software button actions are unavailable: {e}");
                log::warn!("{msg}");
                self.unavailable = Some(msg.clone());
                Err(Some(msg))
            }
        }
    }

    /// A diverted button went down. Returns a user-facing error the first
    /// time playback can't work (e.g. missing uinput permission).
    pub fn press(
        &mut self,
        trigger: &str,
        action: &ButtonAction,
        macros: &[Macro],
    ) -> Option<String> {
        let sink = match self.sink() {
            Ok(s) => s,
            Err(msg) => return msg,
        };
        match action {
            ButtonAction::Macro(id) => {
                if let (Some(m), Some(p)) = (macros.iter().find(|m| &m.id == id), &self.player) {
                    p.press(m, trigger);
                }
            }
            ButtonAction::Key { usage, mods } => {
                let mut s = sink.lock().unwrap();
                for (bit, key) in MOD_KEYS {
                    if mods.has(bit) {
                        log_err(s.key_down(key));
                    }
                }
                log_err(s.key_down(*usage));
            }
            ButtonAction::Mouse(b) => log_err(sink.lock().unwrap().mouse_down(*b)),
            ButtonAction::Media(u) => log_err(sink.lock().unwrap().media(*u)),
            _ => return None,
        }
        self.held.insert(trigger.to_string(), action.clone());
        None
    }

    pub fn release(&mut self, trigger: &str) {
        let Some(action) = self.held.remove(trigger) else {
            return;
        };
        let Some(sink) = self.sink.clone() else {
            return;
        };
        match action {
            ButtonAction::Macro(_) => {
                if let Some(p) = &self.player {
                    p.release(trigger);
                }
            }
            ButtonAction::Key { usage, mods } => {
                let mut s = sink.lock().unwrap();
                log_err(s.key_up(usage));
                for (bit, key) in MOD_KEYS.iter().rev() {
                    if mods.has(*bit) {
                        log_err(s.key_up(*key));
                    }
                }
            }
            ButtonAction::Mouse(b) => log_err(sink.lock().unwrap().mouse_up(b)),
            _ => {}
        }
    }

    pub fn stop_all(&mut self) {
        let triggers: Vec<String> = self.held.keys().cloned().collect();
        for t in triggers {
            self.release(&t);
        }
        if let Some(p) = &self.player {
            p.stop_all();
        }
    }
}

fn log_err(r: periph_core::Result<()>) {
    if let Err(e) = r {
        log::warn!("input injection failed: {e}");
    }
}
