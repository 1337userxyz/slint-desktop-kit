//! Slint-Integration für [`kit_dialog::PickRequest`].

use std::path::PathBuf;

/// Erweiterungs-Trait für [`kit_dialog::PickRequest`] zur direkten Rückgabe
/// der ausgewählten Pfade in die Slint-UI-Event-Loop.
pub trait PickRequestSlintExt {
    /// Startet den Picker non-blocking im Hintergrund und ruft bei Erfolg den Slint-UI-Callback
    /// sicher über `slint::invoke_from_event_loop` auf.
    fn pick_into<W, F>(self, handle: slint::Weak<W>, callback: F)
    where
        W: slint::ComponentHandle + 'static,
        F: FnOnce(&W, PathBuf) + Send + 'static;

    /// Für Mehrfachauswahl von Dateien direkt in die Slint-UI.
    fn pick_files_into<W, F>(self, handle: slint::Weak<W>, callback: F)
    where
        W: slint::ComponentHandle + 'static,
        F: FnOnce(&W, Vec<PathBuf>) + Send + 'static;
}

impl PickRequestSlintExt for kit_dialog::PickRequest {
    fn pick_into<W, F>(self, handle: slint::Weak<W>, callback: F)
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

    fn pick_files_into<W, F>(self, handle: slint::Weak<W>, callback: F)
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
