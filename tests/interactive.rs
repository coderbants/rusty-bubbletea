//! Interactive integration tests for the Bubble Tea examples, driven through
//! a real pseudo-terminal (Playwright-style): keys, typing, mouse, resizing,
//! and assertions on the reconstructed on-screen state.
//!
//! The examples are the same binaries verified byte-for-byte by
//! `scripts/verify_examples.sh`; these tests additionally exercise the
//! interactive behavior (typing, navigating, mouse clicks) that a byte-level
//! key-sweep cannot.

use charming_testkit::PtySession;

fn ex(name: &str) -> String {
    format!("target/debug/examples/{name}")
}

#[test]
fn textinput_type_and_submit() {
    let pty = PtySession::spawn(&ex("textinput"), &[]).expect("spawn");
    pty.wait_for_text("favorite", 5000).expect("initial");
    // The text input is pre-focused: type a name.
    pty.type_text("Pikachu").expect("type");
    pty.wait_for_text("Pikachu", 5000).expect("typed");
    // The typed value is echoed in the input line.
    pty.press("enter").expect("enter");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn textinput_backspace_and_clear() {
    let pty = PtySession::spawn(&ex("textinput"), &[]).expect("spawn");
    pty.wait_for_text("favorite", 5000).expect("initial");
    pty.type_text("abc").expect("type");
    pty.wait_for_text("abc", 5000).expect("typed");
    pty.press("backspace").expect("bs");
    pty.wait_for_text("> ab", 5000).expect("backspaced");
    pty.wait_until(5000, |s| !s.contains("> abc"))
        .expect("no abc");
    // ctrl+w clears the word.
    pty.press("ctrl+w").expect("cw");
    pty.wait_until(5000, |s| !s.contains("> ab"))
        .expect("cleared");
    pty.press("ctrl+c").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn list_simple_navigation() {
    let pty = PtySession::spawn(&ex("list_simple"), &[]).expect("spawn");
    pty.wait_for_text("Ramen", 5000).expect("list shown");
    // The first item is selected: press j (or down) to move down.
    pty.press("j").expect("j");
    pty.wait_for_text("Tomato Soup", 5000)
        .expect("second item shown");
    // The selection indicator '>' must be on the Tomato Soup row.
    pty.wait_until(5000, |s| {
        s.find("Tomato Soup")
            .map(|(_, y)| s.line(y).contains('>'))
            .unwrap_or(false)
    })
    .expect("selection on second item");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn paginator_keys_and_arrows() {
    let pty = PtySession::spawn(&ex("paginator"), &[]).expect("spawn");
    pty.wait_for_text("Item 1", 5000).expect("page 1");
    // 'l' advances one page.
    pty.press("l").expect("l");
    pty.wait_for_text("Item 11", 5000).expect("page 2");
    // Arrow left goes back.
    pty.press("left").expect("left");
    pty.wait_for_text("Item 1", 5000).expect("page 1");
    // Rapid mixed arrows.
    for _ in 0..7 {
        pty.press("right").expect("right");
    }
    pty.press("left").expect("left");
    pty.wait_for_text("Item 61", 5000).expect("page 7");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn tabs_switch_with_arrow_keys() {
    let pty = PtySession::spawn(&ex("tabs"), &[]).expect("spawn");
    pty.wait_for_text("Lip Gloss", 5000).expect("tab 1");
    pty.press("right").expect("right");
    pty.wait_for_text("Blush", 5000).expect("tab 2");
    pty.press("right").expect("right");
    pty.wait_for_text("Eye Shadow", 5000).expect("tab 3");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn mouse_click_updates_last_event() {
    let pty = PtySession::spawn(&ex("mouse"), &[]).expect("spawn");
    pty.wait_for_text("Do mouse stuff", 5000).expect("shown");
    std::thread::sleep(std::time::Duration::from_millis(300));
    // Click at cell (20, 5): SGR protocol uses 1-based coordinates.
    pty.send(&charming_testkit::keys::mouse_click(20, 5))
        .expect("click");
    pty.wait_until(5000, |s| s.contains("Y: 4) left"))
        .expect("event recorded");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn help_toggles_with_question_mark() {
    let pty = PtySession::spawn(&ex("help"), &[]).expect("spawn");
    pty.wait_for_text("Waiting for input", 5000)
        .expect("initial");
    // Press a key so the status line changes, then '?' toggles the help.
    pty.press("k").expect("k");
    pty.wait_for_text("You chose: ↑", 5000).expect("chose up");
    pty.press("?").expect("?");
    pty.wait_for_text("move up", 5000).expect("full help shown");
    pty.press("?").expect("?");
    pty.wait_until(5000, |s| !s.contains("move up"))
        .expect("collapsed");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn table_navigation() {
    let pty = PtySession::spawn(&ex("table"), &[]).expect("spawn");
    pty.wait_for_text("Rank", 5000).expect("table shown");
    // Down navigates without crashing (the selection highlight is color-only,
    // which the screen reconstruction strips).
    pty.press("down").expect("down");
    pty.press("down").expect("down");
    pty.press("up").expect("up");
    std::thread::sleep(std::time::Duration::from_millis(300));
    pty.wait_for_text("Rank", 5000).expect("still shown");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn resize_updates_layout() {
    let mut pty = PtySession::spawn(&ex("paginator"), &[]).expect("spawn");
    pty.wait_for_text("Item 1", 5000).expect("initial");
    // Resize the terminal: the example must not crash and must re-render.
    pty.resize(30, 40).expect("resize");
    std::thread::sleep(std::time::Duration::from_millis(500));
    pty.wait_for_text("Item 1", 5000).expect("still renders");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn spinner_animates() {
    let pty = PtySession::spawn(&ex("spinner"), &[]).expect("spawn");
    // The spinner cycles through frames; the rendered cell must change.
    std::thread::sleep(std::time::Duration::from_millis(400));
    let s1 = pty.screen();
    std::thread::sleep(std::time::Duration::from_millis(400));
    let s2 = pty.screen();
    assert_ne!(s1.to_string(), s2.to_string(), "spinner should animate");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn stopwatch_toggle_and_reset() {
    let pty = PtySession::spawn(&ex("stopwatch"), &[]).expect("spawn");
    // The stopwatch starts running; the help line shows the toggle state.
    pty.wait_for_text("stop", 5000).expect("running");
    // 's' pauses it (the help line's diff rewrites "stop" to "start"; the
    // timer line uses tab/backspace rendering the screen reconstruction
    // cannot follow, so assert on the raw diff).
    pty.press("s").expect("s");
    pty.wait_for_raw("art", 5000).expect("paused");
    pty.press("s").expect("s");
    pty.wait_for_raw("stop", 5000).expect("running again");
    pty.press("q").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn textarea_typing_and_save() {
    let pty = PtySession::spawn(&ex("textarea"), &[]).expect("spawn");
    pty.wait_for_text("Tell me a story", 5000).expect("initial");
    pty.type_text("Hello world").expect("type");
    pty.wait_for_text("Hello world", 5000).expect("typed");
    // Multi-line input: enter splits the line, typing continues.
    pty.press("enter").expect("enter");
    pty.wait_for_text("┃   2", 5000).expect("line 2 created");
    pty.type_text("Line two").expect("type 2");
    pty.wait_for_text("Line two", 5000).expect("typed 2");
    pty.press("esc").expect("esc");
    std::thread::sleep(std::time::Duration::from_millis(200));
    pty.press("ctrl+c").expect("quit");
    pty.wait_for_exit(5000).expect("exit");
}

#[test]
fn textinputs_three_fields() {
    let pty = PtySession::spawn(&ex("textinputs"), &[]).expect("spawn");
    pty.wait_for_text("> N", 5000).expect("field 1");
    pty.type_text("Alice").expect("type name");
    pty.wait_for_text("Alice", 5000).expect("name typed");
    pty.press("tab").expect("tab");
    pty.type_text("alice@example.com").expect("type email");
    pty.wait_for_text("alice@example.com", 5000)
        .expect("email shown");
    pty.press("tab").expect("tab");
    pty.type_text("secret").expect("type password");
    pty.wait_for_text("••••••", 5000).expect("password echoed");
    pty.press("tab").expect("tab");
    // The submit button is focused: enter submits and quits.
    pty.press("enter").expect("submit");
    pty.wait_for_exit(5000).expect("exit");
}
