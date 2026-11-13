# slint-desktop-kit

Modulares, plattformübergreifendes Toolkit für Rust & Slint Desktop-Anwendungen unter Linux und Windows.

## Enthaltene Crates

| Crate | Beschreibung | Features |
| :--- | :--- | :--- |
| **`kit-paths`** | Plattformübergreifende Pfadnormalisierung, Windows ↔ Linux/macOS Mappings (`R:\` ↔ `/r/` oder `/mnt/r/`) und Hilfsfunktionen. | `serde` |
| **`kit-picker`** | Non-blocking Datei- und Ordnerauswahldialoge mit `rfd`, Kontext-Gedächtnis ("Zuletzt gewählter Ordner für Zweck X"), automatischer NixOS/Linux DBus-Preloader und direkter Slint-Integration. | `slint` (default) |
| **`kit-slint`** | Produktivitäts-Helfer für Slint: `ToSlintModel` (`vec.to_slint_model()`), Iterator-Extension (`iter.collect_slint()`), `slint_model!`, Hintergrund-Tasks (`spawn_ui_task`), Timer (`single_shot`) und Farb-/Theme-Helfer (`parse_hex`, `ColorCache`). | Keine |
| **`kit-logging`** | In-Memory Ringpuffer + File-Logger, vollständig an die offizielle Rust `log`-Facade angebunden. Logs landen sowohl in einer Datei als auch in der Slint UI-Konsole. | Keine |
| **`kit-launcher`** | Cross-Platform System Launcher: Dateien im Explorer/Dateimanager hervorheben (`reveal_in_file_manager`), Terminals im Arbeitsverzeichnis starten (`open_in_terminal`), URLs und externe Werkzeuge. | Keine |
| **`kit-appdirs`** | Standard-Verzeichnisse für Konfiguration, Daten, Cache und Logs (XDG, AppData) mit automatischem Entwickler- & Portable-Modus. | Keine |
| **`kit-theme`** | Einheitliches Slint-Theme-System (Hell/Dunkel/System) mit semantischen Tokens (`AppTheme`), Basis-Komponenten (`KitCard`, `KitBadge`, `KitThemeSelector`) und `ThemeMode`. | `serde` |

---

## Verwendung in Projekten

### In `Cargo.toml` einbinden
```toml
[dependencies]
kit-paths = { git = "https://github.com/1337userxyz/slint-desktop-kit", tag = "v0.1.0" }
kit-picker = { git = "https://github.com/1337userxyz/slint-desktop-kit", tag = "v0.1.0" }
kit-slint = { git = "https://github.com/1337userxyz/slint-desktop-kit", tag = "v0.1.0" }
kit-logging = { git = "https://github.com/1337userxyz/slint-desktop-kit", tag = "v0.1.0" }
kit-launcher = { git = "https://github.com/1337userxyz/slint-desktop-kit", tag = "v0.1.0" }
kit-appdirs = { git = "https://github.com/1337userxyz/slint-desktop-kit", tag = "v0.1.0" }
```

### Lokale Entwicklung per `[patch]` oder Pfad-Abhängigkeit
```toml
[patch."https://github.com/1337userxyz/slint-desktop-kit"]
kit-paths = { path = "../slint-desktop-kit/crates/kit-paths" }
kit-picker = { path = "../slint-desktop-kit/crates/kit-picker" }
kit-slint = { path = "../slint-desktop-kit/crates/kit-slint" }
kit-logging = { path = "../slint-desktop-kit/crates/kit-logging" }
kit-launcher = { path = "../slint-desktop-kit/crates/kit-launcher" }
kit-appdirs = { path = "../slint-desktop-kit/crates/kit-appdirs" }
kit-theme = { path = "../slint-desktop-kit/crates/kit-theme" }
```

### Theme einbinden (`kit-theme`)

```toml
# Cargo.toml
[build-dependencies]
slint-build = "1.18"
kit-theme = { path = "../slint-desktop-kit/crates/kit-theme" }
```

```rust
// build.rs
let config = slint_build::CompilerConfiguration::new()
    .with_library_paths(kit_theme::library_paths());
slint_build::compile_with_config("ui/app.slint", config).unwrap();
```

```slint
// in jeder .slint-Datei
import { AppTheme, KitCard, KitBadge, KitThemeSelector } from "@kit-theme";
```

#### Verfügbare Tokens (`AppTheme`)
- **Hintergründe:** `bg-canvas` (Fenster), `bg-surface` (Karten/Panels), `bg-elevated` (Popups/Dialoge), `bg-subtle` (Zebra/Header), `bg-hover`, `bg-selected`, `bg-scrim` (Overlay).
- **Rahmen:** `border`, `border-strong`.
- **Text:** `text-primary`, `text-secondary`, `text-muted`.
- **Akzent & Status:** `accent`, `accent-hover`, `accent-bg`, `accent-fg`, `success`/`success-bg`, `warning`/`warning-bg`, `danger`/`danger-bg`, `info`/`info-bg`.
- **Layout:** `radius-sm`/`md`/`lg`, `spacing-xs`/`sm`/`md`/`lg`, `font-size-sm`/`md`/`lg`/`xl`.

Details und Regeln siehe [crates/kit-theme/README.md](crates/kit-theme/README.md).
