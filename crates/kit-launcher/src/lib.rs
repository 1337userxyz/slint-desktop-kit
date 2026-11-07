//! `kit-launcher`: Cross-Platform System Launcher für Rust Desktop-Anwendungen.
//!
//! Stellt plattformübergreifende Funktionen bereit für:
//! - **Dateimanager**: Ordner öffnen oder Datei im Explorer/Dateimanager hervorheben (`reveal_in_file_manager`)
//! - **Terminal**: Terminal im gewählten Verzeichnis starten mit automatischer Erkennung (`open_in_terminal`)
//! - **URLs & Webbrowser**: URLs im Standard-Browser öffnen (`open_url`)
//! - **Externe Apps & Editoren**: Programme finden (`find_binary`) und mit Dateipfad-Platzhaltern (`{file}`) starten

use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Öffnet einen Pfad (Datei oder Ordner) in der Standardanwendung des Betriebssystems.
pub fn open_path(path: &Path) -> std::io::Result<Child> {
    let p = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(p.as_ref()).spawn()
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(p.as_ref()).spawn()
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(p.as_ref()).spawn()
    }
}

/// Öffnet den Dateimanager und hebt (falls es eine Datei ist) diese direkt hervor.
///
/// - **Windows**: `explorer.exe /select,"<Pfad>"`
/// - **macOS**: `open -R "<Pfad>"`
/// - **Linux**: Versucht DBus `org.freedesktop.FileManager1.ShowItems` oder öffnet das Elternverzeichnis.
pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<Child> {
    #[cfg(target_os = "windows")]
    {
        if path.is_file() {
            Command::new("explorer")
                .arg(format!("/select,{}", path.display()))
                .spawn()
        } else {
            Command::new("explorer").arg(path).spawn()
        }
    }

    #[cfg(target_os = "macos")]
    {
        if path.is_file() {
            Command::new("open").args(["-R", &path.to_string_lossy()]).spawn()
        } else {
            Command::new("open").arg(path).spawn()
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // Auf Linux: Falls Datei, übergeordnetes Verzeichnis mit xdg-open öffnen
        let target_dir = kit_paths::folder_or_parent(path);
        Command::new("xdg-open").arg(target_dir).spawn()
    }
}

/// Startet ein Terminal-Fenster im angegebenen Arbeitsverzeichnis.
///
/// Erkennt unter Linux automatisch installierte moderne & Standard-Terminals:
/// 1. Umgebungsvariable `$TERMINAL`
/// 2. `ghostty`, `foot`, `kitty`, `alacritty`, `wezterm`
/// 3. Desktop-Terminals: `gnome-terminal`, `konsole`, `xfce4-terminal`, `tilix`, `x-terminal-emulator`
///
/// Unter Windows wird bevorzugt `wt.exe` (Windows Terminal) oder PowerShell gestartet.
pub fn open_in_terminal(dir: &Path) -> std::io::Result<Child> {
    let p = dir.to_string_lossy();

    #[cfg(target_os = "windows")]
    {
        // Prüfen, ob Windows Terminal (wt.exe) vorhanden ist
        if find_in_path("wt.exe").is_some() {
            return Command::new("wt").args(["-d", p.as_ref()]).spawn();
        }
        Command::new("cmd")
            .args([
                "/C",
                "start",
                "powershell",
                "-NoExit",
                "-Command",
                &format!("Set-Location -LiteralPath '{}'", p),
            ])
            .spawn()
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").args(["-a", "Terminal", p.as_ref()]).spawn()
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        // 1. $TERMINAL prüfen
        if let Ok(term) = std::env::var("TERMINAL") {
            let term = term.trim();
            if !term.is_empty() {
                if let Ok(child) = Command::new(term).current_dir(dir).spawn() {
                    return Ok(child);
                }
            }
        }

        // 2. Bekannte Terminal-Kandidaten durchprobieren
        let candidates = [
            "ghostty",
            "foot",
            "kitty",
            "alacritty",
            "wezterm",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "tilix",
            "x-terminal-emulator",
            "xterm",
        ];

        for term in candidates {
            if find_in_path(term).is_some() {
                let res = match term {
                    "gnome-terminal" => Command::new(term)
                        .arg(format!("--working-directory={}", p))
                        .spawn(),
                    "konsole" => Command::new(term).args(["--workdir", p.as_ref()]).spawn(),
                    "kitty" => Command::new(term).args(["--directory", p.as_ref()]).spawn(),
                    _ => Command::new(term).current_dir(dir).spawn(),
                };
                if let Ok(child) = res {
                    return Ok(child);
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Kein geeignetes Terminal-Programm (ghostty, kitty, alacritty, gnome-terminal etc.) im PATH gefunden.",
        ))
    }
}

/// Öffnet eine Web-URL im Standardbrowser des Systems.
pub fn open_url(url: &str) -> std::io::Result<Child> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", "", url]).spawn()
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn()
    }
}

