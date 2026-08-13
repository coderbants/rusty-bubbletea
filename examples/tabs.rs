//! Cleanroom Rust port of upstream Go example: `examples/tabs/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program demonstrating a tabbed interface with styled tab headers.

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::border::{self, Border};
use rusty_lipgloss::color::light_dark;
use rusty_lipgloss::join::join_horizontal;
use rusty_lipgloss::size;
use rusty_lipgloss::{new_style, Color, Style, CENTER, TOP};

/// Styles used by the example.
#[derive(Clone)]
struct Styles {
    doc: Style,
    /// Present for upstream struct parity; the upstream example declares it
    /// but never sets or uses it.
    #[allow(dead_code)]
    highlight: Style,
    inactive_tab: Style,
    active_tab: Style,
    window: Style,
}

/// Renders a lipgloss Color back into the string form accepted by
/// `border_foreground` (a hex string for TrueColor, the index for ANSI
/// colors).
fn color_to_str(c: &Color) -> String {
    match c {
        Color::TrueColor { r, g, b } => format!("#{:02X}{:02X}{:02X}", r, g, b),
        Color::Ansi256(i) => i.to_string(),
        Color::Ansi16(i) => i.to_string(),
        _ => String::new(),
    }
}

/// Builds the styles, mirroring `newStyles` in the upstream example.
fn new_styles(bg_is_dark: bool) -> Styles {
    let light_dark = light_dark(bg_is_dark);

    let inactive_tab_border = tab_border_with_bottom("┴", "─", "┴");
    let active_tab_border = tab_border_with_bottom("┘", " ", "└");
    let highlight_color = light_dark(Color::parse("#874BFD"), Color::parse("#7D56F4"));
    let highlight_hex = color_to_str(&highlight_color);

    let doc = new_style().padding(&[1, 2, 1, 2]);
    let inactive_tab = new_style()
        .border(inactive_tab_border, &[true, true, true, true])
        .border_foreground(&[highlight_hex.as_str()])
        .padding(&[0, 1]);
    let active_tab = inactive_tab
        .clone()
        .border(active_tab_border, &[true, true, true, true]);
    let window = new_style()
        .border_foreground(&[highlight_hex.as_str()])
        .padding(&[2, 0])
        .align(&[CENTER])
        .border(border::normal_border(), &[true, true, true, true])
        .unset_border_top();

    Styles {
        doc,
        highlight: new_style(),
        inactive_tab,
        active_tab,
        window,
    }
}

/// A rounded border with custom bottom edge characters, mirroring
/// `tabBorderWithBottom`.
fn tab_border_with_bottom(left: &str, middle: &str, right: &str) -> Border {
    let mut border = border::rounded_border();
    border.bottom_left = left.to_string();
    border.bottom = middle.to_string();
    border.bottom_right = right.to_string();
    border
}

/// The main model: a list of tabs and their content.
struct Model {
    tabs: Vec<String>,
    tab_content: Vec<String>,
    styles: Option<Styles>,
    active_tab: usize,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "q" => return quit(),
                "right" | "l" | "n" | "tab" => {
                    self.active_tab = (self.active_tab + 1).min(self.tabs.len() - 1);
                    return None;
                }
                "left" | "h" | "p" | "shift+tab" => {
                    self.active_tab = self.active_tab.saturating_sub(1);
                    return None;
                }
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> View {
        if self.styles.is_none() {
            return View::new("");
        }

        let s = self.styles.as_ref().unwrap();

        let mut rendered_tabs: Vec<String> = Vec::new();
        for (i, t) in self.tabs.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == self.tabs.len() - 1;
            let is_active = i == self.active_tab;

            let base = if is_active {
                s.active_tab.clone()
            } else {
                s.inactive_tab.clone()
            };

            let (mut border, _, _, _, _) = base.get_border();
            if is_first && is_active {
                border.bottom_left = "│".to_string();
            } else if is_first && !is_active {
                border.bottom_left = "├".to_string();
            } else if is_last && is_active {
                border.bottom_right = "│".to_string();
            } else if is_last && !is_active {
                border.bottom_right = "┤".to_string();
            }
            let style = base.border(border, &[true, true, true, true]);

            rendered_tabs.push(style.render(t));
        }

        let refs: Vec<&str> = rendered_tabs.iter().map(|s| s.as_str()).collect();
        let row = join_horizontal(TOP, &refs);

        let mut doc = String::new();
        doc.push_str(&row);
        doc.push('\n');
        doc.push_str(
            &s.window
                .clone()
                .width(size::width(&row))
                .render(&self.tab_content[self.active_tab]),
        );

        View::new(&s.doc.render(&doc))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tabs = vec![
        "Lip Gloss".to_string(),
        "Blush".to_string(),
        "Eye Shadow".to_string(),
        "Mascara".to_string(),
        "Foundation".to_string(),
    ];
    let tab_content = vec![
        "Lip Gloss Tab".to_string(),
        "Blush Tab".to_string(),
        "Eye Shadow Tab".to_string(),
        "Mascara Tab".to_string(),
        "Foundation Tab".to_string(),
    ];
    let m = Model {
        tabs,
        tab_content,
        styles: Some(new_styles(true)), // default to dark styles.
        active_tab: 0,
    };
    let p = Program::new(m);
    p.run()?;
    Ok(())
}
