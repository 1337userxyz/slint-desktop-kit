//! `kit-logging`: In-Memory Ringpuffer + File-Logger für Slint Desktop-Anwendungen,
//! vollständig integriert mit der offiziellen Rust `log`-Facade.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

/// Log-Stufen kompatibel mit Rust `log::Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl From<Level> for LogLevel {
    fn from(l: Level) -> Self {
        match l {
            Level::Error => LogLevel::Error,
            Level::Warn => LogLevel::Warn,
            Level::Info => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
            Level::Trace => LogLevel::Trace,
        }
    }
}

/// Ein einzelner Log-Eintrag für UI-Anzeige oder Formatierung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

/// Der zentrale Logger mit In-Memory Puffer und optionaler Datei-Ausgabe.
pub struct AppLogger {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    max_entries: usize,
    log_file: Option<PathBuf>,
}

impl AppLogger {
    pub fn new(log_file: Option<PathBuf>, max_entries: usize) -> Self {
        if let Some(ref path) = log_file {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let _ = writeln!(f, "\n=== Session gestartet: {} ===", now);
            }
        }
        Self {
            entries: Arc::new(Mutex::new(Vec::with_capacity(max_entries))),
            max_entries,
            log_file,
        }
    }

    /// Fügt einen Log-Eintrag direkt hinzu (thread-safe).
    pub fn add(&self, level: LogLevel, target: &str, message: &str) {
        let now = Local::now();
        let entry = LogEntry {
            timestamp: now.format("%H:%M:%S%.3f").to_string(),
            level,
            target: target.to_string(),
            message: message.to_string(),
        };

        // In Log-Datei schreiben (falls konfiguriert)
        if let Some(ref path) = self.log_file {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let full_ts = now.format("%Y-%m-%d %H:%M:%S%.3f");
                let _ = writeln!(file, "[{}] [{}] {}: {}", full_ts, level.as_str(), target, message);
            }
        }

        // In den Ringpuffer für die UI schreiben
        if let Ok(mut lock) = self.entries.lock() {
            if lock.len() >= self.max_entries {
                lock.remove(0);
            }
            lock.push(entry);
        }
    }

    /// Holt alle Einträge, optional gefiltert nach Mindest-Stufe.
    pub fn get_entries(&self, min_level: Option<LogLevel>) -> Vec<LogEntry> {
        if let Ok(lock) = self.entries.lock() {
            match min_level {
                None => lock.clone(),
                Some(min) => lock.iter().filter(|e| e.level >= min).cloned().collect(),
            }
        } else {
            Vec::new()
        }
    }

    /// Formatiert die Log-Einträge als mehrzeiligen String.
    pub fn format_text(&self, min_level: Option<LogLevel>) -> String {
        self.get_entries(min_level)
            .iter()
            .map(|e| format!("[{}] [{}] {}: {}", e.timestamp, e.level.as_str(), e.target, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Leert den In-Memory Puffer.
    pub fn clear(&self) {
        if let Ok(mut lock) = self.entries.lock() {
            lock.clear();
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Implementierung des offiziellen Rust `log::Log` Traits
// -------------------------------------------------------------------------------------------------

impl Log for AppLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = LogLevel::from(record.level());
            let target = record.target();
            let msg = format!("{}", record.args());
            self.add(level, target, &msg);
        }
    }

    fn flush(&self) {}
}

// -------------------------------------------------------------------------------------------------
// Globale Initialisierung und Hilfsfunktionen
// -------------------------------------------------------------------------------------------------

static GLOBAL_LOGGER: OnceLock<AppLogger> = OnceLock::new();

/// Initialisiert den globalen Logger und registriert ihn bei der `log`-Facade.
pub fn init_logger(
    log_file: Option<PathBuf>,
    max_entries: usize,
    max_level: LevelFilter,
) -> Result<&'static AppLogger, SetLoggerError> {
    let logger = GLOBAL_LOGGER.get_or_init(|| AppLogger::new(log_file, max_entries));
    log::set_logger(logger)?;
    log::set_max_level(max_level);
    Ok(logger)
}

/// Holt den globalen Logger (erzeugt einen Standard-Logger, falls noch nicht initiiert).
pub fn get_logger() -> &'static AppLogger {
    GLOBAL_LOGGER.get_or_init(|| AppLogger::new(None, 1000))
}

pub fn log_info(target: &str, msg: &str) {
    get_logger().add(LogLevel::Info, target, msg);
}

pub fn log_warn(target: &str, msg: &str) {
    get_logger().add(LogLevel::Warn, target, msg);
}

pub fn log_error(target: &str, msg: &str) {
    get_logger().add(LogLevel::Error, target, msg);
}

pub fn log_debug(target: &str, msg: &str) {
    get_logger().add(LogLevel::Debug, target, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let logger = AppLogger::new(None, 3);
        logger.add(LogLevel::Info, "T1", "msg 1");
        logger.add(LogLevel::Info, "T2", "msg 2");
        logger.add(LogLevel::Warn, "T3", "msg 3");
        assert_eq!(logger.get_entries(None).len(), 3);

        // 4. Eintrag schiebt den ersten Eintrag heraus
        logger.add(LogLevel::Error, "T4", "msg 4");
        let entries = logger.get_entries(None);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].message, "msg 2");
        assert_eq!(entries[2].message, "msg 4");

        // Filter nach Warn oder höher
        let warn_or_above = logger.get_entries(Some(LogLevel::Warn));
        assert_eq!(warn_or_above.len(), 2);
    }
}
