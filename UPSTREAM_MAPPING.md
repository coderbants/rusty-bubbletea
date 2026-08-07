# Upstream Go File Mapping: `charming-bubbletea`

Target Upstream Tag: `charmbracelet/bubbletea@v1.3.4`

| Upstream Go File | Dedicated Rust Port File | Status | Notes / Description |
| :--- | :--- | :--- | :--- |
| `tea.go` | `src/lib.rs`, `src/model.rs` | Ported & Tested | Core Elm architecture Model, Msg, Cmd |
| `tea_init.go` | `src/tea_init.rs` | Ported & Tested | Model init methods |
| `tea_test.go` | `tests/tea_test.rs` | Ported & Tested | Core program unit tests |
| `commands.go` | `src/commands.rs` | Ported & Tested | Built-in commands (`quit`, `batch`, `sequence`, `tick`, `every`, `set_window_title`, `window_size`, `enter_alt_screen`, `exit_alt_screen`) |
| `commands_test.go` | `tests/commands_test.rs` | Ported & Tested | Command suite integration tests |
| `key.go` | `src/key.rs` | Ported & Tested | Key types, KeyMsg, rune decoding |
| `key_sequences.go` | `src/key_sequences.rs` | Ported & Tested | Dedicated module for ANSI escape sequence detection & bracketed paste |
| `key_other.go` | `src/key_other.rs` | Ported & Tested | Dedicated module for POSIX non-windows key handling |
| `key_windows.go` | `src/key_windows.rs` | Ported & Tested | Dedicated module for Windows console input API mapping |
| `key_test.go` | `tests/key_test.rs` | Ported & Tested | Keyboard event suite tests |
| `mouse.go` | `src/mouse.rs` | Ported & Tested | MouseButton, MouseAction, MouseMsg |
| `mouse_test.go` | `tests/mouse_test.rs` | Ported & Tested | Mouse event suite tests |
| `program.go` | `src/program.rs` | Ported & Tested | Program event loop runner & options |
| `options.go` | `src/options.rs` | Ported & Tested | `ProgramOption` configuration constructors (`with_alt_screen`, `without_renderer`, `with_fps`, `with_filter`) |
| `options_test.go` | `tests/commands_test.rs` | Ported & Tested | Program option tests |
| `renderer.go` | `src/renderer.rs` | Ported & Tested | Dedicated `Renderer` trait definition |
| `standard_renderer.go` | `src/standard_renderer.rs` | Ported & Tested | Framerate-based terminal renderer & repainter |
| `nil_renderer.go` | `src/nil_renderer.rs` | Ported & Tested | Dedicated no-op testing renderer implementation |
| `nil_renderer_test.go` | `tests/tea_test.rs` | Ported & Tested | Nil renderer unit test |
| `screen.go` | `src/screen.rs` | Ported & Tested | Dedicated module for alternate screen buffer and mouse mode control commands |
| `screen_test.go` | `tests/commands_test.rs` | Ported & Tested | Screen buffer tests |
| `logging.go` | `src/logging.rs` | Ported & Tested | Dedicated file logger utility (`log_to_file`, `FileLogger`) |
| `logging_test.go` | `tests/tea_test.rs` | Ported & Tested | Logging unit test |
| `exec.go` | `src/exec.rs` | Ported & Tested | `exec_process` external command execution |
| `exec_test.go` | `tests/commands_test.rs` | Ported & Tested | Exec process unit tests |
| `focus.go` | `src/focus.rs` | Ported & Tested | Dedicated `FocusMsg` & `BlurMsg` terminal focus events |
| `tty.go` | `src/tty.rs` | Ported & Tested | `init_terminal` & `restore_terminal` state helpers |
| `tty_unix.go` | `src/tty_unix.rs` | Ported & Tested | Dedicated Unix TTY reader and raw mode initialization module |
| `tty_windows.go` | `src/tty_windows.rs` | Ported & Tested | Dedicated Windows VT console mode helpers module |
| `inputreader_other.go` | `src/inputreader_other.rs` | Ported & Tested | Dedicated Non-windows input reader module |
| `inputreader_windows.go` | `src/inputreader_windows.rs` | Ported & Tested | Dedicated Windows input reader module |
| `signals_unix.go` | `src/signals_unix.rs` | Ported & Tested | Dedicated `listen_for_resize` SIGWINCH resize listener module |
| `signals_windows.go` | `src/signals_windows.rs` | Ported & Tested | Dedicated Windows console signal listener module |
| `LICENSE` | `LICENSE` | Ported & Tested | MIT License (matching upstream copyright) |
| `README.md` | `README.md` | Ported & Tested | Documented Rust port header with graphics & links |

## Examples Mapping (`examples/`)

| Upstream Go Example | Rust Executable Example | Status |
| :--- | :--- | :--- |
| `examples/simple/main.go` | `examples/simple.rs` | Ported & Tested |
| `examples/altscreen-toggle/main.go` | `examples/altscreen_toggle.rs` | Ported & Tested |
| `examples/mouse/main.go` | `examples/mouse.rs` | Ported & Tested |
| `examples/window-size/main.go` | `examples/window_size.rs` | Ported & Tested |
| `examples/fullscreen/main.go` | `examples/fullscreen.rs` | Ported & Tested |
| `examples/debounce/main.go` | `examples/debounce.rs` | Ported & Tested |
| `examples/exec/main.go` | `examples/exec.rs` | Ported & Tested |
| `examples/sequence/main.go` | `examples/sequence.rs` | Ported & Tested |
| `examples/result/main.go` | `examples/result.rs` | Ported & Tested |
