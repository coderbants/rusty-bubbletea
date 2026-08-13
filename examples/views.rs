//! Cleanroom Rust port of upstream Go example: `examples/views/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! An example demonstrating an application with multiple views.
//!
//! Note that this example was produced before the Bubbles progress component
//! was available (github.com/charmbracelet/bubbles/progress) and thus, we're
//! implementing a progress bar from scratch here.
//!
//! Deviations from upstream:
//! - `ease.OutBounce` (fogleman/ease) is inlined as the standard outBounce
//!   easing formula.

use std::time::Duration;

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, tick, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{new_style, Color, Style};

/// Constants used for the progress bar.
const PROGRESS_BAR_WIDTH: usize = 71;
const PROGRESS_FULL_CHAR: &str = "█";
const PROGRESS_EMPTY_CHAR: &str = "░";
const DOT_CHAR: &str = " • ";

fn dot_style() -> Style {
    new_style().foreground("236")
}

/// General styles, mirroring the upstream global styles.
fn keyword_style() -> Style {
    new_style().foreground_color(Color::parse("211"))
}

fn subtle_style() -> Style {
    new_style().foreground_color(Color::parse("241"))
}

fn ticks_style() -> Style {
    new_style().foreground_color(Color::parse("79"))
}

fn checkbox_style() -> Style {
    new_style().foreground_color(Color::parse("212"))
}

fn main_style() -> Style {
    new_style().margin_left(2)
}

/// The gradient colors for the progress bar, mirroring `makeRampStyles`
/// ("#B14FFF" -> "#00FFA3" over the progress bar width).
fn ramp() -> Vec<Style> {
    let mut styles = Vec::new();
    // Mirror upstream `makeRampStyles`: blend in CIE-L*u*v* (go-colorful's
    // `BlendLuv`) and round-trip each color through a truncated hex
    // string (`colorToHex` uses `int64(f * 255)`).
    let mut i = 0.0;
    while i < PROGRESS_BAR_WIDTH as f64 {
        let (r, g, b) = rusty_lipgloss::blending::blend_luv_rgb(
            &Color::parse("#B14FFF"),
            &Color::parse("#00FFA3"),
            i / PROGRESS_BAR_WIDTH as f64,
        );
        let hex = format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0).trunc() as u8,
            (g * 255.0).trunc() as u8,
            (b * 255.0).trunc() as u8
        );
        styles.push(new_style().foreground(hex.as_str()));
        i += 1.0;
    }
    styles
}

/// Messages used to drive the example.
#[derive(Debug)]
struct TickMsg;

/// Messages used to drive the example.
#[derive(Debug)]
struct FrameMsg;

/// A tick once per second, mirroring `tick()`.
fn tick_cmd() -> Cmd {
    tick(Duration::from_secs(1), |_| Some(Box::new(TickMsg)))
}

/// A tick at 60fps, mirroring `frame()`.
fn frame_cmd() -> Cmd {
    tick(Duration::from_millis(1000 / 60), |_| {
        Some(Box::new(FrameMsg))
    })
}

/// The main model.
struct Model {
    choice: usize,
    chosen: bool,
    ticks: usize,
    frames: usize,
    progress: f64,
    loaded: bool,
    quitting: bool,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        tick_cmd()
    }

    /// Main update function.
    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        // Make sure these keys always quit.
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "q" || s == "esc" || s == "ctrl+c" {
                self.quitting = true;
                return quit();
            }
        }

        // Hand off the message to the appropriate update function for the
        // appropriate view based on the current state.
        if !self.chosen {
            self.update_choices(msg)
        } else {
            self.update_chosen(msg)
        }
    }

    /// The main view, which just calls the appropriate sub-view.
    fn view(&self) -> View {
        if self.quitting {
            return View::new("\n  See you later!\n\n");
        }
        let s = if !self.chosen {
            choices_view(self)
        } else {
            chosen_view(self)
        };
        View::new(&main_style().render(&format!("\n{}\n", s)))
    }
}

