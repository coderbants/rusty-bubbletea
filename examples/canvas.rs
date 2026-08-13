//! Cleanroom Rust port of upstream Go example: `examples/canvas/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A program that renders two overlapping cards on a canvas, swapping their
//! z-order on any key press.
//!
//! Deviations from upstream:
//! - `github.com/charmbracelet/x/exp/charmtone` is inlined as hex colors:
//!   `charmtone.Oyster` is `#605F6B` and `charmtone.Charple` is `#6B50FF`.
//! - The generic `reverse` function is written as a small `reverse` helper on
//!   `Vec<isize>`.

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::layer::{new_compositor, new_layer, Layer};
use rusty_lipgloss::{new_style, Border, BOTTOM, CENTER};

/// `charmtone.Oyster`.
const OYSTER: &str = "#605F6B";

/// `charmtone.Charple`.
const CHARPLE: &str = "#6B50FF";

struct Model {
    width: usize,
    flip: bool,
    quitting: bool,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" | "esc" => {
                    self.quitting = true;
                    return quit();
                }
                _ => {
                    self.flip = !self.flip;
                }
            }
        }
        None
    }

    fn view(&self) -> View {
        let view = View::default();
        if self.quitting {
            return view;
        }

        let mut z: Vec<isize> = vec![0, 1];
        if self.flip {
            z = reverse(z);
        }

        let footer = new_style()
            .height(13)
            .foreground_color(rusty_lipgloss::Color::parse(OYSTER))
            .align_vertical(BOTTOM)
            .render("Press any key to swap the cards, or q to quit.");

        let card_a = new_card("Hello").z(z[0]);
        let card_b = new_card("Goodbye").z(z[1]);
        let mut comp = new_compositor(&[new_layer(&footer, &[]), card_a, card_b.x(10).y(2)]);

        let mut view = View::default();
        view.set_content(&comp.render());
        view
    }
}

/// Returns a new card layer, mirroring `newCard`.
fn new_card(s: &str) -> Layer {
    new_layer(
        &new_style()
            .width(20)
            .height(10)
            .border(Border::rounded(), &[])
            .border_foreground(&[CHARPLE])
            .align(&[CENTER, CENTER])
            .render(s),
        &[],
    )
}

/// Reverses a slice, returning a new slice, mirroring `reverse[T any]`.
fn reverse(s: Vec<isize>) -> Vec<isize> {
    let n = s.len();
    let mut r = vec![0; n];
    for (i, v) in s.iter().enumerate() {
        r[n - 1 - i] = *v;
    }
    r
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        width: 0,
        flip: false,
        quitting: false,
    });
    p.run()?;
    Ok(())
}
