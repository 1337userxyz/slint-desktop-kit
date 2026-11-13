//! Automatischer Laufzeit-Preloader für Linux / NixOS.
//! Stellt sicher, dass Grafik- und Windowing-Bibliotheken (Wayland, OpenGL, X11, DBus)
//! aus dem `/nix/store` vorgeladen werden, damit Slint-Anwendungen ohne Wrapper starten können.

/// Stellt sicher, dass alle für die GUI benötigten dynamischen Bibliotheken geladen sind.
/// Sollte am Anfang der `main()`-Funktion vor Erzeugen des ersten Fensters aufgerufen werden.
pub fn ensure_gui_ready() {
    #[cfg(target_os = "linux")]
    linux_preload::ensure_ready();
}

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

        // 2. Auf NixOS im /nix/store nach fehlenden Bibliotheken suchen
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
