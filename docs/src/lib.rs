//! Bubble Tea lifecycle and configuration guide.
//!
//! <user-docs>
//! # Running a Program
//!
//! [`rusty_bubbletea::Program`] owns a model's event loop. Configure terminal
//! dimensions, environment, input, output, color profile, and cancellation
//! through [`rusty_bubbletea::ProgramOptions`]. For a runner moved to another
//! thread, obtain [`rusty_bubbletea::ProgramHandle`] before calling
//! [`rusty_bubbletea::Program::run`]. The handle can queue messages, request a
//! graceful `quit`, request an error `kill`, and wait for renderer cleanup.
//!
//! `with_input(None)` creates a deterministic headless program by disabling
//! input. `without_renderer()` selects the no-op renderer. A graceful quit
//! renders and flushes the final model view; cancellation, interruption, and
//! kill skip that final frame while still restoring terminal state.
//! </user-docs>
//!
//! Maintainer note: this file is the documentation anchor for the public
//! lifecycle contract. Implementation details belong in `src/program.rs` and
//! option semantics belong in `src/options.rs`.
