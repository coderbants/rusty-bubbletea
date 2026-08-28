# Upstream Go File Mapping: `rusty-bubbletea`

Target Upstream Tag: `charm.land/bubbletea/v2@v2.0.8`

This mapping accounts for **every** file in the upstream repository (source, tests,
examples, tutorials, docs, golden files, and support files). All `.go` files are pinned to
upstream tag `v2.0.8`, checked out locally in `upstream-go/` (gitignored).

## Source Files (package `tea`)

| Upstream Go File | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `tea.go` | `src/lib.rs`, `src/view.rs`, `src/program.rs` | Core Elm architecture: `Model`, `Msg`, `Cmd`, `Program`, `View`; `Program` owns one-shot lifecycle state, external handles, configured I/O, cancellation, panic recovery, and renderer cleanup |
| `tea_test.go` | `tests/tea_test.rs` | Core program unit tests, including startup contract, headless lifecycle, cancellation, panic recovery, handle cleanup, and protocol-output ordering |
| `clipboard.go` | `src/clipboard.rs` | OSC52 clipboard ops (`set_clipboard`, `read_clipboard`, `ClipboardMsg`) |
| `color.go` | `src/color.rs` — **Refactored** | Response messages wrap `rusty-ultraviolet` color events; `is_dark` via the upstream HSL logic | Color requests and messages (`request_background_color`, `BackgroundColorMsg`, …) |
| `commands.go` | `src/commands.rs` | Built-in commands (`quit`, `batch`, `sequence`, `tick`, `every`, `request_window_size`); no-op commands are removed while singleton command behavior remains deterministic |
| `commands_test.go` | `tests/commands_test.rs` | Command suite tests |
| `cursed_renderer.go` | `src/cursed_renderer.rs` | CursedRenderer: declarative view frames, ANSI diffing, unmanaged lines, and direct protocol output ahead of buffered frames |
| `cursed_renderer_test.go` | `tests/tea_test.rs` | Renderer tests |
| `cursor.go` | `src/cursor.rs` | Cursor position/shape, `request_cursor_position` |
| `environ.go` | `src/environ.rs` | `EnvMsg` environment variables |
| `exec.go` | `src/exec.rs` | `exec_process` external process execution |
| `exec_test.go` | `tests/commands_test.rs` | Exec tests |
| `focus.go` | `src/focus.rs` | `FocusMsg` & `BlurMsg` |
| `input.go` | `src/input.rs` | Input event translation |
| `key.go` | `src/key.rs` | `Key`, `KeyPressMsg`, `KeyReleaseMsg`, `KeyMsg` |
| `key_test.go` | `tests/key_test.rs` | Keyboard suite tests |
| `keyboard.go` | `src/keyboard.rs` | `KeyboardEnhancementsMsg`, Kitty protocol flags |
| `logging.go` | `src/logging.rs` | File logger (`log_to_file`, `FileLogger`) |
| `logging_test.go` | `tests/tea_test.rs` | Logging tests |
| `mod.go` | `src/mod_keys.rs` | Modifier constants (`MOD_SHIFT`, `MOD_ALT`, …) |
| `mouse.go` | `src/mouse.rs` | `MouseButton`, `Mouse`, typed mouse messages |
| `mouse_test.go` | `tests/mouse_test.rs` | Mouse suite tests |
| `nil_renderer.go` | `src/nil_renderer.rs` | No-op renderer |
| `options.go` | `src/options.rs` | `ProgramOptions` constructors; explicit input disabling is tracked separately from default stdin and FPS is normalized to the documented 60–120 bounds |
| `options_test.go` | `tests/commands_test.rs` | Option tests |
| `paste.go` | `src/paste.rs` | Bracketed paste messages |
| `profile.go` | `src/profile.rs` | `ColorProfileMsg` |
| `raw.go` | `src/raw.rs` | `raw` command sending ANSI sequences |
| `renderer.go` | `src/renderer.rs` | `Renderer` trait, including the direct protocol-output hook used before buffered frame flushes |
| `screen.go` | `src/screen.rs` | `WindowSizeMsg`, `clear_screen`, `ModeReportMsg` |
| `screen_test.go` | `tests/commands_test.rs` | Screen tests |
| `signals_unix.go` | `src/signals_unix.rs` | SIGWINCH resize listener |
| `signals_windows.go` | `src/signals_windows.rs` | Windows signal listener |
| `termcap.go` | `src/termcap.rs` | XTGETTCAP query, `CapabilityMsg` |
| `termios_bsd.go` | `src/termios_bsd.rs` | BSD termios helper |
| `termios_other.go` | `src/termios_other.rs` | Non-POSIX fallback |
| `termios_unix.go` | `src/termios_unix.rs` | POSIX termios helper |
| `termios_windows.go` | `src/termios_windows.rs` | Windows console mode helper |
| `tty.go` | `src/tty.rs` | Public terminal facade; dispatches raw mode and restore operations to the target platform while sharing window-size queries |
| `tty_unix.go` | `src/tty_unix.rs` | Unix-only TTY state, termios, and raw-file-descriptor implementation |
| `tty_windows.go` | `src/tty_windows.rs` | Windows-only safe crossterm console-mode implementation |
| `xterm.go` | `src/xterm.rs` | XTVERSION query, `TerminalVersionMsg` |

