#![cfg(windows)]

//! Windows regression coverage for the compile-safe terminal boundary.

use rusty_bubbletea::tty::{disable_raw_mode, enable_raw_mode, init_terminal, restore_terminal};

#[test]
fn raw_mode_operations_are_safe_no_ops_on_windows() {
    enable_raw_mode().expect("Windows raw-mode fallback should succeed");
    disable_raw_mode().expect("Windows raw-mode restoration fallback should succeed");
}

#[test]
fn terminal_lifecycle_uses_the_same_windows_boundary() {
    init_terminal().expect("Windows terminal initialization fallback should succeed");
    restore_terminal().expect("Windows terminal restoration fallback should succeed");
}
