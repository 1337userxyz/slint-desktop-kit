//! `kit-paths`: Plattformübergreifende Pfadnormalisierung und Mappings (Windows <-> Linux/macOS).
//!
//! Löst Pfade zwischen verschiedenen Betriebssystemen auf (z. B. Samba-Shares, Dual-Boot,
//! `R:\` vs. `/r/` oder `/mnt/r/`).

use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use kit_appdirs as appdirs;

/// Betriebssystem-Typ zur expliziten Steuerung von Konvertierungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    Windows,
    Linux,
    MacOs,
}

impl Os {
    /// Erkennt das aktuelle Betriebssystem zur Laufzeit.
    pub fn current() -> Os {
        if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else {
            Os::Linux
        }
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Os::Windows)
    }

    pub fn is_unix(&self) -> bool {
        matches!(self, Os::Linux | Os::MacOs)
    }
}

/// Präfix-Mapping, z. B. `R:\` ↔ `/mnt/r/` oder `Z:\` ↔ `/shared/`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PathMapping {
    pub windows: String,
    pub unix: String,
}

impl PathMapping {
    pub fn new(windows: impl Into<String>, unix: impl Into<String>) -> Self {
        Self {
            windows: windows.into(),
            unix: unix.into(),
        }
    }
}

/// Normalisiert einen Pfad-String für Vergleiche (lowercase, Forward-Slashes).
pub fn norm(s: &str) -> String {
    s.replace('\\', "/").to_lowercase()
}

/// Gibt den übergeordneten Ordner zurück, oder den Pfad selbst, falls er bereits ein Ordner ist.
pub fn folder_or_parent(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else if let Some(parent) = path.parent() {
        if parent.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            parent.to_path_buf()
        }
    } else {
        path.to_path_buf()
    }
}

/// Wendet eine Liste von `PathMapping`s an, um einen Pfad von `os_from` nach `os_to` zu übersetzen.
pub fn map_from(os_from: Os, os_to: Os, path: &str, maps: &[PathMapping]) -> Option<String> {
    for m in maps {
        let (from, to) = match (os_from, os_to) {
            (Os::Windows, Os::Windows) | (Os::Windows, _) => (&m.windows, &m.unix),
            (_, Os::Windows) => (&m.unix, &m.windows),
            _ => continue,
        };
        let (np, nf) = (norm(path), norm(from));
        if np.starts_with(&nf) {
            let rest = &path[from.len()..];
            let rest = if os_to == Os::Windows {
                rest.replace('/', "\\")
            } else {
                rest.replace('\\', "/")
            };
            return Some(format!("{to}{rest}"));
        }
    }
    None
}

/// Versucht ohne explizites Mapping intelligente Pfad-Konvertierungen:
/// 1. Slashes umkehren (`\` <-> `/`)
/// 2. Windows-Laufwerksbuchstaben zu Unix (`R:\git\...` -> `/r/git/...`, `/mnt/r/git/...`, `/media/r/git/...`)
/// 3. Unix-Pfade zu Windows-Laufwerksbuchstaben (`/r/git/...`, `/mnt/r/git/...` -> `R:\git\...`)
pub fn auto_map_from(os_from: Os, os_to: Os, path: &str) -> Option<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    match (os_from, os_to) {
        (Os::Windows, Os::Linux) | (Os::Windows, Os::MacOs) => {
            let mut chars = trimmed.chars();
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            if c1.is_ascii_alphabetic() && c2 == ':' {
                let drive_lower = c1.to_ascii_lowercase();
                let drive_upper = c1.to_ascii_uppercase();
                let rest = trimmed[2..].replace('\\', "/");
                let rest_clean = rest.trim_start_matches('/');

                let candidates = [
                    format!("/{drive_lower}/{rest_clean}"),
                    format!("/mnt/{drive_lower}/{rest_clean}"),
                    format!("/media/{drive_lower}/{rest_clean}"),
                    format!("/{drive_upper}/{rest_clean}"),
                ];

                // Wenn ein Pfad tatsächlich im Dateisystem existiert -> direkte Auflösung
                for cand in &candidates {
                    if Path::new(cand).exists() {
                        return Some(PathBuf::from(cand));
                    }
                }

                return None;
            }

            let flipped = trimmed.replace('\\', "/");
            if Path::new(&flipped).exists() {
                Some(PathBuf::from(flipped))
            } else {
                None
            }
        }
        (Os::Linux, Os::Windows) | (Os::MacOs, Os::Windows) => {
            let flipped = trimmed.replace('/', "\\");
            if Path::new(&flipped).exists() {
                return Some(PathBuf::from(flipped));
            }

            let rest_after_mnt = trimmed.strip_prefix("/mnt/").or_else(|| trimmed.strip_prefix("/media/"));
            let (drive_char, rest_path) = if let Some(after_mnt) = rest_after_mnt {
                let mut parts = after_mnt.splitn(2, '/');
                let d = parts.next()?;
                if d.len() == 1 && d.chars().next()?.is_ascii_alphabetic() {
                    (d.chars().next()?, parts.next().unwrap_or(""))
                } else {
                    return None;
                }
            } else if trimmed.starts_with('/') {
                let mut parts = trimmed[1..].splitn(2, '/');
                let d = parts.next()?;
                if d.len() == 1 && d.chars().next()?.is_ascii_alphabetic() {
                    (d.chars().next()?, parts.next().unwrap_or(""))
                } else {
                    return None;
                }
            } else {
                return None;
            };

            let drive_upper = drive_char.to_ascii_uppercase();
            let win_rest = rest_path.replace('/', "\\");
            let win_path = format!("{drive_upper}:\\{win_rest}");
            if Path::new(&win_path).exists() {
                Some(PathBuf::from(win_path))
            } else {
                None
            }
        }
        _ => {
            let flipped = if os_to == Os::Windows {
                trimmed.replace('/', "\\")
            } else {
                trimmed.replace('\\', "/")
            };
            if Path::new(&flipped).exists() {
                Some(PathBuf::from(flipped))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norm() {
        assert_eq!(norm("R:\\Git\\Project"), "r:/git/project");
    }

    #[test]
    fn test_map_from() {
        let maps = vec![
            PathMapping::new("R:\\", "/r/"),
            PathMapping::new("Z:\\Shared", "/mnt/shared"),
        ];

        let unix = map_from(Os::Windows, Os::Linux, "R:\\repos\\test", &maps);
        assert_eq!(unix, Some("/r/repos/test".to_string()));

        let win = map_from(Os::Linux, Os::Windows, "/mnt/shared/data.csv", &maps);
        assert_eq!(win, Some("Z:\\Shared\\data.csv".to_string()));
    }

    #[test]
    fn test_auto_map() {
        // Bei Pfaden die nicht existieren, liefert auto_map_from None
        let win_path = "C:\\Projects\\repo_non_existent";
        let mapped = auto_map_from(Os::Windows, Os::Linux, win_path);
        assert_eq!(mapped, None);
    }
}
