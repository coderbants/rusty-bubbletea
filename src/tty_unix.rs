//! Cleanroom Rust port of upstream Go source file: `tty_unix.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <user-docs>
//! # TTY (Unix)
//!
//! POSIX Unix TTY handle and raw mode initialization. The implementation is
//! compiled only for Unix targets so Windows never type-checks its termios or
//! raw-file-descriptor operations.
//! </user-docs>

/// Unix TTY initialization check.
pub fn is_unix_tty() -> bool {
    cfg!(unix)
}

#[cfg(unix)]
mod unix {
    use std::io;
    use std::sync::{Mutex, OnceLock};

    /// Saved terminal state for the raw mode toggle, mirroring the upstream
    /// `p.previousTtyInputState` (`term.MakeRaw`/`term.Restore`).
    static SAVED_TERMIOS: OnceLock<Mutex<Option<libc::termios>>> = OnceLock::new();

    fn saved_termios() -> &'static Mutex<Option<libc::termios>> {
        SAVED_TERMIOS.get_or_init(|| Mutex::new(None))
    }

    /// Restores a saved value and clears it only after restoration succeeds.
    ///
    /// Keeping the value on failure allows terminal cleanup to be retried
    /// instead of permanently losing the pre-raw-mode state.
    fn restore_saved_state<T>(
        saved: &mut Option<T>,
        restore: impl FnOnce(&T) -> io::Result<()>,
    ) -> io::Result<()> {
        if let Some(value) = saved.as_ref() {
            restore(value)?;
            saved.take();
        }
        Ok(())
    }

    /// Enables POSIX raw mode while preserving the upstream output behavior.
    pub(crate) fn enable_raw_mode() -> io::Result<()> {
        use std::os::fd::AsRawFd;

        let fd = io::stdin().as_raw_fd();
        let mut termios: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(fd, &mut termios) } != 0 {
            return Err(io::Error::last_os_error());
        }
        saved_termios()
            .lock()
            .map_err(|_| io::Error::other("saved terminal state lock poisoned"))?
            .replace(termios);

        // This replicates the behavior documented for cfmakeraw in the
        // termios(3) manpage, as x/term's makeRaw does. Only OPOST is cleared
        // from output flags so TABDLY retains the upstream hard-tab behavior.
        termios.c_iflag &= !(libc::IGNBRK
            | libc::BRKINT
            | libc::PARMRK
            | libc::ISTRIP
            | libc::INLCR
            | libc::IGNCR
            | libc::ICRNL
            | libc::IXON);
        termios.c_oflag &= !libc::OPOST;
        termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
        termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
        termios.c_cflag |= libc::CS8;
        termios.c_cc[libc::VMIN] = 1;
        termios.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Restores the terminal state saved by [`enable_raw_mode`].
    pub(crate) fn disable_raw_mode() -> io::Result<()> {
        use std::os::fd::AsRawFd;

        let mut saved = saved_termios()
            .lock()
            .map_err(|_| io::Error::other("saved terminal state lock poisoned"))?;
        let fd = io::stdin().as_raw_fd();
        restore_saved_state(&mut saved, |termios| {
            if unsafe { libc::tcsetattr(fd, libc::TCSANOW, termios) } != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        })
    }

    #[cfg(test)]
    mod tests {
        use super::restore_saved_state;
        use std::io;

        #[test]
        fn failed_restore_preserves_saved_state_for_retry() {
            let mut saved = Some(7_u8);

            let result = restore_saved_state(&mut saved, |_| {
                Err(io::Error::other("injected restore failure"))
            });

            assert!(result.is_err());
            assert_eq!(saved, Some(7));
        }

        #[test]
        fn successful_restore_clears_saved_state() {
            let mut saved = Some(7_u8);

            restore_saved_state(&mut saved, |value| {
                assert_eq!(*value, 7);
                Ok(())
            })
            .expect("injected terminal restoration should succeed");

            assert_eq!(saved, None);
        }
    }
}

#[cfg(unix)]
pub(crate) use unix::{disable_raw_mode, enable_raw_mode};
