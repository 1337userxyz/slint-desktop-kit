//! Helfer für Slint UI-Timer (z. B. zeitgesteuertes Ausblenden von Statusmeldungen).

use std::time::Duration;
use slint::{ComponentHandle, Timer, TimerMode};

/// Startet einen Einmal-Timer (Single-Shot), der nach `duration` aufgerufen wird.
///
/// Typischer Anwendungsfall: Statusmeldung oder Toast-Nachricht nach 3-5 Sekunden automatisch leeren.
///
/// # Beispiel
/// ```rust,ignore
/// use kit_slint::timer::single_shot;
/// use std::time::Duration;
///
/// ui.set_status_message("Datei gespeichert!".into());
/// single_shot(ui.as_weak(), Duration::from_secs(4), |ui| {
///     ui.set_status_message("".into());
/// });
/// ```
pub fn single_shot<W, F>(handle: slint::Weak<W>, duration: Duration, callback: F) -> Timer
where
    W: ComponentHandle + 'static,
    F: FnOnce(&W) + 'static,
{
    let timer = Timer::default();
    let mut cb_opt = Some(callback);
    timer.start(TimerMode::SingleShot, duration, move || {
        if let Some(cb) = cb_opt.take() {
            if let Some(ui) = handle.upgrade() {
                cb(&ui);
            }
        }
    });
    timer
}
