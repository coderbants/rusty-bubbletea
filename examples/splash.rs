//! Cleanroom Rust port of upstream Go example: `examples/splash/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! This example was ported from the awesome Textualize project by
//! @willmcgugan. Check it out here:
//! <https://github.com/Textualize/textual/blob/main/examples/splash.py>
//!
//! Deviations from upstream:
//! - The `image/color` interpolations are done with the RGB bytes of
//!   `rusty_lipgloss::Color`.
//! - `time.Now().UnixNano()*m.rate` (which overflows in Go) is mirrored with
//!   wrapping `i64` arithmetic.

use std::time::{Duration, SystemTime};

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{quit, tick, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{new_style, Color};

// Color gradient
const COLORS: [&str; 12] = [
    "#881177", "#aa3355", "#cc6666", "#ee9944", "#eedd00", "#99dd55", "#44dd88", "#22ccbb",
    "#00bbcc", "#0099cc", "#3366bb", "#663399",
];

/// Messages are events that we respond to in our Update function. This
/// particular one indicates that the timer has ticked.
#[derive(Debug)]
struct TickMsg;

/// A tick command that fires immediately, mirroring `tick()` (which performs
/// no sleep).
fn tick_cmd() -> Cmd {
    tick(Duration::ZERO, |_| Some(Box::new(TickMsg)))
}

struct Model {
    width: usize,
    height: usize,
    rate: i64,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        tick_cmd()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            return quit();
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
        }

        if msg.as_any().is::<TickMsg>() {
            return tick_cmd();
        }

        None
    }

    fn view(&self) -> View {
        let mut v = View {
            alt_screen: true,
            ..Default::default()
        };
        if self.width == 0 {
            v.set_content("Initializing...");
            return v;
        }

        v.set_content(&self.gradient());
        v
    }
}

impl Model {
    fn gradient(&self) -> String {
        // Time-based angle for animation
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);
        let t = nanos.wrapping_mul(self.rate) as f64 / 1_000_000_000.0;
        let angle_radians = -t * std::f64::consts::PI / 180.0;
        let sin_angle = angle_radians.sin();
        let cos_angle = angle_radians.cos();

        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64;

        let mut output = String::new();

        for line_y in 0..self.height {
            let point_y = line_y as f64 * 2.0 - center_y;
            let mut point_x = 0.0 - center_x;

            let x1 = (center_x + (point_x * cos_angle - point_y * sin_angle)) / self.width as f64;
            let x2 = (center_x + (point_x * cos_angle - (point_y + 1.0) * sin_angle))
                / self.width as f64;
            point_x = self.width as f64 - center_x;
            let end_x1 =
                (center_x + (point_x * cos_angle - point_y * sin_angle)) / self.width as f64;
            let delta_x = (end_x1 - x1) / self.width as f64;

            if delta_x.abs() < 0.0001 {
                // Special case for verticals
                let color1 = get_gradient_color(x1);
                let color2 = get_gradient_color(x2);
                let style = new_style()
                    .foreground_color(color1)
                    .background_color(color2);
                output.push_str(&style.render(&"▀".repeat(self.width)));
            } else {
                // Render each column in the row
                for x in 0..self.width {
                    let pos1 = x1 + x as f64 * delta_x;
                    let pos2 = x2 + x as f64 * delta_x;
                    let color1 = get_gradient_color(pos1);
                    let color2 = get_gradient_color(pos2);
                    let style = new_style()
                        .foreground_color(color1)
                        .background_color(color2);
                    output.push_str(&style.render("▀"));
                }
            }
            if line_y < self.height - 1 {
                output.push('\n');
            }
        }

        output
    }
}

/// Returns the interpolated gradient color at the given normalized position,
/// mirroring `getGradientColor`.
fn get_gradient_color(mut position: f64) -> Color {
    // Normalize position to [0,1]
    position = position.clamp(0.0, 1.0);

    // Calculate the color index
    let idx = position * (COLORS.len() - 1) as f64;
    let mut i1 = idx.floor() as i64;
    let mut i2 = idx.ceil() as i64;

    // Ensure indices are within bounds
    i1 = i1.rem_euclid(COLORS.len() as i64);
    i2 = i2.rem_euclid(COLORS.len() as i64);
    if i1 < 0 {
        i1 += COLORS.len() as i64;
    }
    if i2 < 0 {
        i2 += COLORS.len() as i64;
    }

    // Interpolate between colors
    let t = idx - i1 as f64;
    interpolate_colors(
        Color::parse(COLORS[i1 as usize]),
        Color::parse(COLORS[i2 as usize]),
        t,
    )
}

/// Linearly interpolates between two colors, mirroring `interpolateColors`.
fn interpolate_colors(color1: Color, color2: Color, t: f64) -> Color {
    // Parse hex colors
    let (r1, g1, b1, _) = color1.rgba_bytes();
    let (r2, g2, b2, _) = color2.rgba_bytes();

    // Interpolate
    let r = r1 as f64 * (1.0 - t) + r2 as f64 * t;
    let g = g1 as f64 * (1.0 - t) + g2 as f64 * t;
    let b = b1 as f64 * (1.0 - t) + b2 as f64 * t;

    Color::TrueColor {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        width: 0,
        height: 0,
        rate: 90,
    });
    p.run()?;
    Ok(())
}