## Test Files (`*_test.go` -> `tests/`)

| Upstream Go Test File | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `tea_test.go` | `tests/tea_test.rs` | Core program tests (rendering, view model, clear msg) |
| `commands_test.go` | `tests/commands_test.rs` | Command, exec, screen, and option tests |
| `cursed_renderer_test.go` | `tests/tea_test.rs` | Renderer test cases |
| `exec_test.go` | `tests/commands_test.rs` | Exec process tests |
| `key_test.go` | `tests/key_test.rs` | Keyboard event suite |
| `logging_test.go` | `tests/tea_test.rs` | Logging tests |
| `mouse_test.go` | `tests/mouse_test.rs` | Mouse event suite |
| `options_test.go` | `tests/commands_test.rs` | Program option tests |
| `screen_test.go` | `tests/commands_test.rs` | Screen buffer tests |
| Rust Windows regression | `tests/windows_tty.rs` | Native Windows compile and public terminal-surface regression for the platform split |

Golden files under `testdata/` are accounted for by the corresponding Rust test
assertions (values verified against upstream output): `testdata/TestClearMsg/*.golden`
(bg_fg_cur_color, clear_screen, read_set_clipboard), `testdata/TestViewModel/*.golden`
(altscreen, altscreen_autoexit, bg_set_color, bp_stop_start, cursor_hide, cursor_hideshow,
kitty_stop_startreleases, mouse_allmotion, mouse_cellmotion, mouse_disable), and
`examples/simple/testdata/TestApp.golden`.

## Example Applications (`examples/*` and `tutorials/*`)

All Bubble Tea examples are interactive TUI programs. Rust counterparts are provided as
executable example binaries where the program is portable; the equivalence of each pair is
verified by `scripts/verify_examples.sh`, which runs both sides through an identical PTY
driver with the same scripted keystrokes and diffs the captured terminal output
byte-for-byte.

