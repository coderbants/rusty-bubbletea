//! Native Windows regressions for the platform-separated terminal facade.

#![cfg(windows)]

use rusty_bubbletea::{termios_unix, termios_windows, tty, tty_unix, tty_windows};

#[test]
fn windows_selects_windows_terminal_modules() {
    assert!(tty_windows::is_windows_tty());
    assert!(termios_windows::is_windows_termios());
    assert!(!tty_unix::is_unix_tty());
    assert!(!termios_unix::is_unix_termios());
}

#[test]
fn windows_terminal_surface_has_safe_platform_neutral_signatures() {
    let _enable: fn() -> std::io::Result<()> = tty::enable_raw_mode;
    let _disable: fn() -> std::io::Result<()> = tty::disable_raw_mode;
    let _init: fn() -> Result<(), Box<dyn std::error::Error>> = tty::init_terminal;
    let _restore: fn() -> Result<(), Box<dyn std::error::Error>> = tty::restore_terminal;
    let _window_size: fn() -> Result<(u16, u16), Box<dyn std::error::Error>> = tty::get_window_size;
}
