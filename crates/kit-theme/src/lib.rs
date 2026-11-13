//! Gemeinsames Theme-System (`kit-theme`) für Slint-Desktop-Anwendungen.
//!
//! Stellt den Pfad zur Slint-Bibliothek (`@kit-theme`), das [`ThemeMode`]-Enum
//! sowie optional den standardisierten `build.rs`-Helper (`compile_slint`) bereit.

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

/// Standardisierter Slint-Build für alle Kit-Apps:
/// - Registriert `@kit-theme`
/// - Setzt `"fluent"` Style
/// - Setzt `cargo:rerun-if-changed` für `entry_slint` und `@kit-theme`
/// - Linkt unter Linux `dbus-1`
#[cfg(feature = "build")]
pub fn compile_slint(entry_slint: impl AsRef<std::path::Path>) {
    let entry = entry_slint.as_ref();
    println!("cargo:rerun-if-changed={}", entry.display());
    if let Some(ui_dir) = library_path().parent() {
        println!("cargo:rerun-if-changed={}", ui_dir.display());
    }

    let is_linux = std::env::var("CARGO_CFG_TARGET_OS")
        .map(|v| v == "linux")
        .unwrap_or(cfg!(target_os = "linux"));
    if is_linux {
        println!("cargo:rustc-link-lib=dbus-1");
    }

    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(library_paths())
        .with_style("fluent".into());
    slint_build::compile_with_config(entry, config)
        .expect("Slint compilation failed");
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

    #[test]
    fn test_slint_components_compile() {
        let tmp_dir = std::env::temp_dir().join(format!("kit-theme-compile-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let test_slint = tmp_dir.join("test_all.slint");
        std::fs::write(
            &test_slint,
            r##"
import {
    AppTheme,
    KitCard,
    KitBadge,
    KitThemeSelector,
    KitButton,
    KitDeleteButton,
    KitCheckBox,
    KitFilterChip,
    KitNavTab,
    KitHoverCard,
    KitComboItem,
    KitColorItem,
    KitEditableCombo,
    KitRichCombo,
    KitColorCombo,
    KitColSep,
    KitColResizer,
    KitFilterChoiceItem,
    KitColumnFilterModal
} from "@kit-theme";

export component TestAll inherits Window {
    width: 800px;
    height: 600px;
    background: AppTheme.bg-canvas;

    property <[KitComboItem]> rich-items: [{ value: "a", label: "A", badge: "1×" }];
    property <[KitColorItem]> color-items: [{ key: "Blue", hex_value: "#1d4ed8", color: #1d4ed8 }];
    property <[KitFilterChoiceItem]> filter-items: [{ value: "x", display-text: "X", count: 3, selected: true, visible: true }];
    property <length> col-w: 120px;

    VerticalLayout {
        KitThemeSelector {}
        KitCard {
            KitBadge { text: "OK"; kind: 1; }
        }
        KitButton { label: "Save"; primary: true; }
        KitDeleteButton {}
        KitDeleteButton { label: "✕ Entfernen"; }
        KitCheckBox { text: "Aktiv"; }
        KitFilterChip { label: "Alle"; active: true; }
        KitNavTab { label: "Übersicht"; active: true; badge-count: 2; }
        KitEditableCombo { options: ["One", "Two"]; clearable: true; show-count: true; }
        KitRichCombo { items: root.rich-items; clearable: true; show-count: true; }
        KitColorCombo { choices: root.color-items; }
        HorizontalLayout {
            KitColSep {}
            KitColResizer { target-width <=> root.col-w; }
        }
    }
    KitHoverCard { Text { text: "Hover"; } }
    KitColumnFilterModal { items: root.filter-items; }
}
"##,
        )
        .unwrap();

        let out_dir = tmp_dir.join("out");
        let _ = std::fs::create_dir_all(&out_dir);
        std::env::set_var("OUT_DIR", &out_dir);

        let config = slint_build::CompilerConfiguration::new()
            .with_library_paths(library_paths())
            .with_style("fluent".into());
        let res = slint_build::compile_with_config(&test_slint, config);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        res.expect("All @kit-theme Slint exports must compile cleanly");
    }
}
