//! Cleanroom Rust port of upstream Go example: `examples/cellbuffer/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example demonstrating how to draw and animate on a cellular grid.
//! Note that the cellbuffer implementation in this example does not support
//! double-width runes.
//!
//! Deviations from upstream:
//! - `github.com/charmbracelet/harmonica` is inlined: the spring coefficients
//!   of `harmonica.Spring` (Ryan Juckett's damped harmonic motion) are ported
//!   directly, with `harmonica.FPS(fps)` as the `1/60` second delta time.
//! - `tea.MouseMsg` is received as the typed `MouseClickMsg`/`MouseReleaseMsg`/
//!   `MouseWheelMsg`/`MouseMotionMsg` messages; as in the upstream no-op
//!   switch, every mouse message updates the spring target.

use std::time::Duration;

use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::mouse::{MouseClickMsg, MouseMotionMsg, MouseReleaseMsg, MouseWheelMsg};
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::view::MouseMode;
use charming_bubbletea::{quit, tick, Cmd, KeyPressMsg, Msg, Program, View};

const FPS: u64 = 60;
const FREQUENCY: f64 = 7.5;
const DAMPING: f64 = 0.15;
const ASTERISK: &str = "*";

/// Draws an ellipse outline on the cell buffer using the midpoint ellipse
/// algorithm, mirroring `drawEllipse`.
fn draw_ellipse(cb: &mut CellBuffer, xc: f64, yc: f64, rx: f64, ry: f64) {
    let mut dx: f64;
    let mut dy: f64;
    let mut d1: f64;
    let mut d2: f64;
    let mut x: f64 = 0.0;
    let mut y: f64 = ry;

    d1 = ry * ry - rx * rx * ry + 0.25 * rx * rx;
    dx = 2.0 * ry * ry * x;
    dy = 2.0 * rx * rx * y;

    while dx < dy {
        cb.set((x + xc) as isize, (y + yc) as isize);
        cb.set((-x + xc) as isize, (y + yc) as isize);
        cb.set((x + xc) as isize, (-y + yc) as isize);
        cb.set((-x + xc) as isize, (-y + yc) as isize);
        if d1 < 0.0 {
            x += 1.0;
            dx += 2.0 * ry * ry;
            d1 = d1 + dx + (ry * ry);
        } else {
            x += 1.0;
            y -= 1.0;
            dx += 2.0 * ry * ry;
            dy -= 2.0 * rx * rx;
            d1 = d1 + dx - dy + (ry * ry);
        }
    }

    d2 = ((ry * ry) * ((x + 0.5) * (x + 0.5))) + ((rx * rx) * ((y - 1.0) * (y - 1.0)))
        - (rx * rx * ry * ry);

    while y >= 0.0 {
        cb.set((x + xc) as isize, (y + yc) as isize);
        cb.set((-x + xc) as isize, (y + yc) as isize);
        cb.set((x + xc) as isize, (-y + yc) as isize);
        cb.set((-x + xc) as isize, (-y + yc) as isize);
        if d2 > 0.0 {
            y -= 1.0;
            dy -= 2.0 * rx * rx;
            d2 = d2 + (rx * rx) - dy;
        } else {
            y -= 1.0;
            x += 1.0;
            dx += 2.0 * ry * ry;
            dy -= 2.0 * rx * rx;
            d2 = d2 + dx - dy + (rx * rx);
        }
    }
}

/// A cellular grid of terminal cells, mirroring `cellbuffer`.
struct CellBuffer {
    cells: Vec<&'static str>,
    stride: usize,
}

impl CellBuffer {
    fn new() -> CellBuffer {
        CellBuffer {
            cells: Vec::new(),
            stride: 0,
        }
    }

    fn init(&mut self, w: usize, h: usize) {
        if w == 0 {
            return;
        }
        self.stride = w;
        self.cells = vec![BG_CELL; w * h];
        self.wipe();
    }

