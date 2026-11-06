# tristan-kit

Gemeinsames Toolkit für Rust + Slint Desktop-Anwendungen unter Linux und Windows.

## Enthaltene Crates

| Crate | Beschreibung | Features |
| :--- | :--- | :--- |
| **`kit-paths`** | Plattformübergreifende Pfadnormalisierung, Windows <-> Linux/macOS Mappings (`R:\` <-> `/mnt/r/` oder `/r/`) und Hilfsfunktionen. | `serde` |
| **`kit-picker`** | Non-blocking Datei- und Ordnerauswahldialoge mit `rfd`, Kontext-Gedächtnis ("Zuletzt gewählter Ordner für Zweck X") und direkter Slint-Event-Loop Integration. | `slint` (default) |

---

## Verwendung in Projekten

### In `Cargo.toml` einbinden
```toml
[dependencies]
kit-paths = { git = "https://github.com/1337userxyz/tristan-kit", tag = "v0.1.0" }
kit-picker = { git = "https://github.com/1337userxyz/tristan-kit", tag = "v0.1.0" }
```

### Lokale Entwicklung per `[patch]` (Bypass ohne Git-Push)
```toml
[patch."https://github.com/1337userxyz/tristan-kit"]
kit-paths = { path = "../tristan-kit/crates/kit-paths" }
kit-picker = { path = "../tristan-kit/crates/kit-picker" }
```

---

## Kurzanleitung `kit-picker` mit Slint

```rust
use kit_picker::PickRequest;

// Ordner wählen (asynchron, UI friert niemals ein!)
main_window.on_choose_folder({
    let ui_handle = main_window.as_weak();
    move || {
        PickRequest::folder()
            .title("Wurzelordner für Scan wählen")
            .context("scan_roots")
            .pick_into(ui_handle.clone(), |ui, folder| {
                ui.set_scanned_path(folder.display().to_string().into());
            });
    }
});

// CSV-Dateien öffnen
PickRequest::files()
    .title("CSV-Dateien auswählen")
    .context("csv_imports")
    .add_filter("CSV Dateien", &["csv", "tsv", "txt"])
    .pick_files_into(ui_handle, |ui, files| {
        // Dateien verarbeiten...
    });
```
