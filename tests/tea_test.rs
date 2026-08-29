use rusty_bubbletea::clipboard::{self, ClipboardMsg, SetClipboardMsg};
use rusty_bubbletea::color::{self, ForegroundColorMsg};
use rusty_bubbletea::commands::{self, BatchMsg, QuitMsg, SequenceMsg, SuspendMsg};
use rusty_bubbletea::cursor::{Cursor, CursorPositionMsg, CursorShape};
use rusty_bubbletea::environ::EnvMsg;
use rusty_bubbletea::exec;
use rusty_bubbletea::key::KeyMod;
use rusty_bubbletea::keyboard::{
    KeyboardEnhancementsMsg, KITTY_REPORT_ALTERNATE_KEYS, KITTY_REPORT_EVENT_TYPES,
};
use rusty_bubbletea::logging;
use rusty_bubbletea::model::Model;
use rusty_bubbletea::mouse::{
    MouseButton, MouseClickMsg, MouseMotionMsg, MouseMsg, MouseReleaseMsg, MouseWheelMsg,
};
use rusty_bubbletea::nil_renderer::NilRenderer;
use rusty_bubbletea::options::{Context, ProgramOptions};
use rusty_bubbletea::profile::{ColorProfile, ColorProfileMsg};
use rusty_bubbletea::renderer::{PrintLineMsg, Renderer};
use rusty_bubbletea::screen::{clear_screen, ClearScreenMsg, ModeReportMsg, WindowSizeMsg};
use rusty_bubbletea::view::{MouseMode, ProgressBar, ProgressBarState, View};
use rusty_bubbletea::{quit, Cmd, Msg, Program};

struct TestModel {
    counter: usize,
}

impl Model for TestModel {
    fn init(&self) -> Cmd {
        // Immediately send a quit so the program exits without needing a real TTY.
        quit()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().is::<WindowSizeMsg>()
            || msg.as_any().is::<EnvMsg>()
            || msg.as_any().is::<ColorProfileMsg>()
        {
            return None;
        }
        self.counter += 1;
        quit()
    }

    fn view(&self) -> View {
        View::new(&format!("Counter: {}", self.counter))
    }
}

#[derive(Default)]
struct StartupModel {
    window: Option<(usize, usize)>,
    environment: Option<String>,
    profile: Option<ColorProfile>,
}

impl Model for StartupModel {
    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(window) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.window = Some((window.width, window.height));
        } else if let Some(environment) = msg.as_any().downcast_ref::<EnvMsg>() {
            self.environment = Some(environment.getenv("APP_MODE"));
        } else if let Some(profile) = msg.as_any().downcast_ref::<ColorProfileMsg>() {
            self.profile = Some(profile.profile);
            return quit();
        }
        None
    }

    fn view(&self) -> View {
        View::new("startup")
    }
}

struct PanicModel;

impl Model for PanicModel {
    fn init(&self) -> Cmd {
        panic!("intentional lifecycle panic")
    }

    fn update(&mut self, _msg: &dyn Msg) -> Cmd {
        None
    }

    fn view(&self) -> View {
        View::new("panic")
    }
}

/// Full interactive program run.
#[test]
fn test_v2_program_run() {
    let model = TestModel { counter: 0 };
    let prog = Program::new(model).with_options(
        ProgramOptions::default()
            .without_renderer()
            .with_input(None),
    );
    assert_eq!(prog.run().unwrap().counter, 0);
}

#[test]
fn test_program_handle_queues_prestart_quit_and_waits_for_cleanup() {
    let program = Program::new(TestModel { counter: 0 }).with_options(
        ProgramOptions::default()
            .without_renderer()
            .with_input(None),
    );
    let handle = program.handle();
    handle.quit();

    let runner = std::thread::spawn(move || {
        program
            .run()
            .map(|model| model.counter)
            .map_err(|error| error.to_string())
    });
    let model_counter = runner
        .join()
        .expect("program runner thread")
        .expect("quit succeeds");
    handle.wait();
    assert_eq!(model_counter, 0);
}

