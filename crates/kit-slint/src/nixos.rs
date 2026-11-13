//! Automatischer Laufzeit-Preloader für Linux / NixOS.
//! Delegiert an [`kit_dialog::ensure_dialog_ready`], um sicherzustellen,
//! dass Grafik-, Windowing- und Portal-Bibliotheken (DBus, Wayland, X11, OpenGL)
//! vorgeladen sind, damit Slint-Anwendungen ohne Wrapper starten können.

/// Stellt sicher, dass alle für die GUI benötigten dynamischen Bibliotheken geladen sind.
/// Sollte am Anfang der `main()`-Funktion vor Erzeugen des ersten Fensters aufgerufen werden.
pub fn ensure_gui_ready() {
    kit_dialog::ensure_dialog_ready();
}