/// Öffnet einen Pfad im Editor (z. B. VS Code `code`).
pub fn open_in_editor(path: &Path, editor_command: Option<&str>) -> std::io::Result<Child> {
    let editor = editor_command.unwrap_or("code");
    let p = path.to_string_lossy();

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", editor, p.as_ref()]).spawn()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new(editor).arg(p.as_ref()).spawn()
    }
}

/// Führt ein externes Programm mit einem Dateipfad aus.
/// Unterstützt optional Platzhalter wie `{file}` in den Argumenten.
pub fn open_in_external_app(
    executable: &Path,
    target_path: &Path,
    arguments_template: Option<&str>,
) -> std::io::Result<Child> {
    let mut cmd = Command::new(executable);
    let target_str = target_path.to_string_lossy();

    if let Some(args_str) = arguments_template.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if args_str.contains("{file}") {
            for arg in args_str.split_whitespace() {
                if arg.contains("{file}") {
                    cmd.arg(arg.replace("{file}", &target_str));
                } else {
                    cmd.arg(arg);
                }
            }
        } else {
            for arg in args_str.split_whitespace() {
                cmd.arg(arg);
            }
            cmd.arg(target_str.as_ref());
        }
    } else {
        cmd.arg(target_str.as_ref());
    }

    cmd.spawn()
}

/// Sucht eine Binärdatei zuerst in einer Liste fester Pfade und anschließend im System-`PATH`.
pub fn find_binary(name: &str, candidates: &[&str]) -> Option<PathBuf> {
    // 1. Feste Pfade prüfen
    for &cand in candidates {
        let p = PathBuf::from(cand);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. PATH prüfen
    find_in_path(name)
}

/// Sucht eine Binärdatei im `PATH` (berücksichtigt unter Windows automatisch `.exe`).
pub fn find_in_path(binary_name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let direct = dir.join(binary_name);
        if direct.exists() && direct.is_file() {
            return Some(direct);
        }
        #[cfg(target_os = "windows")]
        {
            if !binary_name.ends_with(".exe") {
                let with_exe = dir.join(format!("{}.exe", binary_name));
                if with_exe.exists() && with_exe.is_file() {
                    return Some(with_exe);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_in_path() {
        // Auf jedem System sollte sh, bash, cargo oder cmd existieren
        #[cfg(not(target_os = "windows"))]
        {
            let sh = find_in_path("sh");
            assert!(sh.is_some(), "sh sollte im PATH gefunden werden");
        }
        #[cfg(target_os = "windows")]
        {
            let cmd = find_in_path("cmd.exe");
            assert!(cmd.is_some(), "cmd.exe sollte im PATH gefunden werden");
        }
    }

    #[test]
    fn test_find_binary_candidates() {
        // Nicht existierender Kandidat, aber Fallback auf PATH
        let candidates = ["/tmp/does_not_exist_xyz123", "C:\\does_not_exist_xyz"];
        #[cfg(not(target_os = "windows"))]
        let found = find_binary("sh", &candidates);
        #[cfg(target_os = "windows")]
        let found = find_binary("cmd", &candidates);

        assert!(found.is_some());
    }
}
