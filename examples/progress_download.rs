//! Cleanroom Rust port of upstream Go example: `examples/progress-download/main.go`
//! (including the `examples/progress-download/tui.go` helper)
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A real-world example of downloading a file while rendering an animated
//! progress bar. Note that this port supports plain `http://` URLs: the
//! upstream example uses Go's `net/http`, which additionally supports `https`
//! via TLS, which is out of scope for this cleanroom port.
//!
//! Usage: `cargo run --example progress_download -- --url http://example.com/file.bin`

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusty_bubbles::progress;
use rusty_bubbletea::commands;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::{Color, Style};

fn help_style() -> Style {
    Style::new().foreground_color(Color::parse("#626262"))
}

const PADDING: usize = 2;
const MAX_WIDTH: usize = 80;

/// ProgressMsg reports the download progress as a ratio between 0 and 1.
#[derive(Debug, Clone, Copy)]
struct ProgressMsg(f64);

/// ProgressErrMsg reports an error that occurred during the download.
#[derive(Debug, Clone)]
struct ProgressErrMsg(String);

/// FinalPause is a short pause after the download completes, mirroring the
/// upstream `finalPause` command.
fn final_pause() -> Cmd {
    commands::tick(Duration::from_millis(750), |_t| None)
}

/// WaitForProgress is a command that waits for the next progress report on
/// the shared channel.
fn wait_for_progress(rx: Arc<Mutex<mpsc::Receiver<Result<f64, String>>>>) -> Cmd {
    Some(Box::new(move || match rx.lock().unwrap().recv() {
        Ok(Ok(p)) => Some(Box::new(ProgressMsg(p))),
        Ok(Err(e)) => Some(Box::new(ProgressErrMsg(e))),
        Err(_) => Some(Box::new(ProgressErrMsg("download failed".to_string()))),
    }))
}

/// ProgressWriter tracks the number of bytes downloaded and reports progress,
/// mirroring the upstream `progressWriter`.
struct ProgressWriter {
    total: usize,
    downloaded: usize,
    on_progress: Option<Box<dyn Fn(f64) + Send>>,
    on_error: Option<Box<dyn Fn(String) + Send>>,
}

impl ProgressWriter {
    fn new(total: usize, tx: mpsc::Sender<Result<f64, String>>) -> Self {
        ProgressWriter {
            total,
            downloaded: 0,
            on_progress: Some(Box::new({
                let tx = tx.clone();
                move |ratio: f64| {
                    let _ = tx.send(Ok(ratio));
                }
            })),
            on_error: Some(Box::new(move |err: String| {
                let _ = tx.send(Err(err));
            })),
        }
    }

    /// Start copies the response body to the file, reporting progress as the
    /// download proceeds. This runs on its own thread, like the upstream
    /// `go pw.Start()`.
    fn start(&mut self, file: &mut std::fs::File, reader: &mut dyn Read) {
        let mut buf = [0u8; 8192];
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    if let Some(cb) = &self.on_error {
                        cb(e.to_string());
                    }
                    return;
                }
            };
            if let Err(e) = file.write_all(&buf[..n]) {
                if let Some(cb) = &self.on_error {
                    cb(e.to_string());
                }
                return;
            }
            self.downloaded += n;
            if self.total > 0 {
                if let Some(cb) = &self.on_progress {
                    cb(self.downloaded as f64 / self.total as f64);
                }
            }
        }
    }
}

/// Response is the result of a successful HTTP GET.
struct Response {
    content_length: usize,
    body: Box<dyn Read + Send>,
}

/// GetResponse performs a minimal HTTP/1.1 GET and returns the response's
/// content length and body reader, mirroring the upstream `getResponse`.
fn get_response(url: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let host_port = url
        .strip_prefix("http://")
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url);
    let path = url
        .strip_prefix("http://")
        .and_then(|u| u.find('/').map(|i| &u[i..]))
        .unwrap_or("/");

    let mut stream = TcpStream::connect(host_port)?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: rusty-bubbletea-progress-download\r\n\r\n",
        path, host_port
    );
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    // Read the headers, one byte at a time, until the blank line.
    let mut headers = String::new();
    let mut buf = [0u8; 1];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Err("could not read response headers".into());
        }
        headers.push(buf[0] as char);
        if headers.ends_with("\r\n\r\n") {
            break;
        }
    }

    let mut lines = headers.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status = status_line
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if status != 200 {
        return Err(format!("receiving status of {} for url: {}", status, url).into());
    }

    let mut content_length: usize = 0;
    for line in lines {
        let lower = line.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }

    Ok(Response {
        content_length,
        body: Box::new(stream),
    })
}

