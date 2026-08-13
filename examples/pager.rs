//! Cleanroom Rust port of upstream Go example: `examples/pager/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! An example program demonstrating the pager component from the Bubbles
//! component library. Use the mouse wheel or the `pgup`/`pgdown`, `k`/`j`,
//! `u`/`d` and `g`/`G` keys to scroll through the content.

use rusty_bubbles::viewport;
use rusty_bubbles::viewport::GutterContext;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, MouseMode, Msg, Program, View};
use rusty_lipgloss::border::Border;
use rusty_lipgloss::join::join_horizontal;
use rusty_lipgloss::size;
use rusty_lipgloss::{new_style, Color, Style};

/// The content to display. The upstream example reads `artichoke.md` from
/// disk; this port embeds the same content at compile time.
const ARTICHOKE: &str = include_str!("artichoke.md");

/// TitleStyle is the style for the header line, mirroring the upstream
/// `titleStyle` global.
fn title_style() -> Style {
    let mut b = Border::rounded();
    b.right = "├".to_string();
    new_style()
        .border(b, &[true, true, true, true])
        .padding(&[0, 1])
}

/// InfoStyle is the style for the footer line, mirroring the upstream
/// `infoStyle` global.
fn info_style() -> Style {
    let mut b = Border::rounded();
    b.left = "┤".to_string();
    title_style().border(b, &[true, true, true, true])
}

struct Model {
    content: String,
    ready: bool,
    viewport: viewport::Model,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "ctrl+c" || s == "q" || s == "esc" {
                return quit();
            }
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            let header_height = size::height(&self.header_view());
            let footer_height = size::height(&self.footer_view());
            let vertical_margin_height = header_height + footer_height;

            if !self.ready {
                // Since this program is using the full size of the viewport we
                // need to wait until we've received the window dimensions
                // before we can initialize the viewport. The initial
                // dimensions come in quickly, though asynchronously, which is
                // why we wait for them here.
                self.viewport = viewport::new(vec![
                    viewport::with_width(ws.width),
                    viewport::with_height(ws.height.saturating_sub(vertical_margin_height)),
                ]);
                self.viewport.y_position = header_height;
                self.viewport.left_gutter_func = Some(Box::new(line_numbers));
                self.viewport.highlight_style = new_style()
                    .foreground_color(Color::parse("238"))
                    .background_color(Color::parse("34"));
                self.viewport.selected_highlight_style = new_style()
                    .foreground_color(Color::parse("238"))
                    .background_color(Color::parse("47"));
                self.viewport.set_content(&self.content);
                self.viewport
                    .set_highlights(&find_all_matches(&self.content, "artichoke"));
                self.viewport.highlight_next();
                self.ready = true;
            } else {
                self.viewport.set_width(ws.width);
                self.viewport
                    .set_height(ws.height.saturating_sub(vertical_margin_height));
            }
            return None;
        }

        // Handle keyboard and mouse events in the viewport.
        self.viewport.update(msg)
    }

    fn view(&self) -> View {
        let mut v;
        if !self.ready {
            v = View::new("\n  Initializing...");
        } else {
            v = View::new(&format!(
                "{}\n{}\n{}",
                self.header_view(),
                self.viewport.view(),
                self.footer_view()
            ));
        }
        v.alt_screen = true; // use the full size of the terminal in its "alternate screen buffer"
        v.mouse_mode = MouseMode::MouseModeCellMotion; // turn on mouse support so we can track the mouse wheel
        v
    }
}

impl Model {
    fn header_view(&self) -> String {
        let title = title_style().render("Mr. Pager");
        let line = "─".repeat(self.viewport.width().saturating_sub(size::width(&title)));
        join_horizontal(rusty_lipgloss::CENTER, &[&title, &line])
    }

    fn footer_view(&self) -> String {
        let info = info_style().render(&format!(
            "{:3.0}%:{:3.0}%",
            self.viewport.scroll_percent() * 100.0,
            self.viewport.horizontal_scroll_percent() * 100.0
        ));
        let line = "─".repeat(self.viewport.width().saturating_sub(size::width(&info)));
        join_horizontal(rusty_lipgloss::CENTER, &[&line, &info])
    }
}

/// LineNumbers renders the left gutter with line numbers, mirroring the
/// upstream `LeftGutterFunc`.
fn line_numbers(info: GutterContext) -> String {
    if info.soft {
        return "     │ ".to_string();
    }
    if info.index >= info.total_lines {
        return "   ~ │ ".to_string();
    }
    format!("{:4} │ ", info.index + 1)
}

/// FindAllMatches returns the byte ranges of every occurrence of `needle` in
/// `haystack`, mirroring `regexp.MustCompile(needle).FindAllStringIndex`.
fn find_all_matches(haystack: &str, needle: &str) -> Vec<Vec<usize>> {
    let mut matches: Vec<Vec<usize>> = Vec::new();
    let mut start = 0;
    while let Some(rel) = haystack[start..].find(needle) {
        let begin = start + rel;
        matches.push(vec![begin, begin + needle.len()]);
        start = begin + needle.len();
    }
    matches
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        content: ARTICHOKE.to_string(),
        ready: false,
        viewport: viewport::new(vec![]),
    });
    p.run()?;
    Ok(())
}
