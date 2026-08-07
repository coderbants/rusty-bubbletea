# Upstream Go File Mapping: `charming-bubbletea`

Target Upstream Tag: `charm.land/bubbletea/v2@v2.0.8`

| Upstream Go File (v2.0.8) | Dedicated Rust Port File | Status | Notes / Description |
| :--- | :--- | :--- | :--- |
| `tea.go` | `src/lib.rs`, `src/view.rs`, `src/program.rs` | Ported & Tested | Core Elm architecture Model, Msg, Cmd, View, Program |
| `tea_test.go` | `tests/tea_test.rs` | Ported & Tested | Core program unit tests |
| `clipboard.go` | `src/clipboard.rs` | Ported & Tested | Dedicated module for OSC52 system/primary clipboard operations (`set_clipboard`, `read_clipboard`, `ClipboardMsg`) |
| `color.go` | `src/color.rs` | Ported & Tested | Dedicated module for color requests (`request_background_color`, `request_foreground_color`, `request_cursor_color`) and color messages |
| `commands.go` | `src/commands.rs` | Ported & Tested | Built-in commands (`quit`, `batch`, `sequence`, `tick`, `every`, `request_window_size`) |
| `commands_test.go` | `tests/commands_test.rs` | Ported & Tested | Command suite integration tests |
| `cursed_renderer.go` | `src/cursed_renderer.rs` | Ported & Tested | High-performance CursedRenderer port managing declarative View frames, ANSI diffing, and unmanaged lines |
| `cursed_renderer_test.go` | `tests/tea_test.rs` | Ported & Tested | Renderer test cases |
| `cursor.go` | `src/cursor.rs` | Ported & Tested | Dedicated module for Cursor position, CursorShape, and `request_cursor_position` |
| `environ.go` | `src/environ.rs` | Ported & Tested | Dedicated module for `EnvMsg` environment variables |
| `exec.go` | `src/exec.rs` | Ported & Tested | `exec_process` external process execution |
| `exec_test.go` | `tests/commands_test.rs` | Ported & Tested | Exec process unit tests |
| `focus.go` | `src/focus.rs` | Ported & Tested | Dedicated `FocusMsg` & `BlurMsg` terminal focus events |
| `input.go` | `src/input.rs` | Ported & Tested | Dedicated input event translation module |
| `key.go` | `src/key.rs` | Ported & Tested | Key, KeyPressMsg, KeyReleaseMsg, KeyMsg enum/interface, spacebar formatting |
| `key_test.go` | `tests/key_test.rs` | Ported & Tested | Keyboard event suite tests |
| `keyboard.go` | `src/keyboard.rs` | Ported & Tested | Dedicated module for `KeyboardEnhancementsMsg` and Kitty keyboard protocol flags |
| `logging.go` | `src/logging.rs` | Ported & Tested | Dedicated file logger utility (`log_to_file`, `FileLogger`) |
| `logging_test.go` | `tests/tea_test.rs` | Ported & Tested | Logging unit test |
| `mod.go` | `src/mod_keys.rs` | Ported & Tested | Dedicated module for modifier constants (`MOD_SHIFT`, `MOD_ALT`, `MOD_CTRL`, etc.) |
| `mouse.go` | `src/mouse.rs` | Ported & Tested | MouseButton, Mouse struct, MouseClickMsg, MouseReleaseMsg, MouseWheelMsg, MouseMotionMsg |
| `mouse_test.go` | `tests/mouse_test.rs` | Ported & Tested | Mouse event suite tests |
| `nil_renderer.go` | `src/nil_renderer.rs` | Ported & Tested | Dedicated no-op testing renderer implementation |
| `options.go` | `src/options.rs` | Ported & Tested | `ProgramOptions` configuration constructors (`with_fps`, `without_renderer`, `with_filter`, `with_window_size`) |
| `options_test.go` | `tests/commands_test.rs` | Ported & Tested | Program option tests |
| `paste.go` | `src/paste.rs` | Ported & Tested | Dedicated module for bracketed paste messages (`PasteMsg`, `PasteStartMsg`, `PasteEndMsg`) |
| `profile.go` | `src/profile.rs` | Ported & Tested | Dedicated module for `ColorProfileMsg` terminal color profiles |
| `raw.go` | `src/raw.rs` | Ported & Tested | Dedicated module for `raw` command sending direct ANSI escape sequences |
| `renderer.go` | `src/renderer.rs` | Ported & Tested | Dedicated `Renderer` trait definition |
| `screen.go` | `src/screen.rs` | Ported & Tested | Dedicated module for `WindowSizeMsg`, `clear_screen`, `ModeReportMsg` |
| `screen_test.go` | `tests/commands_test.rs` | Ported & Tested | Screen buffer tests |
| `signals_unix.go` | `src/signals_unix.rs` | Ported & Tested | Dedicated `listen_for_resize` SIGWINCH resize listener module |
| `signals_windows.go` | `src/signals_windows.rs` | Ported & Tested | Dedicated Windows console signal listener module |
| `termcap.go` | `src/termcap.rs` | Ported & Tested | Dedicated module for `request_capability` XTGETTCAP query & `CapabilityMsg` |
| `termios_bsd.go` | `src/termios_bsd.rs` | Ported & Tested | Dedicated BSD termios helper module |
| `termios_other.go` | `src/termios_other.rs` | Ported & Tested | Dedicated non-POSIX termios fallback helper module |
| `termios_unix.go` | `src/termios_unix.rs` | Ported & Tested | Dedicated POSIX Unix termios helper module |
| `termios_windows.go` | `src/termios_windows.rs` | Ported & Tested | Dedicated Windows console mode termios helper module |
| `tty.go` | `src/tty.rs` | Ported & Tested | `init_terminal` & `restore_terminal` state helpers |
| `tty_unix.go` | `src/tty_unix.rs` | Ported & Tested | Dedicated Unix TTY reader and raw mode initialization module |
| `tty_windows.go` | `src/tty_windows.rs` | Ported & Tested | Dedicated Windows VT console mode helpers module |
| `xterm.go` | `src/xterm.rs` | Ported & Tested | Dedicated module for `request_terminal_version` XTVERSION query & `TerminalVersionMsg` |
| `LICENSE` | `LICENSE` | Ported & Tested | MIT License (matching upstream copyright) |
| `README.md` | `README.md` | Ported & Tested | Documented Rust port header with graphics & links |

## Examples Mapping (`examples/`)

| Upstream Go Example | Rust Executable Example | Status |
| :--- | :--- | :--- |
| `examples/simple/main.go` | `examples/simple.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/altscreen-toggle/main.go` | `examples/altscreen_toggle.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/mouse/main.go` | `examples/mouse.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/window-size/main.go` | `examples/window_size.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/fullscreen/main.go` | `examples/fullscreen.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/debounce/main.go` | `examples/debounce.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/exec/main.go` | `examples/exec.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/sequence/main.go` | `examples/sequence.rs` | Ported & Tested (v2.0.8 Declarative View API) |
| `examples/result/main.go` | `examples/result.rs` | Ported & Tested (v2.0.8 Declarative View API) |
