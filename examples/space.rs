//! Cleanroom Rust port of upstream Go example: `examples/space/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! An example to show the FPS count of a moving space-like background.
//!
//! This was ported from the talented Orhun Parmaksız (@orhun)'s space example
//! from his blog post "Why stdout is faster than stderr?".
//!
//! Deviations from upstream:
//! - `math/rand` is replaced with a small xorshift64* PRNG (no external
//!   dependency), seeded from the system clock.
//! - The `image/color` grayscale values are `rusty_lipgloss::Color`s.
//! - `tea.WithFPS(120)` is applied via `ProgramOptions::default().with_fps(120)`.

use std::time::{Duration, SystemTime};

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::options::ProgramOptions;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, tick, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{new_style, Color};

/// A tiny xorshift64* PRNG seeded from the system clock, used in place of
/// Go's `math/rand` (deviation: no external rand dependency).
struct Rng(u64);

impl Rng {
    fn seeded() -> Rng {
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A random float in [0, 1), mirroring `rand.Float64()`.
    fn float64(&mut self) -> f64 {
        ((self.next() >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

/// Messages are events that we respond to in our Update function. This
/// particular one indicates that the timer has ticked.
#[derive(Debug)]
struct TickMsg;

/// A tick command at 60fps, mirroring `tickCmd()`.
fn tick_cmd() -> Cmd {
    tick(Duration::from_micros(1_000_000 / 60), |_| {
        Some(Box::new(TickMsg))
    })
}

struct Model {
    colors: Vec<Vec<Color>>,
    last_width: usize,
    last_height: usize,
    frame_count: usize,
    width: usize,
    height: usize,
    rng: Rng,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        tick_cmd()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" => return quit(),
                _ => {}
            }
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
            if self.width != self.last_width || self.height != self.last_height {
                self.setup_colors();
                self.last_width = self.width;
                self.last_height = self.height;
            }
        }

        if msg.as_any().is::<TickMsg>() {
            self.frame_count += 1;
            return tick_cmd();
        }

        None
    }

    fn view(&self) -> View {
        // Title
        let title = new_style().bold(true).render("Space");

        // Color display
        let mut s = String::new();
        let height = self.height.saturating_sub(1); // leave one line for title
        for y in 0..height {
            for x in 0..self.width {
                let xi = (x + self.frame_count) % self.width;
                let fg = self.colors[y * 2][xi].clone();
                let bg = self.colors[y * 2 + 1][xi].clone();
                let st = new_style().foreground_color(fg).background_color(bg);
                s.push_str(&st.render("▀"));
            }
            if y < height - 1 {
                s.push('\n');
            }
        }

        let mut v = View::new(&format!("{}\n{}", title, s));
        v.alt_screen = true;
        v
    }
}

impl Model {
    fn setup_colors(&mut self) {
        let height = self.height * 2; // double height for half blocks
        self.colors = vec![vec![Color::NoColor; self.width]; height];

        for y in 0..height {
            let randomness_factor = (height - y) as f64 / height as f64;

            for x in 0..self.width {
                let base_value = randomness_factor * (height - y) as f64 / height as f64;
                let random_offset = (self.rng.float64() * 0.2) - 0.1;
                let value = clamp(base_value + random_offset, 0.0, 1.0);

                // Convert value to grayscale color (0-255)
                let gray = (value * 255.0) as u8;
                self.colors[y][x] = Color::parse(&format!("#{:02x}{:02x}{:02x}", gray, gray, gray));
            }
        }
    }
}

/// Clamps the given value to [min, max], mirroring `clamp`.
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        colors: Vec::new(),
        last_width: 0,
        last_height: 0,
        frame_count: 0,
        width: 0,
        height: 0,
        rng: Rng::seeded(),
    })
    .with_options(ProgramOptions::default().with_fps(120));
    p.run()?;
    Ok(())
}
