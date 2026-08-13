//! Cleanroom Rust port of upstream Go example: `examples/paginator/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program demonstrating the paginator component from the Bubbles
//! component library.

use rusty_bubbles::paginator::{self, Type};
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{
    quit, request_background_color, BackgroundColorMsg, Cmd, KeyPressMsg, Msg, Program, View,
};
use rusty_lipgloss::{Color, Style};

fn new_styles(bg_is_dark: bool) -> (String, String) {
    let light_dark = rusty_lipgloss::color::light_dark(bg_is_dark);

    let active_dot = Style::new()
        .foreground_color(light_dark(Color::parse("235"), Color::parse("252")))
        .set_string(&["•"])
        .string();
    let inactive_dot = Style::new()
        .foreground_color(light_dark(Color::parse("250"), Color::parse("238")))
        .set_string(&["•"])
        .string();
    (active_dot, inactive_dot)
}

struct Model {
    items: Vec<String>,
    paginator: paginator::Model,
}

impl Model {
    fn new_model() -> Self {
        let items: Vec<String> = (1..=100).map(|i| format!("Item {}", i)).collect();

        let mut p = paginator::new(vec![]);
        p.type_ = Type::Dots;
        p.per_page = 10;
        p.set_total_pages(items.len());

        let mut m = Model {
            paginator: p,
            items,
        };
        m.update_styles(true); // default to dark styles
        m
    }

    fn update_styles(&mut self, is_dark: bool) {
        let (active, inactive) = new_styles(is_dark);
        self.paginator.active_dot = active;
        self.paginator.inactive_dot = inactive;
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        request_background_color()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(bg) = msg.as_any().downcast_ref::<BackgroundColorMsg>() {
            self.update_styles(bg.is_dark());
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "esc" | "ctrl+c" => return quit(),
                _ => {}
            }
        }

        self.paginator.update(msg)
    }

    fn view(&self) -> View {
        let mut b = String::from("\n  Paginator Example\n\n");
        let (start, end) = self.paginator.get_slice_bounds(self.items.len());
        for item in &self.items[start..end] {
            b += &format!("  • {}\n\n", item);
        }
        b += "  ";
        b += &self.paginator.view();
        b += "\n\n  h/l ←/→ page • q: quit\n";
        View::new(&b)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new_model());
    p.run()?;
    Ok(())
}
