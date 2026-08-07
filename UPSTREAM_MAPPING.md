# Upstream Go File Mapping: `charming-bubbletea`

Target Upstream Tag: `charmbracelet/bubbletea@v1.3.4`

| Upstream Go File | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `tea.go` | `src/lib.rs`, `src/model.rs` | Core Elm architecture Model, Msg, Cmd, init |
| `tea_init.go` | `src/model.rs` | Model initialization logic |
| `tea_test.go` | `tests/tea_test.rs` | Core program tests |
| `commands.go` | `src/commands.rs` | Built-in commands (`quit`, `batch`, `sequence`, `enter_alt_screen`, `exit_alt_screen`, `WindowSizeMsg`) |
| `commands_test.go` | `tests/commands_test.rs` | Command suite integration tests |
| `key.go` | `src/key.rs` | Key types, KeyMsg, rune decoding |
| `key_sequences.go` | `src/key.rs` | ANSI escape sequence key mapping |
| `key_other.go` | `src/key.rs` | Non-windows key handling |
| `key_windows.go` | `src/key.rs` | Windows console input API mapping |
| `key_test.go` | `tests/key_test.rs` | Keyboard event suite tests |
| `mouse.go` | `src/mouse.rs` | MouseButton, MouseAction, MouseMsg |
| `mouse_test.go` | `tests/mouse_test.rs` | Mouse event suite tests |
| `exec.go` | `examples/exec.rs`, `src/program.rs` | External command execution |
| `exec_test.go` | `tests/commands_test.rs` | Exec process tests |
| `options.go` | `src/program.rs` | Program startup options |
| `options_test.go` | `tests/commands_test.rs` | Program option tests |
| `renderer.go` | `src/program.rs` | Renderer interface definition |
| `standard_renderer.go` | `src/program.rs` | Standard ANSI terminal renderer |
| `nil_renderer.go` | `src/program.rs` | No-op testing renderer |
| `nil_renderer_test.go` | `tests/tea_test.rs` | Nil renderer test |
| `screen.go` | `src/program.rs` | Alternate screen buffer controls |
| `screen_test.go` | `tests/commands_test.rs` | Screen buffer tests |
| `logging.go` | `src/program.rs` | File logger utility |
| `logging_test.go` | `tests/tea_test.rs` | Logging unit test |
| `focus.go` | `src/program.rs` | Focus / blur terminal reporting |
| `tty.go` | `src/program.rs` | TTY handle initialization |
| `tty_unix.go` | `src/program.rs` | Unix TTY reader |
| `tty_windows.go` | `src/program.rs` | Windows VT console mode |
| `inputreader_other.go` | `src/program.rs` | Non-windows input reader |
| `inputreader_windows.go` | `src/program.rs` | Windows input reader |
| `signals_unix.go` | `src/program.rs` | Unix POSIX signal listener (SIGWINCH, SIGINT) |
| `signals_windows.go` | `src/program.rs` | Windows console signal listener |
| `LICENSE` | `LICENSE` | MIT License (matching upstream copyright) |
| `README.md` | `README.md` | Documented Rust port header with graphics & links |

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