    fn set(&mut self, x: isize, y: isize) {
        if x < 0 || y < 0 || x as usize >= self.width() || y as usize >= self.height() {
            return;
        }
        let i = y as usize * self.stride + x as usize;
        if i > self.cells.len() - 1 {
            return;
        }
        self.cells[i] = ASTERISK;
    }

    fn wipe(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = BG_CELL;
        }
    }

    fn width(&self) -> usize {
        self.stride
    }

    fn height(&self) -> usize {
        let mut h = self.cells.len() / self.stride;
        if !self.cells.len().is_multiple_of(self.stride) {
            h += 1;
        }
        h
    }

    fn ready(&self) -> bool {
        !self.cells.is_empty()
    }

    fn string(&self) -> String {
        let mut b = String::new();
        for (i, cell) in self.cells.iter().enumerate() {
            if i > 0 && i % self.stride == 0 && i < self.cells.len() - 1 {
                b.push('\n');
            }
            b.push_str(cell);
        }
        b
    }
}

/// The blank cell character.
const BG_CELL: &str = " ";

/// Messages are events that we respond to in our Update function. This
/// particular one indicates that the animation frame has advanced.
#[derive(Debug)]
struct FrameMsg;

/// A tick command at 60fps, mirroring `animate()`.
fn animate() -> Cmd {
    tick(Duration::from_micros(1_000_000 / FPS), |_| {
        Some(Box::new(FrameMsg))
    })
}

/// A cached set of spring motion parameters, mirroring
/// `harmonica.Spring` (Ryan Juckett's damped harmonic motion).
struct Spring {
    pos_pos_coef: f64,
    pos_vel_coef: f64,
    vel_pos_coef: f64,
    vel_vel_coef: f64,
}

/// The machine epsilon, mirroring `epsilon` in harmonica.
const EPSILON: f64 = 2.220446049250313e-16;

impl Spring {
    /// Initializes a new Spring, computing the parameters needed to simulate
    /// a damped spring over a given period of time, mirroring
    /// `harmonica.NewSpring`.
    fn new(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Spring {
        // Keep values in a legal range.
        let angular_frequency = angular_frequency.max(0.0);
        let damping_ratio = damping_ratio.max(0.0);

        // If there is no angular frequency, the spring will not move and we
        // can return identity.
        if angular_frequency < EPSILON {
            return Spring {
                pos_pos_coef: 1.0,
                pos_vel_coef: 0.0,
                vel_pos_coef: 0.0,
                vel_vel_coef: 1.0,
            };
        }

        if damping_ratio > 1.0 + EPSILON {
            // Over-damped.
            let za = -angular_frequency * damping_ratio;
            let zb = angular_frequency * (damping_ratio * damping_ratio - 1.0).sqrt();
            let z1 = za - zb;
            let z2 = za + zb;

            let e1 = (z1 * delta_time).exp();
            let e2 = (z2 * delta_time).exp();

            let inv_two_zb = 1.0 / (2.0 * zb); // = 1 / (z2 - z1)

            let e1_over_two_zb = e1 * inv_two_zb;
            let e2_over_two_zb = e2 * inv_two_zb;

            let z1e1_over_two_zb = z1 * e1_over_two_zb;
            let z2e2_over_two_zb = z2 * e2_over_two_zb;

            Spring {
                pos_pos_coef: e1_over_two_zb * z2 - z2e2_over_two_zb + e2,
                pos_vel_coef: -e1_over_two_zb + e2_over_two_zb,
                vel_pos_coef: (z1e1_over_two_zb - z2e2_over_two_zb + e2) * z2,
                vel_vel_coef: -z1e1_over_two_zb + z2e2_over_two_zb,
            }
        } else if damping_ratio < 1.0 - EPSILON {
            // Under-damped.
            let omega_zeta = angular_frequency * damping_ratio;
            let alpha = angular_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();

            let exp_term = (-omega_zeta * delta_time).exp();
            let cos_term = (alpha * delta_time).cos();
            let sin_term = (alpha * delta_time).sin();

            let inv_alpha = 1.0 / alpha;

            let exp_sin = exp_term * sin_term;
            let exp_cos = exp_term * cos_term;
            let exp_omega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

            Spring {
                pos_pos_coef: exp_cos + exp_omega_zeta_sin_over_alpha,
                pos_vel_coef: exp_sin * inv_alpha,
                vel_pos_coef: -exp_sin * alpha - omega_zeta * exp_omega_zeta_sin_over_alpha,
                vel_vel_coef: exp_cos - exp_omega_zeta_sin_over_alpha,
            }
        } else {
            // Critically damped.
            let exp_term = (-angular_frequency * delta_time).exp();
            let time_exp = delta_time * exp_term;
            let time_exp_freq = time_exp * angular_frequency;

            Spring {
                pos_pos_coef: time_exp_freq + exp_term,
                pos_vel_coef: time_exp,
                vel_pos_coef: -angular_frequency * time_exp_freq,
                vel_vel_coef: -time_exp_freq + exp_term,
            }
        }
    }

    /// Updates position and velocity values against a given target value,
    /// mirroring `Spring.Update`.
    fn update(&self, pos: f64, vel: f64, equilibrium_pos: f64) -> (f64, f64) {
        let old_pos = pos - equilibrium_pos; // update in equilibrium relative space
        let old_vel = vel;

        let new_pos = old_pos * self.pos_pos_coef + old_vel * self.pos_vel_coef + equilibrium_pos;
        let new_vel = old_pos * self.vel_pos_coef + old_vel * self.vel_vel_coef;

        (new_pos, new_vel)
    }
}

struct Model {
    cells: CellBuffer,
    spring: Spring,
    target_x: f64,
    target_y: f64,
    x: f64,
    y: f64,
    x_velocity: f64,
    y_velocity: f64,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        animate()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            return quit();
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            if !self.cells.ready() {
                self.target_x = ws.width as f64 / 2.0;
                self.target_y = ws.height as f64 / 2.0;
            }
            self.cells.init(ws.width, ws.height);
            return None;
        }