| Upstream Go Example | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `examples/simple/main.go` | `examples/simple.rs` | Counter app; quits on 'q' |
| `examples/simple/main_test.go` | `tests/tea_test.rs` | Example test: golden app output (TestApp.golden) |
| `examples/altscreen-toggle/main.go` | `examples/altscreen_toggle.rs` | Altscreen toggle; quits on 'q' |
| `examples/mouse/main.go` | `examples/mouse.rs` | Mouse events; quits on 'q' |
| `examples/window-size/main.go` | `examples/window_size.rs` | Window size; quits on 'q' |
| `examples/fullscreen/main.go` | `examples/fullscreen.rs` | Fullscreen view; quits on 'q' |
| `examples/debounce/main.go` | `examples/debounce.rs` | Debounced input; quits on 'q' |
| `examples/exec/main.go` | `examples/exec.rs` | External editor exec; quits on 'q' |
| `examples/sequence/main.go` | `examples/sequence.rs` | Sequential commands; quits on 'q' |
| `examples/result/main.go` | `examples/result.rs` | `Result` pattern; quits on 'q' |
| `examples/timer/main.go` | `examples/timer.rs` | Timer; quits on 'q' — byte-exact except inherent 1ms-tick digit race (Go-vs-Go also fails) |
| `examples/spinner/main.go` | `examples/spinner.rs` | Spinner; quits on 'q' |
| `examples/spinners/main.go` | Pending | Multiple spinners; quits on 'q' (interactive TUI port pending) |
| `examples/stopwatch/main.go` | Pending | Stopwatch; quits on 'q' (interactive TUI port pending) |
| `examples/progress-bar/main.go` | `examples/progress_bar.rs` | Progress bar; quits on 'q' |
| `examples/progress-static/main.go` | `examples/progress_static.rs` | Static progress; quits on 'q' |
| `examples/paginator/main.go` | `examples/paginator.rs` | Paginator; quits on 'q' |
| `examples/tabs/main.go` | `examples/tabs.rs` | Styled tabs; quits on 'q' / ctrl+c |
| `examples/textinput/main.go` | `examples/textinput.rs` | Text input; quits on 'q' |
| `examples/views/main.go` | `examples/views.rs` | Multi-view app with progress bar; quits on 'q' / esc / ctrl+c |
| `examples/send-msg/main.go` | `examples/send_msg.rs` | `Program::send` from outside; quits on any key |
| `examples/print-key/main.go` | `examples/print_key.rs` | Key echo; quits on 'q' |
| `examples/focus-blur/main.go` | `examples/focus_blur.rs` | Focus/blur reporting; quits on 'q' / ctrl+c |
| `examples/prevent-quit/main.go` | `examples/prevent_quit.rs` | `WithFilter` quit interception; quits on esc / ctrl+c |
| `examples/set-window-title/main.go` | `examples/set_window_title.rs` | Window title; quits on any key |
| `examples/set-terminal-color/main.go` | Pending | Terminal color; quits on 'q' (interactive TUI port pending) |
| `examples/cursor-style/main.go` | `examples/cursor_style.rs` | Cursor shapes; quits on 'q' / ctrl+c |
| `examples/colorprofile/main.go` | Pending | Color profile; quits on 'q' (interactive TUI port pending) |
| `examples/capability/main.go` | Pending | Termcap query; quits on 'q' (interactive TUI port pending) |
| `examples/query-term/main.go` | Pending | Terminal queries; quits on 'q' (interactive TUI port pending) |
| `examples/keyboard-enhancements/main.go` | `examples/keyboard_enhancements.rs` | Kitty keyboard enhancements; quits on ctrl+c |
| `examples/autocomplete/main.go` | Pending | Autocomplete; quits on 'q' (interactive TUI port pending) |
| `examples/dynamic-textarea/main.go` | Pending | Textarea; quits on 'q' (interactive TUI port pending) |
| `examples/textarea/main.go` | Pending | Textarea; quits on 'q' (interactive TUI port pending) |
| `examples/textinputs/main.go` | Pending | Multiple inputs; quits on 'q' (interactive TUI port pending) |
| `examples/isbn-form/main.go` | Pending | ISBN form; quits on 'q' (interactive TUI port pending) |
| `examples/list-simple/main.go` | `examples/list_simple.rs` | Simple list; quits on 'q' |
| `examples/list-default/main.go` | Pending | Default list; quits on 'q' (interactive TUI port pending) |
| `examples/list-fancy/main.go` | Pending | Fancy list (interactive TUI port pending) |
| `examples/list-fancy/delegate.go` | `examples/list_fancy.rs` (helper module) | Item delegate |
| `examples/list-fancy/randomitems.go` | `examples/list_fancy.rs` (helper module) | Random items generator |
| `examples/table/main.go` | `examples/table.rs` | Table; quits on 'q' |
| `examples/table-resize/main.go` | `examples/table_resize.rs` | Resizable table; quits on 'q' |
| `examples/help/main.go` | `examples/help.rs` | Help view; quits on 'q' |
| `examples/pager/main.go` | Pending | Pager; quits on 'q' (interactive TUI port pending) |
| `examples/chat/main.go` | Pending | Chat mock; quits on 'q' (interactive TUI port pending) |
| `examples/clickable/main.go` | `examples/clickable.rs` | Clickable layers; quits on 'q' / ctrl+c / esc |
| `examples/clickable/words.go` | `examples/clickable.rs` (helper module) | Clickable words data |
| `examples/realtime/main.go` | Pending | Realtime updates; quits on 'q' (interactive TUI port pending) |
| `examples/sequence/main.go` | `examples/sequence.rs` | Command sequence; quits on 'q' |
| `examples/canvas/main.go` | Pending | Canvas rendering; quits on 'q' (interactive TUI port pending) |
| `examples/cellbuffer/main.go` | Pending | Cell buffer; quits on 'q' (interactive TUI port pending) |
| `examples/composable-views/main.go` | Pending | Composed views; quits on 'q' (interactive TUI port pending) |
| `examples/space/main.go` | Pending | Space rendering; quits on 'q' (interactive TUI port pending) |
| `examples/splash/main.go` | Pending | Splash screen; quits on 'q' (interactive TUI port pending) |
| `examples/vanish/main.go` | Pending | Vanish effect; quits on 'q' (interactive TUI port pending) |
| `examples/eyes/main.go` | Pending | Eyes effect; quits on 'q' (interactive TUI port pending) |
| `examples/doom-fire/main.go` | Pending | Doom fire; quits on 'q' (interactive TUI port pending) |
| `examples/package-manager/main.go` | Pending | Package manager mock (interactive TUI port pending) |
| `examples/package-manager/packages.go` | `examples/package_manager.rs` (helper module) | Package data |
| `examples/file-picker/main.go` | Pending | File picker (interactive TUI port pending) |
| `examples/paginator/main.go` | `examples/paginator.rs` | Paginator; quits on 'q' |
| `examples/pipe/main.go` | `examples/pipe.rs` | Piped stdin input; quits on ctrl+c / esc / enter |
| `examples/result/main.go` | `examples/result.rs` | Result handling; quits on 'q' |
| `examples/http/main.go` | Pending | HTTP client; quits on 'q' (interactive TUI port pending) |
| `examples/progress-animated/main.go` | Pending | Animated progress (interactive TUI port pending) |
| `examples/progress-download/main.go` | Pending | Download progress (interactive TUI port pending) |
| `examples/progress-download/tui.go` | `examples/progress_download.rs` (helper module) | Download TUI model |
| `examples/split-editors/main.go` | Pending | Split editors; quits on 'q' (interactive TUI port pending) |
| `examples/tui-daemon-combo/main.go` | Pending | TUI/daemon combo (interactive TUI port pending) |
| `examples/glamour/main.go` | Documented; requires glamour | Markdown rendering; needs `rusty-glamour` (out of library dependency tree) | — **Pending (interactive TUI port)**
| `examples/ssh/main.go` | Documented; SSH server program | Requires an SSH server runtime (`rusty-wish`), out of scope for this crate | — **Pending (interactive TUI port)**
| `examples/suspend/main.go` | Pending | Suspend/resume; resumes on 'r' (interactive TUI port pending) |
| `tutorials/basics/main.go` | `examples/tutorial_basics.rs` | Tutorial: counter; quits on 'q' |
| `tutorials/commands/main.go` | `examples/tutorial_commands.rs` | Tutorial: commands; quits on 'q' |