#[test]
fn test_program_handle_kill_returns_killed_after_cleanup() {
    let program = Program::new(TestModel { counter: 0 }).with_options(
        ProgramOptions::default()
            .without_renderer()
            .with_input(None),
    );
    let handle = program.handle();
    handle.kill();

    let runner = std::thread::spawn(move || {
        program
            .run()
            .map(|_| String::new())
            .map_err(|error| error.to_string())
    });
    let error = runner
        .join()
        .expect("program runner thread")
        .expect_err("kill should return an error");
    handle.wait();
    assert_eq!(error, rusty_bubbletea::program::ERR_PROGRAM_KILLED);
}

#[test]
fn test_program_uses_configured_startup_contract() {
    let options = ProgramOptions::default()
        .without_renderer()
        .with_input(None)
        .with_window_size(100, 40)
        .with_environment(vec![
            ("TERM".to_string(), "xterm-256color".to_string()),
            ("APP_MODE".to_string(), "test".to_string()),
        ])
        .with_color_profile(ColorProfile::ANSI256);
    let model = Program::new(StartupModel::default())
        .with_options(options)
        .run()
        .expect("configured startup should quit cleanly");

    assert_eq!(model.window, Some((100, 40)));
    assert_eq!(model.environment.as_deref(), Some("test"));
    assert_eq!(model.profile, Some(ColorProfile::ANSI256));
}

#[test]
fn test_program_context_cancellation_returns_killed() {
    let context = Context::new();
    context.cancel();
    let result = Program::new(TestModel { counter: 0 })
        .with_options(
            ProgramOptions::default()
                .without_renderer()
                .with_input(None)
                .with_context(context),
        )
        .run();

    let error = result.err().expect("cancelled program should fail");
    assert_eq!(
        error.downcast_ref::<rusty_bubbletea::program::ProgramError>(),
        Some(&rusty_bubbletea::program::ProgramError::Killed)
    );
}

#[test]
fn test_program_recovers_from_init_panic() {
    let result = Program::new(PanicModel)
        .with_options(
            ProgramOptions::default()
                .without_renderer()
                .with_input(None),
        )
        .run();

    let error = result
        .err()
        .expect("panic should be converted to ProgramError");
    assert_eq!(
        error.downcast_ref::<rusty_bubbletea::program::ProgramError>(),
        Some(&rusty_bubbletea::program::ProgramError::Panic)
    );
}