impl Model {
    /// Update loop for the first view where you're choosing a task.
    fn update_choices(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "j" | "down" => self.choice = (self.choice + 1).min(3),
                "k" | "up" => self.choice = self.choice.saturating_sub(1),
                "enter" => {
                    self.chosen = true;
                    return frame_cmd();
                }
                _ => {}
            }
        } else if msg.as_any().is::<TickMsg>() {
            if self.ticks == 0 {
                self.quitting = true;
                return quit();
            }
            self.ticks -= 1;
            return tick_cmd();
        }

        None
    }

    /// Update loop for the second view after a choice has been made.
    fn update_chosen(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().is::<FrameMsg>() {
            if !self.loaded {
                self.frames += 1;
                self.progress = out_bounce(self.frames as f64 / 100.0);
                if self.progress >= 1.0 {
                    self.progress = 1.0;
                    self.loaded = true;
                    self.ticks = 3;
                    return tick_cmd();
                }
                return frame_cmd();
            }
        } else if msg.as_any().is::<TickMsg>() && self.loaded {
            if self.ticks == 0 {
                self.quitting = true;
                return quit();
            }
            self.ticks -= 1;
            return tick_cmd();
        }

        None
    }
}

/// The first view, where you're choosing a task.
fn choices_view(m: &Model) -> String {
    let c = m.choice;

    let choices = format!(
        "{}\n{}\n{}\n{}",
        checkbox("Plant carrots", c == 0),
        checkbox("Go to the market", c == 1),
        checkbox("Read something", c == 2),
        checkbox("See friends", c == 3),
    );

    let out = format!(
        "What to do today?\n\n{}\n\nProgram quits in {} seconds\n\n{}{}{}{}{}",
        choices,
        ticks_style().render(&m.ticks.to_string()),
        subtle_style().render("j/k, up/down: select"),
        dot_style().render(DOT_CHAR),
        subtle_style().render("enter: choose"),
        dot_style().render(DOT_CHAR),
        subtle_style().render("q, esc: quit"),
    );
    out
}

/// The second view, after a task has been chosen.
fn chosen_view(m: &Model) -> String {
    let msg = match m.choice {
        0 => format!(
            "Carrot planting?\n\nCool, we'll need {} and {}...",
            keyword_style().render("libgarden"),
            keyword_style().render("vegeutils")
        ),
        1 => format!(
            "A trip to the market?\n\nOkay, then we should install {} and {}...",
            keyword_style().render("marketkit"),
            keyword_style().render("libshopping")
        ),
        2 => format!(
            "Reading time?\n\nOkay, cool, then we'll need a library. Yes, an {}.",
            keyword_style().render("actual library")
        ),
        _ => format!(
            "It's always good to see friends.\n\nFetching {} and {}...",
            keyword_style().render("social-skills"),
            keyword_style().render("conversationutils")
        ),
    };

    let label = if m.loaded {
        format!(
            "Downloaded. Exiting in {} seconds...",
            ticks_style().render(&m.ticks.to_string())
        )
    } else {
        "Downloading...".to_string()
    };

    format!("{}\n\n{}\n{}{}", msg, label, progressbar(m.progress), "%")
}

/// Renders a checkbox, checked or unchecked.
fn checkbox(label: &str, checked: bool) -> String {
    if checked {
        checkbox_style().render(&format!("[x] {}", label))
    } else {
        format!("[ ] {}", label)
    }
}

/// Renders a gradient progress bar at the given percentage.
fn progressbar(percent: f64) -> String {
    let w = PROGRESS_BAR_WIDTH as f64;

    let full_size = (w * percent).round() as usize;
    let ramp = ramp();
    let mut full_cells = String::new();
    for r in ramp.iter().take(full_size) {
        full_cells += &r.render(PROGRESS_FULL_CHAR);
    }

    let empty_size = PROGRESS_BAR_WIDTH - full_size;
    let empty_cells = subtle_style()
        .render(PROGRESS_EMPTY_CHAR)
        .repeat(empty_size);

    format!(
        "{}{} {:3.0}",
        full_cells,
        empty_cells,
        (percent * 100.0).round()
    )
}

/// OutBounce easing, mirroring `ease.OutBounce` from fogleman/ease.
fn out_bounce(x: f64) -> f64 {
    const N1: f64 = 7.5625;
    const D1: f64 = 2.75;
    if x < 1.0 / D1 {
        N1 * x * x
    } else if x < 2.0 / D1 {
        N1 * (x - 1.5 / D1) * (x - 1.5 / D1) + 0.75
    } else if x < 2.5 / D1 {
        N1 * (x - 2.25 / D1) * (x - 2.25 / D1) + 0.9375
    } else {
        N1 * (x - 2.625 / D1) * (x - 2.625 / D1) + 0.984375
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let initial_model = Model {
        choice: 0,
        chosen: false,
        ticks: 10,
        frames: 0,
        progress: 0.0,
        loaded: false,
        quitting: false,
    };
    let p = Program::new(initial_model);
    p.run()?;
    Ok(())
}
