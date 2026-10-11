//! Gearwise: configure Logitech, Razer, Corsair and SteelSeries mice and keyboards.

// A GUI app: no console window on Windows (release builds keep stderr off).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod debounce;
mod drivers;
mod fx;
mod icon;
mod icons;
mod illustration;
mod input;
mod keymap;
mod layout;
mod pages;
mod persist;
mod png;
mod screenshot;
mod screenshot_brands;
mod screenshot_demo;
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
    let mut hw_screenshot_dir = None;
    let mut shot_size = [1280.0, 800.0];
    while let Some(a) = args.next() {
        match a.as_str() {
            "--screenshot" => screenshot_dir = args.next().map(PathBuf::from),
            // Hidden: screenshot the connected hardware (throwaway library).
            "--hw-screenshot" => hw_screenshot_dir = args.next().map(PathBuf::from),
            // Hidden: write the app icon PNGs (16..512 px) into a directory and exit.
            "--export-icons" => {
                let dir = args
                    .next()
                    .map_or_else(|| PathBuf::from("packaging/icons"), PathBuf::from);
                return match icon::export(&dir) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        eprintln!("cannot export icons to {}: {e}", dir.display());
                        std::process::exit(1)
                    }
                };
            }
            // Hidden: only run tour steps whose name contains one of these (comma separated).
            "--only" => {
                if let Some(list) = args.next() {
                    screenshot::set_only(&list);
                }
            }
            // Hidden: run the whole screenshot tour in the light theme.
            "--light" => screenshot::force_light(),
            // Hidden: window size for the screenshot tour, e.g. 1080x680.
            "--size" => {
                let parsed = args.next().and_then(|v| {
                    v.split_once('x')
                        .and_then(|(w, h)| Some([w.parse().ok()?, h.parse().ok()?]))
                });
                shot_size = parsed.unwrap_or(shot_size);
            }
            "--version" | "-V" => {
                println!("gearwise {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!("gearwise [--version]");
                return Ok(());
            }
            _ => {}
        }
    }
    let size = if screenshot_dir.is_some() || hw_screenshot_dir.is_some() {
        shot_size
    } else {
        [1180.0, 780.0]
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Gearwise")
        .with_icon(icon::window_icon())
        .with_app_id("gearwise")
        .with_inner_size(size)
        .with_min_inner_size([1080.0, 660.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Gearwise",
        options,
        Box::new(|cc| {
            Ok(Box::new(app::App::new(
                cc,
                app::Options {
                    screenshot_dir,
                    library_path: hw_screenshot_dir.as_ref().map(|d| d.join("library.json")),
                    hardware: true,
                    hw_screenshot_dir,
                },
            )))
        }),
    )
}