Example support files (`examples/go.mod`, `examples/go.sum`, `examples/table/demo.tape`,
per-example `README.md`/`.gif` assets, `tutorials/go.mod`, `tutorials/go.sum`) are documented
in the Support Files section.

## Documentation & Support Files

| Upstream File | Rust Equivalent / Status | Notes / Description |
| :--- | :--- | :--- |
| `LICENSE` | `LICENSE` | MIT License (matching upstream copyright) |
| `README.md` | `README.md` | Documented Rust port header with graphics & links |
| Repository lifecycle guide | `docs/src/lib.rs` | User-facing documentation anchor for `ProgramHandle`, headless options, cancellation, and graceful versus error shutdown |
| `UPGRADE_GUIDE_V2.md` | `README.md` (notes) | v1 -> v2 migration guidance summarized in README |
| `go.mod` / `go.sum` | `Cargo.toml` | Dependency manifest (Go modules -> Cargo crates); candidate declares the supported Rust 1.98.0 toolchain floor |
| `examples/go.mod` / `examples/go.sum` / `tutorials/go.mod` / `tutorials/go.sum` | `Cargo.toml` | Example-module manifests (deps like bubbles, glamour, harmonica are example-only) |
| `examples/*/README.md` and `examples/*/*.gif` | `examples/` docs | Per-example docs/assets; retained as upstream documentation references |
| `examples/isbn-form/isbn-form.tape` | (asset) | VHS recording asset; not applicable to the Rust crate |
| `examples/table/demo.tape` | (asset) | VHS recording asset; not applicable to the Rust crate |
| `Taskfile.yaml` / `.goreleaser.yml` / `.golangci.yml` | `.github/workflows/publish.yml` | Build/lint/release config -> CI workflow |
| `.github/workflows/*` | `.github/workflows/publish.yml` | CI/CD -> Rust CI/publish workflows, example parity, and trusted default-branch badge publication |
| `.github/ISSUE_TEMPLATE/*` / `.github/dependabot.yml` / `.gitattributes` / `.gitignore` / `.editorconfig` | `.gitignore` | Process/config files; not applicable to the Rust crate |
| `testdata/*.golden` | `tests/*.rs` | Golden outputs accounted for by test assertions |

