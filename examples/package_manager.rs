//! Cleanroom Rust port of upstream Go example: `examples/package-manager/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A fake package manager: installs a shuffled list of packages with a
//! spinner and a progress bar. Press `q`, `esc` or `ctrl+c` to quit.
//!
//! Cleanroom Rust port of upstream Go source file: `examples/package-manager/packages.go`

use std::time::Duration;

use charming_bubbles::progress;
use charming_bubbles::spinner;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{
    batch, print_f, quit, sequence, tick, Cmd, KeyPressMsg, Msg, Program, View,
};
use charming_lipgloss::size;
use charming_lipgloss::{new_style, Color, Style};

struct Model {
    packages: Vec<String>,
    index: usize,
    width: usize,
    height: usize,
    spinner: spinner::Model,
    progress: progress::Model,
    done: bool,
}

fn current_pkg_name_style() -> Style {
    new_style().foreground_color(Color::parse("211"))
}

fn done_style() -> Style {
    new_style().margin(&[1, 2])
}

fn check_mark() -> String {
    new_style().foreground_color(Color::parse("42")).render("✓")
}

impl Model {
    fn new() -> Self {
        let p = progress::new(vec![
            progress::with_default_blend(),
            progress::with_width(40),
            progress::without_percentage(),
        ]);
        let mut s = spinner::new(vec![]);
        s.style = new_style().foreground_color(Color::parse("63"));
        Model {
            packages: get_packages(),
            index: 0,
            width: 0,
            height: 0,
            spinner: s,
            progress: p,
            done: false,
        }
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        let tick_msg = self.spinner.tick_msg();
        batch(vec![
            download_and_install(&self.packages[self.index]),
            Some(Box::new(move || Some(Box::new(tick_msg)))),
        ])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "ctrl+c" | "esc" | "q" => return quit(),
                _ => {}
            }
        }

        if let Some(pkg) = msg.as_any().downcast_ref::<InstalledPkgMsg>() {
            let pkg = pkg.0.clone();
            if self.index >= self.packages.len() - 1 {
                // Everything's been installed. We're done!
                self.done = true;
                return sequence(vec![
                    print_f(format_args!("{} {}", check_mark(), pkg)), // print the last success message
                    quit(),                                            // exit the program
                ]);
            }

            // Update progress bar.
            self.index += 1;
            let progress_cmd = self
                .progress
                .set_percent(self.index as f64 / self.packages.len() as f64);

            return batch(vec![
                progress_cmd,
                print_f(format_args!("{} {}", check_mark(), pkg)), // print success message above our program
                download_and_install(&self.packages[self.index]),  // download the next package
            ]);
        }

        if msg.as_any().is::<spinner::TickMsg>() {
            return self.spinner.update(msg);
        }

        if msg.as_any().is::<progress::FrameMsg>() {
            return self.progress.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        let n = self.packages.len();
        let w = size::width(&format!("{}", n));

        if self.done {
            return View::new(&done_style().render(&format!("Done! Installed {} packages.\n", n)));
        }

        let pkg_count = format!(" {:>w$}/{:>w$}", self.index, n, w = w);

        let spin = self.spinner.view() + " ";
        let prog = self.progress.view();
        let cells_avail = self
            .width
            .saturating_sub(size::width(&(spin.clone() + &prog + &pkg_count)));

        let pkg_name = current_pkg_name_style().render(&self.packages[self.index]);
        let info = new_style()
            .max_width(cells_avail)
            .render(&format!("Installing {}", pkg_name));

        let cells_remaining = self
            .width
            .saturating_sub(size::width(&(spin.clone() + &info + &prog + &pkg_count)));
        let gap = " ".repeat(cells_remaining);

        View::new(&(spin + &info + &gap + &prog + &pkg_count))
    }
}

/// InstalledPkgMsg indicates that a package has been installed.
#[derive(Debug)]
struct InstalledPkgMsg(String);

/// DownloadAndInstall simulates downloading and installing a package. In a
/// real program this is where you'd do I/O stuff.
fn download_and_install(pkg: &str) -> Cmd {
    let pkg = pkg.to_string();
    let d = Duration::from_millis(prng_rand() % 500);
    tick(d, move |_| Some(Box::new(InstalledPkgMsg(pkg.clone()))))
}

/// A tiny xorshift64 PRNG mirroring the upstream `rand.Intn` calls.
fn prng_rand() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15);
    let mut x = STATE.load(Ordering::Relaxed);
    if x == 0 {
        x = 0x9E3779B97F4A7C15;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    STATE.store(x, Ordering::Relaxed);
    x
}

// ---------------------------------------------------------------------------
// packages.go
// ---------------------------------------------------------------------------

const PACKAGES: &[&str] = &[
    "vegeutils",
    "libgardening",
    "currykit",
    "spicerack",
    "fullenglish",
    "eggy",
    "bad-kitty",
    "chai",
    "hojicha",
    "libtacos",
    "babys-monads",
    "libpurring",
    "currywurst-devel",
    "xmodmeow",
    "licorice-utils",
    "cashew-apple",
    "rock-lobster",
    "standmixer",
    "coffee-CUPS",
    "libesszet",
    "zeichenorientierte-benutzerschnittstellen",
    "schnurrkit",
    "old-socks-devel",
    "jalapeño",
    "molasses-utils",
    "xkohlrabi",
    "party-gherkin",
    "snow-peas",
    "libyuzu",
];

/// GetPackages returns a shuffled copy of the package list with a version
/// suffix appended to each package, mirroring the upstream `getPackages`.
fn get_packages() -> Vec<String> {
    let mut pkgs: Vec<String> = PACKAGES.iter().map(|s| s.to_string()).collect();

    for i in (1..pkgs.len()).rev() {
        let j = (prng_rand() % (i as u64 + 1)) as usize;
        pkgs.swap(i, j);
    }

    pkgs.iter()
        .map(|p| {
            format!(
                "{}-{}.{}.{}",
                p,
                prng_rand() % 10,
                prng_rand() % 10,
                prng_rand() % 10
            )
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
