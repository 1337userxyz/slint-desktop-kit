//! Gemeinsames Theme-System (`kit-theme`) für Slint-Desktop-Anwendungen.
//!
//! Stellt den Pfad zur Slint-Bibliothek (`@kit-theme`) sowie das [`ThemeMode`]-Enum
//! für die Konfiguration auf Rust-Seite bereit.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Gibt den absoluten Pfad zur Slint-Einstiegsdatei (`ui/index.slint`) zurück.
pub fn library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ui")
        .join("index.slint")
}

/// Gibt die Library-Paths-Map für `slint_build::CompilerConfiguration::with_library_paths` zurück.
///
/// Ermöglicht in `.slint`-Dateien:
/// ```slint
/// import { AppTheme, KitCard, KitBadge, KitThemeSelector } from "@kit-theme";
/// ```
pub fn library_paths() -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    map.insert("kit-theme".to_string(), library_path());
    map
}

/// Unterstützte Theme-Modi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    /// Gibt die kanonische String-Repräsentation (`"system"`, `"light"`, `"dark"`) zurück.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// Parst einen String tolerant; unbekannte Werte fallen auf [`ThemeMode::System`] zurück.
    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "light" | "hell" => Self::Light,
            "dark" | "dunkel" => Self::Dark,
            _ => Self::System,
        }
    }
}

impl FromStr for ThemeMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_roundtrip() {
        assert_eq!(ThemeMode::default(), ThemeMode::System);
        assert_eq!(ThemeMode::System.as_str(), "system");
        assert_eq!(ThemeMode::Light.as_str(), "light");
        assert_eq!(ThemeMode::Dark.as_str(), "dark");

        assert_eq!(ThemeMode::from_str_lossy("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str_lossy("LIGHT"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str_lossy("unknown"), ThemeMode::System);

        assert_eq!("dark".parse::<ThemeMode>(), Ok(ThemeMode::Dark));
        assert!("invalid".parse::<ThemeMode>().is_err());
        assert_eq!(ThemeMode::Dark.to_string(), "dark");
    }

    #[test]
    fn test_library_paths() {
        let p = library_path();
        assert!(p.ends_with("index.slint"));
        assert!(p.exists());
        let map = library_paths();
        assert_eq!(map.get("kit-theme"), Some(&p));
    }
}
