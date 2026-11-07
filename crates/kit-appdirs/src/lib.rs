//! `kit-appdirs`: Plattformübergreifende Verzeichnispfade für Config, Cache, Data und Logs.
//!
//! Unterstützt automatischen Portable- und Entwickler-Modus (Dateien im Arbeitsverzeichnis
//! oder neben der Executable haben Vorrang, bevor auf Systemordner wie XDG oder AppData zurückgegriffen wird).

use std::path::PathBuf;

pub use directories::ProjectDirs;

/// Repräsentiert die Standardverzeichnisse für eine benannte Desktop-Anwendung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
    pub app_name: String,
    pub qualifier: String,
    pub organization: String,
}

impl AppDirs {
    /// Erstellt eine neue Instanz für die Anwendung mit Standard-Qualifiern.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            qualifier: String::new(),
            organization: String::new(),
        }
    }

    /// Setzt die Organisation (z. B. "Tristan").
    pub fn with_org(mut self, organization: impl Into<String>) -> Self {
        self.organization = organization.into();
        self
    }

    /// Ermittelt die Projekt-Verzeichnisse über die `directories`-Spezifikation.
    pub fn project_dirs(&self) -> Option<ProjectDirs> {
        ProjectDirs::from(&self.qualifier, &self.organization, &self.app_name)
    }

    /// Standard-Konfigurationsordner:
    /// - **Linux**: `~/.config/<app_name>` oder `$XDG_CONFIG_HOME/<app_name>`
    /// - **Windows**: `%APPDATA%\<organization>\<app_name>`
    /// - **macOS**: `~/Library/Application Support/<organization>.<app_name>`
    pub fn config_dir(&self) -> Option<PathBuf> {
        self.project_dirs().map(|d| d.config_dir().to_path_buf())
    }

    /// Standard-Datenordner:
    /// - **Linux**: `~/.local/share/<app_name>` oder `$XDG_DATA_HOME/<app_name>`
    /// - **Windows**: `%LOCALAPPDATA%\<organization>\<app_name>\data`
    /// - **macOS**: `~/Library/Application Support/<organization>.<app_name>`
    pub fn data_dir(&self) -> Option<PathBuf> {
        self.project_dirs().map(|d| d.data_dir().to_path_buf())
    }

    /// Standard-Cache-Ordner:
    /// - **Linux**: `~/.cache/<app_name>` oder `$XDG_CACHE_HOME/<app_name>`
    /// - **Windows**: `%LOCALAPPDATA%\<organization>\<app_name>\cache`
    /// - **macOS**: `~/Library/Caches/<organization>.<app_name>`
    pub fn cache_dir(&self) -> Option<PathBuf> {
        self.project_dirs().map(|d| d.cache_dir().to_path_buf())
    }

    /// Standard-Log-Ordner:
    /// - **Linux**: `~/.local/state/<app_name>/logs` oder Datenordner `/logs`
    /// - **Windows**: `%LOCALAPPDATA%\<organization>\<app_name>\logs`
    pub fn log_dir(&self) -> Option<PathBuf> {
        self.project_dirs().map(|d| {
            d.state_dir()
                .unwrap_or_else(|| d.data_dir())
                .join("logs")
        })
    }

    /// Löst eine Konfigurationsdatei mit Portable-/Entwickler-Priorität auf:
    /// 1. Prüft lokales Arbeitsverzeichnis (`./file_name`), falls existierend (Developer-/Portable-Modus)
    /// 2. Prüft neben der Executable (`exe_dir/file_name`), falls existierend
    /// 3. Fallback: OS-Konfigurationsordner (`config_dir/file_name`)
    ///
    /// Stellt sicher, dass das übergeordnete Verzeichnis existiert.
    pub fn resolve_config_file(&self, file_name: &str) -> PathBuf {
        if let Ok(cwd) = std::env::current_dir() {
            let local = cwd.join(file_name);
            if local.exists() {
                return local;
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let exe_local = parent.join(file_name);
                if exe_local.exists() {
                    return exe_local;
                }
            }
        }

        let path = self
            .config_dir()
            .map(|d| d.join(file_name))
            .unwrap_or_else(|| PathBuf::from(file_name));

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        path
    }

    /// Löst eine Cache-Datei im System-Cache-Ordner auf und erstellt das Verzeichnis bei Bedarf.
    pub fn resolve_cache_file(&self, file_name: &str) -> PathBuf {
        let path = self
            .cache_dir()
            .map(|d| d.join(file_name))
            .unwrap_or_else(|| PathBuf::from(file_name));

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        path
    }

    /// Löst eine Daten-Datei im System-Daten-Ordner auf und erstellt das Verzeichnis bei Bedarf.
    pub fn resolve_data_file(&self, file_name: &str) -> PathBuf {
        let path = self
            .data_dir()
            .map(|d| d.join(file_name))
            .unwrap_or_else(|| PathBuf::from(file_name));

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        path
    }
}

/// Schneller Zugriff auf das Konfigurationsverzeichnis einer Anwendung.
pub fn config_dir(app_name: &str) -> Option<PathBuf> {
    AppDirs::new(app_name).config_dir()
}

/// Schneller Zugriff auf das Datenverzeichnis einer Anwendung.
pub fn data_dir(app_name: &str) -> Option<PathBuf> {
    AppDirs::new(app_name).data_dir()
}

/// Schneller Zugriff auf das Cache-Verzeichnis einer Anwendung.
pub fn cache_dir(app_name: &str) -> Option<PathBuf> {
    AppDirs::new(app_name).cache_dir()
}

/// Löst eine Konfigurationsdatei für eine Anwendung portabel auf.
pub fn resolve_config(app_name: &str, file_name: &str) -> PathBuf {
    AppDirs::new(app_name).resolve_config_file(file_name)
}

/// Löst eine Cache-Datei für eine Anwendung auf.
pub fn resolve_cache(app_name: &str, file_name: &str) -> PathBuf {
    AppDirs::new(app_name).resolve_cache_file(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_dirs_resolution() {
        let dirs = AppDirs::new("testapp");
        let cfg = dirs.config_dir();
        assert!(cfg.is_some(), "config_dir sollte auf allen Ziel-OS auflösbar sein");
        let path_str = cfg.unwrap().to_string_lossy().to_lowercase();
        assert!(
            path_str.contains("testapp"),
            "Pfad muss App-Namen enthalten: {path_str}"
        );
    }

    #[test]
    fn test_resolve_config_file_fallback() {
        let dirs = AppDirs::new("testapp");
        let path = dirs.resolve_config_file("non_existent_random_config_123.json");
        assert!(path.to_string_lossy().contains("non_existent_random_config_123.json"));
    }
}
