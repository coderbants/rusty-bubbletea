//! Cleanroom Rust port of upstream Go source file: `tty.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <user-docs>
//! # TTY Terminal Management
//!
//! TTY initialization, input stream setup, raw mode toggles, and window dimension queries for Bubble Tea v2.0.8.
//! On Windows, raw-mode operations are compile-safe no-ops until the native
//! console adapter is completed; window-size queries remain available.
//! </user-docs>

use crossterm::terminal::size as term_size;
#[cfg(unix)]
use std::sync::{Mutex, OnceLock};

/// Saved terminal state for the raw mode toggle, mirroring the upstream
/// `p.previousTtyInputState` (x/term `MakeRaw`/`Restore`).
#[cfg(unix)]
static SAVED_TERMIOS: OnceLock<Mutex<Option<libc::termios>>> = OnceLock::new();

/// Initializes terminal raw mode, mirroring the upstream `initInput` ->
/// `term.MakeRaw` path (`tty_unix.go`). Unlike a fully-zeroed `cfmakeraw`,
/// only `OPOST` is cleared from the output flags, so `TABDLY` (and thus the
/// hard-tab cursor optimization) behaves exactly as it does upstream.
#[cfg(unix)]
pub fn enable_raw_mode() -> std::io::Result<()> {
    use std::os::fd::AsRawFd;
    let fd = std::io::stdin().as_raw_fd();
    let mut t: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(fd, &mut t) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    *SAVED_TERMIOS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap() = Some(t);

    // This attempts to replicate the behaviour documented for cfmakeraw in
    // the termios(3) manpage, as x/term's `makeRaw` does.
    t.c_iflag &= !(libc::IGNBRK
        | libc::BRKINT
        | libc::PARMRK
        | libc::ISTRIP
        | libc::INLCR
        | libc::IGNCR
        | libc::ICRNL
        | libc::IXON);
    t.c_oflag &= !libc::OPOST;
    t.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
    t.c_cflag &= !(libc::CSIZE | libc::PARENB);
    t.c_cflag |= libc::CS8;
    t.c_cc[libc::VMIN] = 1;
    t.c_cc[libc::VTIME] = 0;
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &t) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Keeps the raw-mode API available on Windows while native console mode
/// support remains deferred.
#[cfg(not(unix))]
pub fn enable_raw_mode() -> std::io::Result<()> {
    Ok(())
}

/// Restores the terminal state saved by [`enable_raw_mode`], mirroring the
/// upstream `term.Restore` path.
#[cfg(unix)]
pub fn disable_raw_mode() -> std::io::Result<()> {
    use std::os::fd::AsRawFd;
    let saved = SAVED_TERMIOS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .take();
    if let Some(t) = saved {
        let fd = std::io::stdin().as_raw_fd();
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &t) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

/// Keeps terminal restoration deterministic on Windows when no raw mode was
/// enabled by the compile-safe fallback.
#[cfg(not(unix))]
pub fn disable_raw_mode() -> std::io::Result<()> {
    Ok(())
}

/// Initializes terminal raw mode.
pub fn init_terminal() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    Ok(())
}

/// Restores terminal raw mode.
pub fn restore_terminal() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    Ok(())
}

/// Queries current window size columns and rows.
pub fn get_window_size() -> Result<(u16, u16), Box<dyn std::error::Error>> {
    let (w, h) = term_size()?;
    Ok((w, h))
}
