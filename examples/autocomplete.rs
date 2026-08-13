//! Cleanroom Rust port of upstream Go example: `examples/autocomplete/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! An autocomplete example that fetches the Charmbracelet organization's
//! repositories from the GitHub API and suggests them as you type. Press
//! `tab` to complete, `ctrl+n`/`ctrl+p` to cycle through suggestions and
//! `esc` to quit.
//!
//! NOTE: the upstream example uses `net/http` to fetch the repositories. This
//! port shells out to `curl` (via `std::process::Command`) instead, since no
//! HTTP client is available in the dependency tree.

use rusty_bubbles::help;
use rusty_bubbles::key;
use rusty_bubbles::textinput;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::join::join_vertical;
use rusty_lipgloss::{new_style, Color, LEFT};

const REPOS_URL: &str = "https://api.github.com/orgs/charmbracelet/repos";

/// GotReposSuccessMsg contains the fetched repository names.
#[derive(Debug)]
struct GotReposSuccessMsg(Vec<String>);

/// GotReposErrMsg reports a failure fetching the repositories. Mirroring the
/// upstream, this message is intentionally ignored by the model.
#[derive(Debug)]
#[allow(dead_code)]
struct GotReposErrMsg(String);

/// GetRepos fetches the list of Charmbracelet repositories, mirroring the
/// upstream `getRepos` command.
fn get_repos() -> Cmd {
    Some(Box::new(|| {
        let out = std::process::Command::new("curl")
            .args([
                "-s",
                "-f",
                "-H",
                "Accept: application/vnd.github+json",
                "-H",
                "X-GitHub-Api-Version: 2022-11-28",
                REPOS_URL,
            ])
            .output();

        match out {
            Ok(o) if o.status.success() => {
                let body = String::from_utf8_lossy(&o.stdout).to_string();
                let names = parse_repo_names(&body);
                if names.is_empty() {
                    return Some(Box::new(GotReposErrMsg(
                        "could not parse repositories".to_string(),
                    )));
                }
                Some(Box::new(GotReposSuccessMsg(names)))
            }
            _ => Some(Box::new(GotReposErrMsg(
                "could not fetch repositories".to_string(),
            ))),
        }
    }))
}

/// ParseRepoNames extracts the `"name"` string values from a GitHub API
/// repositories JSON array, mirroring the upstream `json.Unmarshal` into
/// `[]repo`.
fn parse_repo_names(body: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find the next `"name"` key.
        if bytes[i..].starts_with(b"\"name\"") {
            let after = i + 6;
            // Skip whitespace and the colon.
            let mut j = after;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b':' {
                j += 1;
                while j < bytes.len()
                    && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n')
                {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'"' {
                    j += 1;
                    let mut value = String::new();
                    while j < bytes.len() {
                        let c = bytes[j];
                        if c == b'\\' && j + 1 < bytes.len() {
                            value.push(bytes[j + 1] as char);
                            j += 2;
                            continue;
                        }
                        if c == b'"' {
                            break;
                        }
                        value.push(c as char);
                        j += 1;
                    }
                    names.push(value);
                }
            }
            i = j.saturating_add(1);
        } else {
            i += 1;
        }
    }
    names
}

/// Keymap holds the keybindings for this program, satisfying the
/// `help::KeyMap` interface.
struct Keymap {
    complete: key::Binding,
    next: key::Binding,
    prev: key::Binding,
    quit: key::Binding,
}

impl help::KeyMap for Keymap {
    fn short_help(&self) -> Vec<key::Binding> {
        vec![
            self.complete.clone(),
            self.next.clone(),
            self.prev.clone(),
            self.quit.clone(),
        ]
    }

    fn full_help(&self) -> Vec<Vec<key::Binding>> {
        vec![self.short_help()]
    }
}

struct Model {
    text_input: textinput::Model,
    help: help::Model,
    keymap: Keymap,
}

impl Model {
    fn initial_model() -> Self {
        let mut ti = textinput::new();
        ti.prompt = "charmbracelet/".to_string();

        let mut s = ti.styles().clone();
        s.focused.prompt = new_style()
            .foreground_color(Color::parse("63"))
            .margin_left(2);
        s.cursor.color = Color::parse("63");
        ti.set_styles(s);

        ti.set_virtual_cursor(false);
        let _ = ti.focus();
        ti.char_limit = 50;
        ti.set_width(20);
        ti.show_suggestions = true;

        let km = Keymap {
            complete: key::new_binding(vec![
                key::with_keys(&["tab"]),
                key::with_help("tab", "complete"),
                key::with_disabled(),
            ]),
            next: key::new_binding(vec![
                key::with_keys(&["ctrl+n"]),
                key::with_help("ctrl+n", "next"),
                key::with_disabled(),
            ]),
            prev: key::new_binding(vec![
                key::with_keys(&["ctrl+p"]),
                key::with_help("ctrl+p", "prev"),
                key::with_disabled(),
            ]),
            quit: key::new_binding(vec![
                key::with_keys(&["enter", "ctrl+c", "esc"]),
                key::with_help("esc", "quit"),
            ]),
        };

        Model {
            text_input: ti,
            keymap: km,
            help: help::new(),
        }
    }

    fn header_view(&self) -> String {
        "Enter a Charm™ repo:\n".to_string()
    }

    fn footer_view(&self) -> String {
        "\n".to_string() + &self.help.view(&self.keymap)
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        batch(vec![
            get_repos(),
            Some(Box::new(|| Some(textinput::blink()))),
        ])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(got) = msg.as_any().downcast_ref::<GotReposSuccessMsg>() {
            let suggestions: Vec<String> = got.0.clone();
            self.text_input.set_suggestions(&suggestions);
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if key::matches(&k.0, std::slice::from_ref(&self.keymap.quit)) {
                return quit();
            }
        }

        let cmd = self.text_input.update(msg);

        // Determine whether to show completion keybindings.
        let has_choices = self.text_input.matched_suggestions().len() > 1;
        self.keymap.complete.set_enabled(has_choices);
        self.keymap.next.set_enabled(has_choices);
        self.keymap.prev.set_enabled(has_choices);

        cmd
    }

    fn view(&self) -> View {
        if self.text_input.available_suggestions().is_empty() {
            return View::new("One sec, we're fetching completions...");
        }

        let str = join_vertical(
            LEFT,
            &[
                &self.header_view(),
                &self.text_input.view(),
                &self.footer_view(),
            ],
        );

        let mut v = View::new(&str);
        if let Some(mut c) = self.text_input.cursor() {
            c.position.y += rusty_lipgloss::size::height(&self.header_view());
            v.cursor = Some(c);
        }
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