/// Model is the Bubble Tea model for the download progress view, mirroring the
/// upstream `tui.go` model.
struct Model {
    // The shared download state, mirroring the upstream `pw *progressWriter`
    // field. The download thread drives the writer via a clone of this Arc.
    #[allow(dead_code)]
    pw: Arc<Mutex<ProgressWriter>>,
    rx: Arc<Mutex<mpsc::Receiver<Result<f64, String>>>>,
    progress: progress::Model,
    err: Option<String>,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        wait_for_progress(self.rx.clone())
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if msg.as_any().downcast_ref::<KeyPressMsg>().is_some() {
            return quit();
        }

        if let Some(w) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            let width = w.width.saturating_sub(PADDING * 2 + 4);
            self.progress.set_width(width);
            if self.progress.width() > MAX_WIDTH {
                self.progress.set_width(MAX_WIDTH);
            }
            return None;
        }

        if let Some(e) = msg.as_any().downcast_ref::<ProgressErrMsg>() {
            self.err = Some(e.0.clone());
            return quit();
        }

        if let Some(pm) = msg.as_any().downcast_ref::<ProgressMsg>() {
            let mut cmds: Vec<Cmd> = Vec::new();

            if pm.0 >= 1.0 {
                cmds.push(commands::sequence(vec![final_pause(), quit()]));
            }

            cmds.push(self.progress.set_percent(pm.0));

            // Keep waiting for the next progress report unless we're done.
            if pm.0 < 1.0 {
                cmds.push(wait_for_progress(self.rx.clone()));
            }

            return batch(cmds);
        }

        // FrameMsg is sent when the progress bar wants to animate itself.
        if msg.as_any().downcast_ref::<progress::FrameMsg>().is_some() {
            return self.progress.update(msg);
        }

        None
    }

    fn view(&self) -> View {
        if let Some(err) = &self.err {
            return View::new(&format!("Error downloading: {}\n", err));
        }

        let pad = " ".repeat(PADDING);
        View::new(&format!(
            "\n{}{}\n\n{}{}",
            pad,
            self.progress.view(),
            pad,
            help_style().render("Press any key to quit")
        ))
    }
}

fn usage() {
    println!("Usage: progress_download --url <url>");
    println!("  -url string");
    println!("        url for the file to download");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let mut url = String::new();
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--url" || args[i] == "-url") && i + 1 < args.len() {
            url = args[i + 1].clone();
            i += 1;
        }
        i += 1;
    }

    if url.is_empty() {
        usage();
        std::process::exit(1);
    }

    let resp = match get_response(&url) {
        Ok(r) => r,
        Err(e) => {
            println!("could not get response: {}", e);
            std::process::exit(1);
        }
    };

    // Don't add TUI if the header doesn't include content size: it's
    // impossible to see progress without a total.
    if resp.content_length == 0 {
        println!("can't parse content length, aborting download");
        std::process::exit(1);
    }

    let filename = url.rsplit('/').next().unwrap_or(&url).to_string();
    let mut file = match std::fs::File::create(&filename) {
        Ok(f) => f,
        Err(e) => {
            println!("could not create file: {}", e);
            std::process::exit(1);
        }
    };

    let (tx, rx) = mpsc::channel::<Result<f64, String>>();
    let pw = Arc::new(Mutex::new(ProgressWriter::new(resp.content_length, tx)));

    let m = Model {
        pw: pw.clone(),
        rx: Arc::new(Mutex::new(rx)),
        progress: progress::new(vec![progress::with_default_blend()]),
        err: None,
    };

    // Start the download on its own thread, mirroring the upstream
    // `go pw.Start()`.
    let pw_thread = pw.clone();
    let mut body = resp.body;
    std::thread::spawn(move || {
        pw_thread.lock().unwrap().start(&mut file, &mut body);
    });

    let p = Program::new(m);
    p.run()?;
    Ok(())
}
