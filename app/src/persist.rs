//! Loading and saving the profile library.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use periph_core::profile::Library;

pub struct Loaded {
    pub lib: Library,
    /// Something the user should hear about (a corrupt file was set aside).
    pub notice: Option<String>,
}

/// Load the library; a corrupt file is moved aside and replaced by a fresh one.
pub fn load(path: &Path) -> Loaded {
    match Library::load(path) {
        Ok(lib) => Loaded { lib, notice: None },
        Err(e) => {
            log::warn!("cannot read {}: {e}", path.display());
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            let backup = path.with_extension(format!("json.corrupt-{stamp}"));
            let notice = match std::fs::rename(path, &backup) {
                Ok(()) => format!(
                    "Your settings file could not be read. It was saved as {} and a fresh one was started.",
                    backup.display()
                ),
                Err(_) => {
                    "Your settings file could not be read, so defaults are in use.".to_string()
                }
            };
            Loaded {
                lib: Library::starter(),
                notice: Some(notice),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use periph_core::profile::new_id;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("periph-studio-test-{}", new_id("t")));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn corrupt_file_is_backed_up_and_replaced() {
        let dir = temp_dir();
        let path = dir.join("library.json");
        std::fs::write(&path, b"{ not json").unwrap();
        let loaded = load(&path);
        assert!(loaded.notice.is_some());
        assert_eq!(loaded.lib.profiles.len(), 1);
        assert!(!path.exists());
        let backups: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(std::fs::read(backups[0].path()).unwrap(), b"{ not json");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn missing_file_loads_silently() {
        let dir = temp_dir();
        let loaded = load(&dir.join("library.json"));
        assert!(loaded.notice.is_none());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn saved_library_loads_back() {
        let dir = temp_dir();
        let path = dir.join("nested").join("library.json");
        let mut lib = Library::starter();
        lib.prefs.accent = "#8b5cf6".into();
        lib.save(&path).unwrap();
        assert_eq!(load(&path).lib, lib);
        std::fs::remove_dir_all(dir).ok();
    }
}
