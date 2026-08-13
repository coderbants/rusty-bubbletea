//! Cleanroom Rust port of upstream Go example: `examples/list-simple/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple list for choosing what to have for dinner. Press enter to select
//! an item, or q/ctrl+c to quit.

use std::any::Any;

use rusty_bubbles::list;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View, WindowSizeMsg};
use rusty_lipgloss::{new_style, Color, Style};

const LIST_HEIGHT: usize = 14;

#[derive(Clone)]
struct Styles {
    title: Style,
    item: Style,
    selected_item: Style,
    pagination: Style,
    help: Style,
    quit_text: Style,
}

impl Styles {
    fn new(dark_bg: bool) -> Self {
        let defaults = list::default_styles(dark_bg);
        Styles {
            title: new_style().margin_left(2),
            item: new_style().padding_left(4),
            selected_item: new_style()
                .padding_left(2)
                .foreground_color(Color::parse("170")),
            pagination: defaults.pagination_style.padding_left(4),
            help: defaults.help_style.padding_left(4).padding_bottom(1),
            quit_text: new_style().margin(&[1, 0, 2, 4]),
        }
    }
}

#[derive(Debug, Clone)]
struct Item(String);

impl list::Item for Item {
    fn filter_value(&self) -> String {
        String::new()
    }

    fn box_clone(&self) -> Box<dyn list::Item + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_default_item(&self) -> Option<&dyn list::DefaultItem> {
        Some(self)
    }
}

impl list::DefaultItem for Item {
    fn title(&self) -> String {
        self.0.clone()
    }

    fn description(&self) -> String {
        String::new()
    }
}

struct ItemDelegate {
    styles: Styles,
}

impl list::ItemDelegate for ItemDelegate {
    fn render(&self, m: &list::Model, index: usize, item: &dyn list::Item) -> String {
        let Some(i) = item.as_default_item() else {
            return String::new();
        };

        let s = format!("{}. {}", index + 1, i.title());

        if index == m.index() {
            return self.styles.selected_item.render(&format!("> {}", s));
        }
        self.styles.item.render(&s)
    }

    fn height(&self) -> usize {
        1
    }

    fn spacing(&self) -> usize {
        0
    }

    fn update(&self, _msg: &dyn Msg, _m: &list::Model) -> Cmd {
        None
    }
}

struct Model {
    list: list::Model,
    choice: String,
    styles: Styles,
    quitting: bool,
}

impl Model {
    fn new() -> Self {
        let items: Vec<Box<dyn list::Item + Send + Sync>> = vec![
            Box::new(Item("Ramen".to_string())),
            Box::new(Item("Tomato Soup".to_string())),
            Box::new(Item("Hamburgers".to_string())),
            Box::new(Item("Cheeseburgers".to_string())),
            Box::new(Item("Currywurst".to_string())),
            Box::new(Item("Okonomiyaki".to_string())),
            Box::new(Item("Pasta".to_string())),
            Box::new(Item("Fillet Mignon".to_string())),
            Box::new(Item("Caviar".to_string())),
            Box::new(Item("Just Wine".to_string())),
        ];

        const DEFAULT_WIDTH: usize = 20;

        let l = list::new(
            items,
            Box::new(ItemDelegate {
                styles: Styles::new(true),
            }),
            DEFAULT_WIDTH,
            LIST_HEIGHT,
        );

        let mut m = Model {
            list: l,
            choice: String::new(),
            styles: Styles::new(true),
            quitting: false,
        };
        // Mirror upstream: custom title, hidden status bar, filtering
        // disabled.
        m.list.title = "What do you want for dinner?".to_string();
        m.list.set_show_status_bar(false);
        m.list.set_filtering_enabled(false);
        m.update_styles(true); // default to dark styles.
        m
    }

    fn update_styles(&mut self, is_dark: bool) {
        self.styles = Styles::new(is_dark);
        self.list.styles.title = self.styles.title.clone();
        self.list.styles.pagination_style = self.styles.pagination.clone();
        self.list.styles.help_style = self.styles.help.clone();
        self.list.set_delegate(Box::new(ItemDelegate {
            styles: self.styles.clone(),
        }));
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.list.set_width(ws.width);
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" => {
                    self.quitting = true;
                    return quit();
                }
                "enter" => {
                    if let Some(i) = self.list.selected_item() {
                        if let Some(item) = i.as_any().downcast_ref::<Item>() {
                            self.choice = item.0.clone();
                        }
                    }
                    return quit();
                }
                _ => {}
            }
        }

        self.list.update(msg)
    }

    fn view(&self) -> View {
        if !self.choice.is_empty() {
            return View::new(
                &self
                    .styles
                    .quit_text
                    .render(&format!("{}? Sounds good to me.", self.choice)),
            );
        }
        if self.quitting {
            return View::new(&self.styles.quit_text.render("Not hungry? That’s cool."));
        }
        View::new(&format!("\n{}", self.list.view()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
