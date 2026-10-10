//! Periph Studio: configure Logitech, Razer and Corsair mice and keyboards.

mod app;
mod debounce;
mod drivers;
mod fx;
mod icon;
mod icons;
mod illustration;
mod input;
mod keymap;
mod pages;
mod persist;
mod png;
mod screenshot;
mod sidebar;
mod state;
mod theme;
mod toasts;
mod widgets;
mod worker;

use std::path::PathBuf;

fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let mut screenshot_dir = None;
    let mut shot_size = [1280.0, 800.0];
    while let Some(a) = args.next() {
        match a.as_str() {
            "--screenshot" => screenshot_dir = args.next().map(PathBuf::from),
            // Hidden: window size for the screenshot tour, e.g. 1080x680.
            "--size" => {
                let parsed = args.next().and_then(|v| v.split_once('x').and_then(|(w, h)| Some([w.parse().ok()?, h.parse().ok()?])));
                shot_size = parsed.unwrap_or(shot_size);
            }
            "--version" | "-V" => {
                println!("periph-studio {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!("periph-studio [--version]");
                return Ok(());
            }
            _ => {}
        }
    }
    let size = if screenshot_dir.is_some() { shot_size } else { [1180.0, 780.0] };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Periph Studio")
        .with_icon(icon::window_icon())
        .with_app_id("periph-studio")
        .with_inner_size(size)
        .with_min_inner_size([1080.0, 660.0]);
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("Periph Studio", options, Box::new(|cc| Ok(Box::new(app::App::new(cc, app::Options { screenshot_dir, library_path: None })))))
}
