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
```
