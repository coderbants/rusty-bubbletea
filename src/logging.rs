//! Cleanroom Rust port of upstream Go source file: `logging.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Logging
//!
//! Logging utilities for writing debug logs to a file without interfering with terminal UI rendering.
//! </public-docs>

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::Mutex;

/// <upstream-comment>
/// LogToFile sets up default logging to log to a file.
/// </upstream-comment>
pub fn log_to_file(path: &str, prefix: &str) -> io::Result<FileLogger> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;

    let formatted_prefix = if !prefix.is_empty() && !prefix.ends_with(' ') {
        format!("{} ", prefix)
    } else {
        prefix.to_string()
    };

    Ok(FileLogger {
        file: Mutex::new(file),
        prefix: formatted_prefix,
    })
}

/// <upstream-comment>
/// FileLogger is a thread-safe file logger.
/// </upstream-comment>
pub struct FileLogger {
    file: Mutex<File>,
    prefix: String,
}

impl FileLogger {
    /// Logs a formatted message to the log file.
    pub fn log(&self, msg: &str) -> io::Result<()> {
        let mut f = self.file.lock().unwrap();
        writeln!(f, "{}{}", self.prefix, msg)
    }
}