        // In the upstream, any tea.MouseMsg updates the spring target after a
        // no-op type switch over the click and motion variants.
        if let Some(m) = msg.as_any().downcast_ref::<MouseClickMsg>() {
            return self.set_mouse_target(m.0.x, m.0.y);
        }
        if let Some(m) = msg.as_any().downcast_ref::<MouseMotionMsg>() {
            return self.set_mouse_target(m.0.x, m.0.y);
        }
        if let Some(m) = msg.as_any().downcast_ref::<MouseReleaseMsg>() {
            return self.set_mouse_target(m.0.x, m.0.y);
        }
        if let Some(m) = msg.as_any().downcast_ref::<MouseWheelMsg>() {
            return self.set_mouse_target(m.0.x, m.0.y);
        }

        if msg.as_any().is::<FrameMsg>() {
            if !self.cells.ready() {
                return None;
            }

            self.cells.wipe();
            let (nx, nvx) = self.spring.update(self.x, self.x_velocity, self.target_x);
            self.x = nx;
            self.x_velocity = nvx;
            let (ny, nvy) = self.spring.update(self.y, self.y_velocity, self.target_y);
            self.y = ny;
            self.y_velocity = nvy;
            draw_ellipse(&mut self.cells, self.x, self.y, 16.0, 8.0);
            return animate();
        }

        None
    }

    fn view(&self) -> View {
        let mut v = View::new(&self.cells.string());
        v.alt_screen = true;
        v.mouse_mode = MouseMode::MouseModeCellMotion;
        v
    }
}

impl Model {
    fn set_mouse_target(&mut self, x: usize, y: usize) -> Cmd {
        if !self.cells.ready() {
            return None;
        }
        self.target_x = x as f64;
        self.target_y = y as f64;
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model {
        cells: CellBuffer::new(),
        spring: Spring::new(1.0 / FPS as f64, FREQUENCY, DAMPING),
        target_x: 0.0,
        target_y: 0.0,
        x: 0.0,
        y: 0.0,
        x_velocity: 0.0,
        y_velocity: 0.0,
    };

    let p = Program::new(m);
    p.run()?;
    Ok(())
}
