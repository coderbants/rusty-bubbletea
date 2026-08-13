//! Cleanroom Rust port of upstream Go example: `examples/file-picker/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple file picker. Navigate with the arrow keys, enter a directory with
//! `enter` (or the right arrow) and select a file with `enter`. Press `q` or
//! `ctrl+c` to quit.

use std::time::Duration;

use rusty_bubbles::filepicker;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, tick, Cmd, KeyPressMsg, Msg, Program, View};

/// ClearErrorMsg is sent after a short delay to clear the displayed error.
#[derive(Debug)]
struct ClearErrorMsg;

/// ClearErrorAfter returns a command that delivers a ClearErrorMsg after the
/// given duration.
fn clear_error_after(t: Duration) -> Cmd {
    tick(t, |_| Some(Box::new(ClearErrorMsg)))
}

struct Model {
    filepicker: filepicker::Model,
    selected_file: String,
    quitting: bool,
    err: Option<String>,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        self.filepicker.init()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "ctrl+c" || s == "q" {
                self.quitting = true;
                return quit();
            }
        }
        if msg.as_any().is::<ClearErrorMsg>() {
            self.err = None;
        }

        let cmd = self.filepicker.update(msg);

        // Did the user select a file?
        if let (true, path) = self.filepicker.did_select_file(msg) {
            // Get the path of the selected file.
            self.selected_file = path;
        }

        // Did the user select a disabled file?
        // This is only necessary to display an error to the user.
        if let (true, path) = self.filepicker.did_select_disabled_file(msg) {
            // Let's clear the selectedFile and display an error.
            self.err = Some(path + " is not valid.");
            self.selected_file = String::new();
            return batch(vec![cmd, clear_error_after(2 * Duration::from_secs(1))]);
        }

        cmd
    }

    fn view(&self) -> View {
        if self.quitting {
            return View::new("");
        }

        let mut s = String::from("\n  ");
        if let Some(err) = &self.err {
            s += &self.filepicker.styles.disabled_file.render(err);
        } else if self.selected_file.is_empty() {
            s += "Pick a file:";
        } else {
            s += &("Selected file: ".to_string()
                + &self.filepicker.styles.selected.render(&self.selected_file));
        }
        s += &("\n\n".to_string() + &self.filepicker.view() + "\n");

        let mut v = View::new(&s);
        v.alt_screen = true;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fp = filepicker::new();
    fp.allowed_types = vec![
        ".mod".to_string(),
        ".sum".to_string(),
        ".go".to_string(),
        ".txt".to_string(),
        ".md".to_string(),
    ];
    fp.current_directory = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    let m = Model {
        filepicker: fp,
        selected_file: String::new(),
        quitting: false,
        err: None,
    };

    let mm = Program::new(m).run()?;
    println!(
        "\n  You selected: {}\n",
        mm.filepicker.styles.selected.render(&mm.selected_file)
    );
    Ok(())
}