#[test]
fn test_protocol_query_precedes_buffered_renderer_startup_output() {
    let output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    struct RecordingWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for RecordingWriter {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            let mut output = self
                .0
                .lock()
                .map_err(|_| std::io::Error::other("recording writer lock poisoned"))?;
            output.extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let result = Program::new(TestModel { counter: 0 })
        .with_options(
            ProgramOptions::default()
                .with_input(None)
                .with_output(Box::new(RecordingWriter(output.clone())))
                .with_environment(vec![("TERM".to_string(), "xterm-256color".to_string())]),
        )
        .run();
    assert!(result.is_ok());

    let bytes = output.lock().expect("recorded output lock").clone();
    assert!(
        bytes.starts_with(b"\x1b[?2026$p\x1b[?2027$p"),
        "protocol query must precede buffered renderer startup output: {bytes:?}"
    );
}

#[cfg(unix)]
#[test]
fn test_raw_mode_start_failure_closes_initialized_renderer() {
    let output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    struct RecordingWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for RecordingWriter {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            let mut output = self
                .0
                .lock()
                .map_err(|_| std::io::Error::other("recording writer lock poisoned"))?;
            output.extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let result = Program::new(TestModel { counter: 0 })
        .with_options(
            ProgramOptions::default()
                .with_input(Some(Box::new(std::io::Cursor::new(Vec::<u8>::new()))))
                .with_output(Box::new(RecordingWriter(output.clone())))
                .with_environment(vec![("TERM".to_string(), "xterm-256color".to_string())]),
        )
        .run();

    assert!(result.is_err(), "non-terminal stdin must reject raw mode");
    assert!(
        !output.lock().expect("recorded output lock").is_empty(),
        "renderer close must flush its buffered startup state after raw-mode failure"
    );
}

#[derive(Default)]
struct MultiMsgModel {
    messages_received: usize,
}

impl Model for MultiMsgModel {
    fn init(&self) -> Cmd {
        // Send a batch of initial commands that then triggers a quit
        commands::sequence(vec![
            clear_screen(),
            color::request_background_color(),
            color::request_foreground_color(),
            color::request_cursor_color(),
            commands::request_window_size(),
            rusty_bubbletea::termcap::request_capability("Tc"),
            rusty_bubbletea::xterm::request_terminal_version(),
            Some(Box::new(|| {
                Some(Box::new(PrintLineMsg {
                    message_body: "hello print".into(),
                }))
            })),
            Some(Box::new(|| {
                Some(Box::new(WindowSizeMsg {
                    width: 90,
                    height: 30,
                }))
            })),
            Some(Box::new(|| {
                Some(Box::new(ColorProfileMsg {
                    profile: ColorProfile::TrueColor,
                }))
            })),
            quit(),
        ])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        self.messages_received += 1;
        if msg.as_any().is::<QuitMsg>() {
            return quit();
        }
        None
    }

    fn view(&self) -> View {
        View::new(&format!("Received: {}", self.messages_received))
    }
}

#[test]
fn test_program_with_message_flow() {
    let model = MultiMsgModel::default();
    let prog = Program::new(model).with_options(
        ProgramOptions::default()
            .without_signal_handler()
            .with_fps(60),
    );
    prog.println("Test println");
    prog.printf("Test printf");
    let finished_model = prog.run().unwrap();
    assert!(finished_model.messages_received > 0);
}

#[test]
fn test_view_properties_and_equality() {
    let mut v1 = View::new("Hello");
    assert_eq!(v1.content, "Hello");
    v1.set_content("World");
    assert_eq!(v1.content, "World");

    let v2 = View::new("World");
    assert!(v1.equals(&v2));

    v1.alt_screen = true;
    assert!(!v1.equals(&v2));

    v1.report_focus = true;
    v1.disable_bracketed_paste_mode = true;
    v1.window_title = "Title".to_string();
    v1.mouse_mode = MouseMode::MouseModeCellMotion;
    v1.progress_bar = Some(ProgressBar {
        state: ProgressBarState::ProgressBarDefault,
        value: 50,
    });
    assert_eq!(ProgressBarState::ProgressBarDefault.to_string(), "Default");
    assert_eq!(ProgressBarState::ProgressBarError.to_string(), "Error");
    assert_eq!(
        ProgressBarState::ProgressBarIndeterminate.to_string(),
        "Indeterminate"
    );
    assert_eq!(ProgressBarState::ProgressBarWarning.to_string(), "Warning");
    assert_eq!(ProgressBarState::ProgressBarNone.to_string(), "None");

    let c = Cursor::new(10, 5);
    assert_eq!(c.position.x, 10);
    assert_eq!(c.position.y, 5);
    assert_eq!(c.shape, CursorShape::CursorBlock);
    assert!(c.blink);
    assert_eq!(c.color, None);
}

#[test]
fn test_options_and_context() {
    let ctx = Context::new();
    assert!(!ctx.done());
    ctx.cancel();
    assert!(ctx.done());

    let opts: ProgramOptions<TestModel> = ProgramOptions::default()
        .with_fps(30)
        .without_renderer()
        .without_signals()
        .without_signal_handler()
        .without_catch_panics()
        .with_window_size(100, 40)
        .with_color_profile(ColorProfile::TrueColor)
        .with_context(ctx.clone())
        .with_filter(Box::new(|_m: &TestModel, msg: Box<dyn Msg>| Some(msg)))
        .with_environment(vec![("A".into(), "B".into())]);

    assert_eq!(opts.fps, 30);
    assert!(opts.disable_renderer);
    assert!(opts.disable_signals);
    assert!(opts.disable_signal_handler);
    assert!(opts.disable_catch_panics);
    assert_eq!(opts.width, 100);
    assert_eq!(opts.height, 40);
    assert_eq!(opts.color_profile, Some(ColorProfile::TrueColor));
    assert!(opts.context.is_some());
    assert!(opts.filter.is_some());
    assert!(opts.environ.is_some());
    assert_eq!(ProgramOptions::<TestModel>::default().with_fps(0).fps, 60);
    assert_eq!(
        ProgramOptions::<TestModel>::default().with_fps(121).fps,
        120
    );
}

#[test]
fn test_commands_and_messages() {
    let q = commands::quit();
    assert!(q.is_some());
    let msg = (q.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<QuitMsg>());

    let s = commands::suspend();
    let msg = (s.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<SuspendMsg>());

    let b = commands::batch(vec![commands::quit(), commands::suspend()]);
    let msg = (b.unwrap())().unwrap();
    let batch = msg.as_ref().as_any().downcast_ref::<BatchMsg>().unwrap();
    assert_eq!(batch.0.len(), 2);
    assert!(format!("{:?}", batch).contains("2 commands"));

    let seq = commands::sequence(vec![commands::quit(), commands::suspend()]);
    let msg = (seq.unwrap())().unwrap();
    let seq_msg = msg.as_ref().as_any().downcast_ref::<SequenceMsg>().unwrap();
    assert_eq!(seq_msg.0.len(), 2);
    assert!(format!("{:?}", seq_msg).contains("2 commands"));

    let singleton = commands::batch(vec![None, commands::quit()]);
    let singleton_msg = (singleton.expect("one command should be retained"))()
        .expect("retained command should produce a message");
    assert!(singleton_msg.as_ref().as_any().is::<QuitMsg>());
    assert!(commands::sequence(vec![None, None]).is_none());

    let rsz = commands::request_window_size();
    assert!(rsz.is_some());

    let ev = commands::every(std::time::Duration::from_millis(1), |_t| {
        Some(Box::new(QuitMsg))
    });
    assert!(ev.is_some());
    let _ = (ev.unwrap())();

    let tk = commands::tick(std::time::Duration::from_millis(1), |_t| {
        Some(Box::new(QuitMsg))
    });
    assert!(tk.is_some());
    let _ = (tk.unwrap())();

    let clr = clear_screen();
    let msg = (clr.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<ClearScreenMsg>());

    let mode = ModeReportMsg { mode: 1, value: 2 };
    assert_eq!(mode.mode, 1);
    assert_eq!(mode.value, 2);

    let ws = WindowSizeMsg {
        width: 80,
        height: 24,
    };
    assert_eq!(ws.width, 80);
    assert_eq!(ws.height, 24);
}

#[test]
fn test_clipboard_and_color_commands() {
    let set_cb = clipboard::set_clipboard("hello clip");
    assert!(set_cb.is_some());
    let msg = (set_cb.unwrap())().unwrap();
    let set_msg = msg
        .as_ref()
        .as_any()
        .downcast_ref::<SetClipboardMsg>()
        .unwrap();
    assert_eq!(set_msg.0, "hello clip");

    let read_cb = clipboard::read_clipboard();
    assert!(read_cb.is_some());

    let set_prim = clipboard::set_primary_clipboard("prim clip");
    assert!(set_prim.is_some());

    let read_prim = clipboard::read_primary_clipboard();
    assert!(read_prim.is_some());

    let cb_msg = ClipboardMsg {
        content: "clip content".to_string(),
        selection: b'c',
    };
    assert_eq!(cb_msg.clipboard(), b'c');
    assert_eq!(format!("{}", cb_msg), "clip content");

    let req_bg = color::request_background_color();
    assert!(req_bg.is_some());
    let req_fg = color::request_foreground_color();
    assert!(req_fg.is_some());
    let req_cur = color::request_cursor_color();
    assert!(req_cur.is_some());

    let fg_msg = ForegroundColorMsg(rusty_x_ansi::color::RGBColor { r: 255, g: 0, b: 0 });
    assert_eq!(fg_msg.to_hex(), "#ff0000");
    assert!(!fg_msg.is_dark());

    let bg_msg = color::BackgroundColorMsg(rusty_x_ansi::color::RGBColor { r: 0, g: 0, b: 0 });
    assert_eq!(bg_msg.to_hex(), "#000000");
    assert!(bg_msg.is_dark());

    let cur_msg = color::CursorColorMsg(rusty_x_ansi::color::RGBColor {
        r: 255,
        g: 255,
        b: 255,
    });
    assert_eq!(cur_msg.to_hex(), "#ffffff");
    assert!(!cur_msg.is_dark());

    // Termcap & Xterm requests
    let cap_cmd = rusty_bubbletea::termcap::request_capability("Tc");
    assert!(cap_cmd.is_some());
    let cap_msg = (cap_cmd.unwrap())().unwrap();
    assert_eq!(
        cap_msg
            .as_ref()
            .as_any()
            .downcast_ref::<rusty_bubbletea::termcap::RequestCapabilityMsg>()
            .unwrap()
            .0,
        "Tc"
    );
    let cap_resp = rusty_bubbletea::termcap::CapabilityMsg {
        content: "Tc=1".to_string(),
    };
    assert_eq!(format!("{}", cap_resp), "Tc=1");

    let xterm_cmd = rusty_bubbletea::xterm::request_terminal_version();
    assert!(xterm_cmd.is_some());
    let xt_resp = rusty_bubbletea::xterm::TerminalVersionMsg {
        name: "xterm 388".to_string(),
    };
    assert_eq!(format!("{}", xt_resp), "xterm 388");

    // TTY helpers
    let _ = rusty_bubbletea::tty::get_window_size();
    let (_tx_sig, _rx_sig) = std::sync::mpsc::channel::<Box<dyn Msg>>();
    #[cfg(unix)]
    rusty_bubbletea::signals_unix::listen_for_resize(&_tx_sig);
}

#[test]
fn test_keyboard_and_mouse_and_env_and_logging() {
    let km = KeyboardEnhancementsMsg {
        flags: KITTY_REPORT_EVENT_TYPES | KITTY_REPORT_ALTERNATE_KEYS,
    };
    assert!(km.supports_key_disambiguation());
    assert!(km.supports_event_types());
    assert!(km.supports_alternate_keys());
    assert!(!km.supports_all_keys_as_escape_codes());
    assert!(!km.supports_associated_text());

    let m = rusty_bubbletea::mouse::Mouse {
        x: 5,
        y: 10,
        button: MouseButton::MouseLeft,
        mod_keys: KeyMod::default(),
    };
    let click = MouseClickMsg(m.clone());
    let motion = MouseMotionMsg(m.clone());
    let release = MouseReleaseMsg(m.clone());
    let wheel = MouseWheelMsg(m.clone());

    assert_eq!(format!("{}", click), "(5;10) MouseLeft");
    assert_eq!(format!("{}", release), "(5;10) MouseLeft");
    assert_eq!(format!("{}", wheel), "(5;10) MouseLeft");
    assert_eq!(format!("{}", motion), "(5;10) MouseLeft+motion");

    let none_mouse = rusty_bubbletea::mouse::Mouse {
        x: 0,
        y: 0,
        button: MouseButton::MouseNone,
        mod_keys: KeyMod::default(),
    };
    assert_eq!(
        format!("{}", MouseMotionMsg(none_mouse)),
        "(0;0) MouseNone motion"
    );

    let msg_click = MouseMsg::Click(click);
    assert_eq!(msg_click.mouse().x, 5);
    assert_eq!(format!("{}", msg_click), "(5;10) MouseLeft");

    let _ = MouseMsg::Motion(motion);
    let _ = MouseMsg::Release(release);
    let _ = MouseMsg::Wheel(wheel);

    let env = EnvMsg::new(vec![("FOO".to_string(), "BAR".to_string())]);
    assert_eq!(env.getenv("FOO"), "BAR");
    assert_eq!(env.getenv("NONEXIST"), "");
    let (val, ok) = env.lookup_env("FOO");
    assert!(ok);
    assert_eq!(val, "BAR");

    let cur_pos = CursorPositionMsg { x: 12, y: 34 };
    assert_eq!(cur_pos.x, 12);
    assert_eq!(cur_pos.y, 34);

    let exec = exec::exec_process("echo", &["hi"]);
    assert!(exec.is_some());

    let mut nil = NilRenderer;
    nil.start();
    nil.render(View::new("nil"));
    assert!(nil.flush(false).is_ok());
    nil.resize(80, 24);
    nil.clear_screen();
    assert!(nil.write_string("abc").is_ok());
    assert!(nil.insert_above("line".into()).is_ok());
    nil.reset();
    assert!(nil.close().is_ok());

    let log_path = std::env::temp_dir()
        .join("rusty-bubbletea-test.log")
        .to_string_lossy()
        .into_owned();
    let mut logger = logging::log_to_file(&log_path, "test").unwrap();
    logger.log("test message");
}

#[test]
fn test_cursed_renderer_full_lifecycle_and_features() {
    use rusty_bubbletea::cursed_renderer::new_cursed_renderer;
    use rusty_x_ansi::method::WidthMethod;

    let buf = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    struct MockWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for MockWriter {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut rend = new_cursed_renderer(
        Box::new(MockWriter(buf.clone())),
        &["TERM=xterm-256color".to_string()],
        80,
        24,
    );

    rend.set_optimizations(true, true, true);
    rend.set_color_profile(rusty_colorprofile::Profile::TrueColor);
    rend.set_syncd_updates(true);
    rend.set_width_method(WidthMethod::GraphemeWidth);
    rend.set_width_method(WidthMethod::WcWidth);
    rend.set_scroll_optim(true);

    rend.start();
    rend.resize(100, 30);
    rend.clear_screen();
    assert!(rend.write_string("Hello Cursed").is_ok());
    assert!(rend.insert_above("Inserted line".to_string()).is_ok());

    let mut v = View::new("Line 1\nLine 2\nLine 3");
    v.alt_screen = true;
    v.report_focus = true;
    v.window_title = "Test Window".to_string();
    v.mouse_mode = MouseMode::MouseModeAllMotion;
    v.cursor = Some(Cursor::new(5, 2));
    v.foreground_color = Some(rusty_x_ansi::color::RGBColor {
        r: 255,
        g: 100,
        b: 50,
    });
    v.background_color = Some(rusty_x_ansi::color::RGBColor {
        r: 10,
        g: 20,
        b: 30,
    });
    v.progress_bar = Some(ProgressBar {
        state: ProgressBarState::ProgressBarDefault,
        value: 50,
    });

    rend.render(v.clone());
    assert!(rend.flush(false).is_ok());

    // Switch view modes: cell motion and exit alt screen
    v.alt_screen = false;
    v.mouse_mode = MouseMode::MouseModeCellMotion;
    v.progress_bar = Some(ProgressBar {
        state: ProgressBarState::ProgressBarError,
        value: 100,
    });
    rend.render(v.clone());
    assert!(rend.flush(false).is_ok());

    // Switch mouse mode to none
    v.mouse_mode = MouseMode::MouseModeNone;
    v.progress_bar = Some(ProgressBar {
        state: ProgressBarState::ProgressBarIndeterminate,
        value: 0,
    });
    rend.render(v);
    assert!(rend.flush(true).is_ok());

    let m_ev = rusty_bubbletea::mouse::Mouse {
        x: 2,
        y: 3,
        button: MouseButton::MouseLeft,
        mod_keys: KeyMod::default(),
    };
    let _ = rend.on_mouse(MouseMsg::Click(MouseClickMsg(m_ev)));

    rend.reset();
    assert!(rend.close().is_ok());
}
