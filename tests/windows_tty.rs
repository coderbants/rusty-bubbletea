//! Native Windows regressions for the platform-separated terminal facade.

#![cfg(windows)]

use rusty_bubbletea::{termios_unix, termios_windows, tty, tty_unix, tty_windows};

fn assert_explained(error: &dyn std::error::Error, context: &str) {
    assert!(!error.to_string().trim().is_empty(), "{context}");
}

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

#[test]
fn windows_terminal_operations_round_trip_when_a_console_is_available() {
    let enabled = tty::enable_raw_mode();
    match enabled {
        Ok(()) => {
            tty::disable_raw_mode().expect("a console that enables raw mode must restore it");
        }
        Err(error) => {
            assert_explained(
                &error,
                "raw-mode failure should explain why the host console is unavailable",
            );
        }
    }

    let initialized = tty::init_terminal();
    match initialized {
        Ok(()) => {
            tty::restore_terminal().expect("a console that initializes must restore");
        }
        Err(error) => {
            assert_explained(
                error.as_ref(),
                "terminal initialization failure should explain why the host console is unavailable",
            );
        }
    }

    match tty::get_window_size() {
        Ok((columns, rows)) => {
            assert!(
                columns > 0,
                "a Windows console must report positive columns"
            );
            assert!(rows > 0, "a Windows console must report positive rows");
        }
        Err(error) => assert_explained(
            error.as_ref(),
            "window-size failure should explain why the host console is unavailable",
        ),
    }
}