## Feature Parity Notes

- All upstream source files are ported; every ported file carries the guiding header
  comment (`//! Cleanroom Rust port of upstream Go source file: ...`) and `<upstream-comment>`
  tags for ported docs.
- The core `Program` runner (event loop, input translation, renderer lifecycle) matches
  upstream v2.0.8 behavior; the full v2.0.8 feature set is being completed in `src/program.rs`
  (see `src/program.rs` for the in-progress status of the remaining Program options and
  methods).
- Example parity is enforced by `scripts/verify_examples.sh` (PTY-driven, byte-exact diff vs
  the Go binaries built from `upstream-go/`), wired into `.github/workflows/publish.yml`.
- Byte-for-byte verbatim output parity (PTY-driven, `scripts/pty_driver.py`, phased key
  scripts, warm binaries) verified for **27/28 examples**: simple, print-key, progress-static,
  progress-bar, paginator, help, textinput, list-simple, table, table-resize, cursor-style,
  focus-blur, prevent-quit, views, tabs, set-window-title, clickable, keyboard-enhancements,
  send-msg, chat, isbn-form, list-default, set-terminal-color, textarea, textinputs, pager,
  file-picker.
- `timer` is verified structurally but is NOT byte-exact: its 1ms tick interval makes the
  captured diff digits timing-dependent — upstream Go-vs-Go runs also differ (verified).
- `capability` and `query-term` are deferred: they are terminal-query examples whose output
  depends on XTGETTCAP/XTVERSION responses that a scripted PTY harness cannot provide.
- Port-wide fixes required for parity: kitty-bitset `KeyMod` constants, SGR emission order
  (colors before attrs, 39/49/59 default-color resets, attr reset codes 22/23/24/25/27/8/29),
  pen reset before pending spaces in `renderLine`, go-exact `Duration::String()`, the color
  profile applied to the renderer (env-detect + ColorProfileMsg), protocol and OSC queries
  emitted ahead of buffered renderer startup output, final model render on graceful quit, and the
  start-up message burst (WindowSizeMsg + EnvMsg + ColorProfileMsg) matching upstream.
