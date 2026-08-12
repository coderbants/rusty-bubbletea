//! Cleanroom Rust port of upstream Go example: `examples/doom-fire/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! This Doom Fire implementation was ported from @const-void's Node version.
//! See <https://github.com/const-void/DOOM-fire-node>
//!
//! Deviations from upstream:
//! - `math/rand` is replaced with a small xorshift64* PRNG (no external
//!   dependency), seeded from the system clock.
//! - `lipgloss.White` (ANSI basic color 7) is inlined as `Color::parse("7")`.
//! - The ANSI palette entries are applied as `Color::Ansi256`.

use std::time::{Duration, SystemTime};

use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{quit, tick, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::{new_style, Color};

/// Same color palette as the original.
const FIRE_PALETTE: [i64; 26] = [
    0, 233, 234, 52, 53, 88, 89, 94, 95, 96, 130, 131, 132, 133, 172, 214, 215, 220, 220, 221, 3,
    226, 227, 230, 231, 7,
];

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

    /// Random integer in [0, n), mirroring `rand.Intn(n)`.
    fn intn(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Messages are events that we respond to in our Update function. This
/// particular one indicates that the timer has ticked.
#[derive(Debug)]
struct TickMsg;

/// A tick command at 50ms (20fps), mirroring `tick()`.
fn tick_cmd() -> Cmd {
    tick(Duration::from_millis(50), |_| Some(Box::new(TickMsg)))
}

struct Model {
    screen_buf: Vec<i32>,
    width: usize,
    height: usize,
    fire_palette: Vec<i64>,
    start_time: SystemTime,
    rng: Rng,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        tick_cmd()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "q" || s == "ctrl+c" {
                return quit();
            }
        }

        if msg.as_any().is::<TickMsg>() {
            self.spread_fire();
            return tick_cmd();
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height * 2; // Double height for half-block characters
            self.screen_buf = vec![0; self.width * self.height];
            // Initialize the bottom row with white (maximum intensity)
            for i in 0..self.width {
                self.screen_buf[(self.height - 1) * self.width + i] =
                    (self.fire_palette.len() - 1) as i32;
            }
        }

        None
    }

    fn view(&self) -> View {
        if self.width == 0 {
            return View::new("Initializing...");
        }

        let mut s = String::new();

        let mut y = 0;
        while y < self.height - 2 {
            for x in 0..self.width {
                let pixel_hi = self.screen_buf[y * self.width + x];
                let pixel_lo = self.screen_buf[(y + 1) * self.width + x];

                let hi_color = self.fire_palette[pixel_hi as usize] as u8;
                let lo_color = self.fire_palette[pixel_lo as usize] as u8;

                s += &new_style()
                    .foreground_color(Color::Ansi256(hi_color))
                    .background_color(Color::Ansi256(lo_color))
                    .render("▀");
            }
            if y < self.height - 2 {
                s.push('\n');
            }
            y += 2;
        }

        let elapsed = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_default();
        s += &new_style().foreground("7").render(&format!(
            "Press q or ctrl+c to quit. Elapsed: {:?}",
            Duration::from_secs(elapsed.as_secs())
        ));

        let mut v = View::new(&s);
        v.alt_screen = true;
        v
    }
}

impl Model {
    fn spread_fire(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.spread_pixel((y * self.width + x) as i64);
            }
        }
    }

    fn spread_pixel(&mut self, idx: i64) {
        if idx < self.width as i64 {
            return;
        }

        let pixel = self.screen_buf[idx as usize];
        if pixel == 0 {
            self.screen_buf[(idx - self.width as i64) as usize] = 0;
            return;
        }

        let rnd = self.rng.intn(3) as i64;
        let dst = idx - rnd + 1;
        if dst - (self.width as i64) >= 0
            && dst - (self.width as i64) < self.screen_buf.len() as i64
        {
            let decay = rnd & 1;
            let new_value = (pixel - decay as i32).max(0);
            self.screen_buf[(dst - self.width as i64) as usize] = new_value;
        }
    }
}

fn initial_model() -> Model {
    Model {
        screen_buf: Vec::new(),
        width: 0,
        height: 0,
        fire_palette: FIRE_PALETTE.to_vec(),
        start_time: SystemTime::now(),
        rng: Rng::seeded(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(initial_model());
    p.run()?;
    Ok(())
}
