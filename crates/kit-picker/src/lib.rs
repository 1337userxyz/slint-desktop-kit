//! `kit-picker`: Non-blocking Datei- und Ordnerpicker mit RFD,
//! Startpfad-Erkennung, Zweck-Gedächtnis ("Recent Paths") und Slint-Event-Loop Integration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Stellt sicher, dass auf Linux/NixOS die benötigten systemweiten Bibliotheken
/// (insbesondere `libdbus-1.so.3` für Portaldialoge sowie Wayland, X11, OpenGL für GUI) geladen sind.
/// Wird automatisch beim Öffnen von Dialogen ausgeführt, kann aber auch
/// explizit am Anfang von `main()` aufgerufen werden.
pub fn ensure_dialog_ready() {
    #[cfg(target_os = "linux")]
    linux_preload::ensure_ready();
}

/// Alias für Rückwärtskompatibilität.
pub use ensure_dialog_ready as ensure_linux_dialog_ready;

#[cfg(target_os = "linux")]
mod linux_preload {
    use std::collections::HashSet;
    use std::ffi::CString;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn ensure_ready() {
        INIT.call_once(preload);
    }

    struct LibTarget {
        names: &'static [&'static str],
        pkg_pattern: &'static str,
    }

    const TARGETS: &[LibTarget] = &[
        LibTarget { names: &["libdbus-1.so.3", "libdbus-1.so"], pkg_pattern: "dbus" },
        LibTarget { names: &["libwayland-client.so.0", "libwayland-client.so"], pkg_pattern: "wayland" },
        LibTarget { names: &["libwayland-cursor.so.0", "libwayland-cursor.so"], pkg_pattern: "wayland" },
        LibTarget { names: &["libwayland-egl.so.1", "libwayland-egl.so"], pkg_pattern: "wayland" },
        LibTarget { names: &["libxkbcommon.so.0", "libxkbcommon.so"], pkg_pattern: "libxkbcommon" },
        LibTarget { names: &["libGL.so.1", "libGL.so"], pkg_pattern: "libglvnd" },
        LibTarget { names: &["libEGL.so.1", "libEGL.so"], pkg_pattern: "libglvnd" },
        LibTarget { names: &["libX11.so.6", "libX11.so"], pkg_pattern: "libx11" },
        LibTarget { names: &["libXcursor.so.1", "libXcursor.so"], pkg_pattern: "libxcursor" },
        LibTarget { names: &["libXi.so.6", "libXi.so"], pkg_pattern: "libxi" },
        LibTarget { names: &["libXrandr.so.2", "libXrandr.so"], pkg_pattern: "libxrandr" },
    ];

    fn preload() {
        extern "C" {
            fn dlopen(
                filename: *const std::os::raw::c_char,
                flag: std::os::raw::c_int,
            ) -> *mut std::ffi::c_void;
        }
        const RTLD_NOW: std::os::raw::c_int = 2;
        const RTLD_GLOBAL: std::os::raw::c_int = 256;

        let mut loaded_indices = HashSet::new();

        // 1. Erst normal versuchen (falls im Standard-Suchpfad vorhanden)
        for (idx, target) in TARGETS.iter().enumerate() {
            for name in target.names {
                if let Ok(c_name) = CString::new(*name) {
                    unsafe {
                        let handle = dlopen(c_name.as_ptr(), RTLD_NOW | RTLD_GLOBAL);
                        if !handle.is_null() {
                            loaded_indices.insert(idx);
                            break;
                        }
                    }
                }
            }
        }

        if loaded_indices.len() == TARGETS.len() {
            return;
        }

        // 2. Auf NixOS liegen Bibliotheken isoliert im /nix/store
        if let Ok(entries) = std::fs::read_dir("/nix/store") {
            for entry in entries.flatten() {
                if loaded_indices.len() == TARGETS.len() {
                    break;
                }
                let path = entry.path();
                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                    for (idx, target) in TARGETS.iter().enumerate() {
                        if loaded_indices.contains(&idx) {
                            continue;
                        }
                        if folder_name.contains(target.pkg_pattern) {
                            let lib_dir = path.join("lib");
                            for so_name in target.names {
                                let so = lib_dir.join(so_name);
                                if so.exists() {
                                    if let Ok(c_so) = CString::new(so.to_string_lossy().as_bytes()) {
                                        unsafe {
                                            let handle = dlopen(c_so.as_ptr(), RTLD_NOW | RTLD_GLOBAL);
                                            if !handle.is_null() {
                                                loaded_indices.insert(idx);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Globales Gedächtnis für kürzlich verwendete Verzeichnisse je Kontext-Schlüssel (z. B. "clone_dest", "csv_import").
static RECENT_DIRS: OnceLock<Arc<Mutex<HashMap<String, PathBuf>>>> = OnceLock::new();

fn get_recent_store() -> &'static Arc<Mutex<HashMap<String, PathBuf>>> {
    RECENT_DIRS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

/// Speichert das zuletzt genutzte Verzeichnis für einen Kontext.
pub fn remember_dir(context_key: &str, path: &Path) {
    let dir = kit_paths::folder_or_parent(path);
    if let Ok(mut store) = get_recent_store().lock() {
        store.insert(context_key.to_string(), dir);
    }
}

/// Holt das zuletzt genutzte Verzeichnis für einen Kontext.
pub fn recall_dir(context_key: &str) -> Option<PathBuf> {
    get_recent_store()
        .lock()
        .ok()?
        .get(context_key)
        .cloned()
}

/// Art des Dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKind {
    SingleFile,
    MultipleFiles,
    Folder,
    SaveFile,
}

/// Dateityp-Filter für `rfd`.
#[derive(Debug, Clone)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Deklarative Konfiguration für einen Datei-/Ordner-Auswahldialog.
#[derive(Debug, Clone)]
pub struct PickRequest {
    pub kind: PickKind,
    pub title: Option<String>,
    pub start_hint: Option<PathBuf>,
    pub context_key: Option<String>,
    pub filters: Vec<FileFilter>,
    pub default_file_name: Option<String>,
    pub default_extension: Option<String>,
}

impl PickRequest {
    pub fn new(kind: PickKind) -> Self {
        Self {
            kind,
            title: None,
            start_hint: None,
            context_key: None,
            filters: Vec::new(),
            default_file_name: None,
            default_extension: None,
        }
    }

    pub fn folder() -> Self {
        Self::new(PickKind::Folder)
    }

    pub fn file() -> Self {
        Self::new(PickKind::SingleFile)
    }

    pub fn files() -> Self {
        Self::new(PickKind::MultipleFiles)
    }

    pub fn save() -> Self {
        Self::new(PickKind::SaveFile)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Setzt einen Kontext-Schlüssel (z. B. "clone_dest").
    /// Der Picker merkt sich automatisch das zuletzt gewählte Verzeichnis für diesen Schlüssel!
    pub fn context(mut self, key: impl Into<String>) -> Self {
        self.context_key = Some(key.into());
        self
    }

    /// Schlägt einen Startpfad vor. Wenn der Pfad eine Datei ist, wird deren Ordner genommen.
    pub fn start_hint(mut self, hint: impl AsRef<Path>) -> Self {
        let p = hint.as_ref();
        if !p.as_os_str().is_empty() {
            self.start_hint = Some(kit_paths::folder_or_parent(p));
        }
        self
    }

    /// Fügt einen Dateifilter hinzu (z. B. `add_filter("CSV Dateien", &["csv", "tsv"])`).
    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter {
            name: name.into(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    pub fn default_name(mut self, name: impl Into<String>) -> Self {
        self.default_file_name = Some(name.into());
        self
    }

    /// Automatische Dateiendung (z. B. "ods" oder "csv"), falls der Nutzer beim Speichern keine Endung eingibt.
    pub fn default_extension(mut self, ext: impl Into<String>) -> Self {
        self.default_extension = Some(ext.into());
        self
    }

    /// Ermittelt das optimale Startverzeichnis (erst `start_hint`, dann `context_key`, sonst Fallback).
    fn resolve_start_dir(&self) -> Option<PathBuf> {
        if let Some(ref hint) = self.start_hint {
            if hint.exists() {
                return Some(hint.clone());
            }
        }
        if let Some(ref key) = self.context_key {
            if let Some(dir) = recall_dir(key) {
                if dir.exists() {
                    return Some(dir);
                }
            }
        }
        None
    }

    /// Baut den vorkonfigurierten `rfd::FileDialog`.
    fn build_rfd(&self) -> rfd::FileDialog {
        ensure_dialog_ready();
        let mut dialog = rfd::FileDialog::new();

        if let Some(ref title) = self.title {
            dialog = dialog.set_title(title);
        }

        if let Some(start) = self.resolve_start_dir() {
            dialog = dialog.set_directory(start);
        }

        if let Some(ref name) = self.default_file_name {
            dialog = dialog.set_file_name(name);
        }

        for f in &self.filters {
            let ext_slices: Vec<&str> = f.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&f.name, &ext_slices);
        }

        dialog
    }

    /// Führt die Auswahl synchron auf dem aufrufenden Thread aus.
    pub fn pick_sync(&self) -> Option<PathBuf> {
        let mut res = match self.kind {
            PickKind::SingleFile => self.build_rfd().pick_file(),
            PickKind::MultipleFiles => self.build_rfd().pick_files().and_then(|mut v| v.pop()),
            PickKind::Folder => self.build_rfd().pick_folder(),
            PickKind::SaveFile => self.build_rfd().save_file(),
        };

        if let (Some(ref mut path), Some(ref ext)) = (&mut res, &self.default_extension) {
            if path.extension().is_none() {
                path.set_extension(ext);
            }
        }

        if let (Some(ref path), Some(ref key)) = (&res, &self.context_key) {
            remember_dir(key, path);
        }
        res
    }

    /// Führt die Mehrfachauswahl synchron aus.
    pub fn pick_files_sync(&self) -> Option<Vec<PathBuf>> {
        let res = self.build_rfd().pick_files();
        if let (Some(paths), Some(ref key)) = (&res, &self.context_key) {
            if let Some(first) = paths.first() {
                remember_dir(key, first);
            }
        }
        res
    }

    /// Führt die Auswahl in einem separaten Hintergrund-Thread aus (Non-Blocking!).
    pub fn pick_async<F>(self, callback: F)
    where
        F: FnOnce(Option<PathBuf>) + Send + 'static,
    {
        std::thread::spawn(move || {
            let res = self.pick_sync();
            callback(res);
        });
    }

    /// Führt die Mehrfachauswahl in einem separaten Hintergrund-Thread aus.
    pub fn pick_files_async<F>(self, callback: F)
    where
        F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static,
    {
        std::thread::spawn(move || {
            let res = self.pick_files_sync();
            callback(res);
        });
    }
}

// -------------------------------------------------------------------------------------------------
// Slint Integration
// -------------------------------------------------------------------------------------------------

#[cfg(feature = "slint")]
impl PickRequest {
    /// Startet den Picker non-blocking im Hintergrund und ruft bei Erfolg den Slint-UI-Callback
    /// sicher über `invoke_from_event_loop` auf.
    ///
    /// # Beispiel
    /// ```rust,ignore
    /// PickRequest::folder()
    ///     .title("Zielordner wählen")
    ///     .context("clone_dest")
    ///     .pick_into(ui.as_weak(), |ui, path| {
    ///         ui.set_dest_path(path.display().to_string().into());
    ///     });
    /// ```
    pub fn pick_into<W, F>(self, handle: slint::Weak<W>, callback: F)
    where
        W: slint::ComponentHandle + 'static,
        F: FnOnce(&W, PathBuf) + Send + 'static,
    {
        self.pick_async(move |res| {
            if let Some(path) = res {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = handle.upgrade() {
                        callback(&ui, path);
                    }
                });
            }
        });
    }

    /// Für Mehrfachauswahl von Dateien direkt in die Slint UI.
    pub fn pick_files_into<W, F>(self, handle: slint::Weak<W>, callback: F)
    where
        W: slint::ComponentHandle + 'static,
        F: FnOnce(&W, Vec<PathBuf>) + Send + 'static,
    {
        self.pick_files_async(move |res| {
            if let Some(paths) = res {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = handle.upgrade() {
                        callback(&ui, paths);
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_store() {
        let fake_path = PathBuf::from("/test/dir/sub/file.csv");
        remember_dir("csv_import", &fake_path);
        let remembered = recall_dir("csv_import");
        assert_eq!(remembered, Some(PathBuf::from("/test/dir/sub")));
    }

    #[test]
    fn test_pick_request_builder() {
        let req = PickRequest::file()
            .title("Test File")
            .context("test_ctx")
            .default_extension("ods")
            .add_filter("CSV", &["csv"]);
        assert_eq!(req.kind, PickKind::SingleFile);
        assert_eq!(req.title, Some("Test File".to_string()));
        assert_eq!(req.context_key, Some("test_ctx".to_string()));
        assert_eq!(req.default_extension, Some("ods".to_string()));
        assert_eq!(req.filters.len(), 1);
    }
}
