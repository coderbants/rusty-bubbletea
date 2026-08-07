//! Cleanroom Rust port of upstream Go source file: `logging.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Logging Utilities
//!
//! Logging helpers (`log_to_file`, `FileLogger`) for logging to file without corrupting the TUI.
//! </public-docs>

use std::fs::{File, OpenOptions};
use std::io::Write;

/// FileLogger utility struct.
pub struct FileLogger {
    file: File,
    prefix: String,
}

impl FileLogger {
    /// Creates a new FileLogger.
    pub fn new(path: &str, prefix: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let mut pref = prefix.to_string();
        if !pref.is_empty() && !pref.ends_with(' ') {
            pref.push(' ');
        }
        Ok(Self { file, prefix: pref })
    }

    /// Logs a string line.
    pub fn log(&mut self, s: &str) {
        let _ = writeln!(self.file, "{}{}", self.prefix, s);
    }
}

/// Helper function to log to a file.
pub fn log_to_file(path: &str, prefix: &str) -> Result<FileLogger, std::io::Error> {
    FileLogger::new(path, prefix)
}
