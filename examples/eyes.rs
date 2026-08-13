//! Cleanroom Rust port of upstream Go example: `examples/eyes/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Roughly converted to Rust from
//! <https://github.com/dmtrKovalenko/esp32-smooth-eye-blinking/blob/main/src/main.cpp>
//!
//! Deviations from upstream:
//! - `math/rand` is replaced with a small xorshift64* PRNG (no external
//!   dependency), seeded from the system clock.
//! - `tea.KeyMsg` is received as `KeyPressMsg` (the v2.0.8 key message type).

use std::time::{Duration, SystemTime};

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, tick, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{new_style, Color};

// Eye dimensions (corresponding to original EYE_WIDTH and EYE_HEIGHT)
const EYE_WIDTH: isize = 15;
const EYE_HEIGHT: isize = 12; // Increased height for taller eyes
const EYE_SPACING: isize = 40;

// Blink animation timing (matching original constants)
const BLINK_FRAMES: usize = 20;
const OPEN_TIME_MIN: u64 = 1000;
const OPEN_TIME_MAX: u64 = 4000;

// Characters for drawing the eyes
const EYE_CHAR: &str = "●";
const BG_CHAR: &str = " ";

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

/// A tick command at 50ms, mirroring `tickCmd()`.
fn tick_cmd() -> Cmd {
    tick(Duration::from_millis(50), |_| Some(Box::new(TickMsg)))
}

struct Model {
    width: isize,
    height: isize,
    eye_positions: [isize; 2],
    eye_y: isize,
    is_blinking: bool,
    blink_state: usize,
    last_blink: SystemTime,
    open_time: Duration,
    rng: Rng,
}

impl Model {
    fn update_eye_positions(&mut self) {
        let start_x = (self.width - EYE_SPACING) / 2;
        self.eye_y = self.height / 2;

        self.eye_positions[0] = start_x;
        self.eye_positions[1] = start_x + EYE_SPACING;
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        tick_cmd()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "esc" => return quit(),
                _ => {}
            }
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width as isize;
            self.height = ws.height as isize;
            self.update_eye_positions();
        }

        if msg.as_any().is::<TickMsg>() {
            let current_time = SystemTime::now();

            if !self.is_blinking
                && current_time
                    .duration_since(self.last_blink)
                    .unwrap_or_default()
                    >= self.open_time
            {
                self.is_blinking = true;
                self.blink_state = 0;
            }

            if self.is_blinking {
                self.blink_state += 1;

                if self.blink_state >= BLINK_FRAMES {
                    self.is_blinking = false;
                    self.last_blink = current_time;
                    self.open_time = Duration::from_millis(
                        self.rng.intn(OPEN_TIME_MAX - OPEN_TIME_MIN) + OPEN_TIME_MIN,
                    );

                    // 10% chance of double blink (matching original logic)
                    if self.rng.intn(10) == 0 {
                        self.open_time = Duration::from_millis(300);
                    }
                }
            }
        }

        tick_cmd()
    }

    fn view(&self) -> View {
        let mut v = View {
            alt_screen: true,
            ..Default::default()
        }; // Use alternate screen buffer

        // Create empty canvas
        let mut canvas: Vec<Vec<&str>> =
            vec![vec![BG_CHAR; self.width as usize]; self.height as usize];

        // Calculate current eye height based on blink state
        let mut current_height = EYE_HEIGHT;
        if self.is_blinking {
            let mut blink_progress: f64;

            if self.blink_state < BLINK_FRAMES / 2 {
                // Closing eyes (with easing function from original)
                blink_progress = self.blink_state as f64 / (BLINK_FRAMES / 2) as f64;
                blink_progress = 1.0 - (blink_progress * blink_progress);
            } else {
                // Opening eyes (with easing function from original)
                blink_progress =
                    (self.blink_state - BLINK_FRAMES / 2) as f64 / (BLINK_FRAMES / 2) as f64;
                blink_progress = blink_progress * (2.0 - blink_progress);
            }

            current_height = ((EYE_HEIGHT as f64) * blink_progress).max(1.0) as isize;
        }

        // Draw both eyes
        for i in 0..2 {
            draw_ellipse(
                &mut canvas,
                self.eye_positions[i],
                self.eye_y,
                EYE_WIDTH,
                current_height,
            );
        }

        // Convert canvas to string
        let mut s = String::new();
        for row in &canvas {
            for cell in row {
                s.push_str(cell);
            }
            s.push('\n');
        }

        // Style output
        let style = new_style().foreground_color(Color::parse("#F0F0F0"));

        v.set_content(&style.render(&s));

        v
    }
}

/// Improved ellipse drawing algorithm with better angles, mirroring
/// `drawEllipse`.
fn draw_ellipse(canvas: &mut [Vec<&str>], x0: isize, y0: isize, rx: isize, ry: isize) {
    let mut y = -ry;
    while y <= ry {
        // Calculate the width at this y position for a smoother ellipse
        // Use a slightly modified formula to improve the angles
        let width = ((rx as f64) * (1.0 - (y as f64 / ry as f64).powi(2)).sqrt()) as isize;

        let mut x = -width;
        while x <= width {
            // Calculate canvas position
            let canvas_x = x0 + x;
            let canvas_y = y0 + y;

            // Make sure we're within canvas bounds
            if canvas_x >= 0
                && (canvas_x as usize) < canvas[0].len()
                && canvas_y >= 0
                && (canvas_y as usize) < canvas.len()
            {
                canvas[canvas_y as usize][canvas_x as usize] = EYE_CHAR;
            }
            x += 1;
        }
        y += 1;
    }
}

fn initial_model() -> Model {
    let mut rng = Rng::seeded();
    let open_time = Duration::from_millis(rng.intn(OPEN_TIME_MAX - OPEN_TIME_MIN) + OPEN_TIME_MIN);

    let mut m = Model {
        width: 80,
        height: 24,
        eye_positions: [0, 0],
        eye_y: 0,
        is_blinking: false,
        blink_state: 0,
        last_blink: SystemTime::now(),
        open_time,
        rng,
    };

    m.update_eye_positions();
    m
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(initial_model());
    p.run()?;
    Ok(())
}
