//! Helfer zur asynchronen Hintergrund-Verarbeitung mit Rückmeldung an den Slint UI-Thread.

use slint::ComponentHandle;

/// Führt eine Aufgabe `work` in einem separaten Hintergrund-Thread aus und übergibt das Ergebnis
/// sicher an den UI-Thread über `slint::invoke_from_event_loop`.
///
/// Wenn das Fenster vor Abschluss der Hintergrundaufgabe geschlossen wird (`handle.upgrade() == None`),
/// wird der Callback sicher ignoriert.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::spawn_ui_task;
///
/// spawn_ui_task(
///     ui.as_weak(),
///     || {
///         // Teure Hintergrundarbeit (z. B. Git-Scan, CSV-Parsing, Datei-Laden)
///         std::thread::sleep(std::time::Duration::from_millis(500));
///         "Fertig!".to_string()
///     },
///     |ui, ergebnis| {
///         // Läuft garantiert im UI-Thread:
///         ui.set_status_text(ergebnis.into());
///     },
/// );
/// ```
pub fn spawn_ui_task<W, T, Work, UiCb>(handle: slint::Weak<W>, work: Work, on_done: UiCb)
where
    W: ComponentHandle + 'static,
    T: Send + 'static,
    Work: FnOnce() -> T + Send + 'static,
    UiCb: FnOnce(&W, T) + Send + 'static,
{
    std::thread::spawn(move || {
        let result = work();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = handle.upgrade() {
                on_done(&ui, result);
            }
        });
    });
}

/// Führt eine Aufgabe aus, die fehlschlagen kann (`Result<T, E>`), und trennt Erfolgs- und Fehlerbehandlung.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::spawn_ui_task_result;
///
/// spawn_ui_task_result(
///     ui.as_weak(),
///     || std::fs::read_to_string("data.txt"),
///     |ui, content| ui.set_content(content.into()),
///     |ui, err| ui.set_error_message(format!("Fehler: {err}").into()),
/// );
/// ```
pub fn spawn_ui_task_result<W, T, E, Work, OkCb, ErrCb>(
    handle: slint::Weak<W>,
    work: Work,
    on_ok: OkCb,
    on_err: ErrCb,
) where
    W: ComponentHandle + 'static,
    T: Send + 'static,
    E: Send + 'static,
    Work: FnOnce() -> Result<T, E> + Send + 'static,
    OkCb: FnOnce(&W, T) + Send + 'static,
    ErrCb: FnOnce(&W, E) + Send + 'static,
{
    std::thread::spawn(move || {
        let result = work();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = handle.upgrade() {
                match result {
                    Ok(val) => on_ok(&ui, val),
                    Err(err) => on_err(&ui, err),
                }
            }
        });
    });
}

/// Vereinfachtes `slint::invoke_from_event_loop` mit automatischem `handle.upgrade()`.
pub fn invoke_ui<W, F>(handle: slint::Weak<W>, callback: F)
where
    W: ComponentHandle + 'static,
    F: FnOnce(&W) + Send + 'static,
{
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = handle.upgrade() {
            callback(&ui);
        }
    });
}
