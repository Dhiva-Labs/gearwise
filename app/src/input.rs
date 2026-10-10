//! Macro playback seam. The UI reports trigger presses of buttons assigned
//! to a macro; the engine decides what to do with them.

#[derive(Default)]
pub struct MacroEngine;

impl MacroEngine {
    pub fn trigger_down(&mut self, macro_id: &str, trigger: u16) {
        log::info!("macro {macro_id}: trigger 0x{trigger:04x} down");
    }

    pub fn trigger_up(&mut self, macro_id: &str, trigger: u16) {
        log::info!("macro {macro_id}: trigger 0x{trigger:04x} up");
    }

    pub fn stop_all(&mut self) {
        log::info!("macro playback: stop all");
    }
}
