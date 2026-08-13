//! Cleanroom Rust port of upstream Go example: `examples/http/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple program that makes a GET request and prints the response status.
//!
//! Deviations from upstream:
//! - `net/http` (which would require a TLS stack for the upstream's
//!   `https://charm.sh/`) is replaced with a minimal HTTP/1.1 GET over
//!   `std::net::TcpStream`, so the target URL is `http://example.com` on
//!   port 80.
//! - `http.StatusText` is replaced with a small `status_text` mapping for
//!   common status codes.
//! - The 10s timeout of the upstream `http.Client` is applied as a
//!   `TcpStream` read timeout.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};

const URL: &str = "http://example.com/";
const HOST: &str = "example.com";
const PORT: u16 = 80;

struct Model {
    status: i32,
    err: Option<String>,
}

/// A message indicating the HTTP response status code.
#[derive(Debug)]
struct StatusMsg(i32);

/// A message indicating that the request failed.
#[derive(Debug)]
struct ErrMsg(String);

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(Box::new(check_server()))))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" | "esc" => return quit(),
                _ => return None,
            }
        }

        if let Some(s) = msg.as_any().downcast_ref::<StatusMsg>() {
            self.status = s.0;
            return quit();
        }

        if let Some(e) = msg.as_any().downcast_ref::<ErrMsg>() {
            self.err = Some(e.0.clone());
            return None;
        }

        None
    }

    fn view(&self) -> View {
        let mut s = format!("Checking {}...", URL);
        if let Some(err) = &self.err {
            s += &format!("something went wrong: {}", err);
        } else if self.status != 0 {
            s += &format!("{} {}", self.status, status_text(self.status));
        }
        View::new(&format!("{}\n", s))
    }
}

/// Performs a GET request and returns either a [StatusMsg] or an [ErrMsg],
/// mirroring `checkServer`.
fn check_server() -> Box<dyn Msg> {
    let mut stream = match TcpStream::connect((HOST, PORT)) {
        Ok(s) => s,
        Err(err) => return Box::new(ErrMsg(err.to_string())),
    };
    if let Err(err) = stream.set_read_timeout(Some(Duration::from_secs(10))) {
        return Box::new(ErrMsg(err.to_string()));
    }

    let request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        HOST
    );
    if let Err(err) = stream.write_all(request.as_bytes()) {
        return Box::new(ErrMsg(err.to_string()));
    }

    let mut response = String::new();
    if let Err(err) = stream.read_to_string(&mut response) {
        return Box::new(ErrMsg(err.to_string()));
    }

    // The status line is of the form "HTTP/1.1 200 OK".
    let status_line = response.lines().next().unwrap_or_default();
    let mut parts = status_line.split_whitespace();
    let _protocol = parts.next();
    let code = parts.next().and_then(|c| c.parse::<i32>().ok());

    match code {
        Some(code) => Box::new(StatusMsg(code)),
        None => Box::new(ErrMsg(
            "unable to parse the response status line".to_string(),
        )),
    }
}

/// Returns the text associated with the given HTTP status code, mirroring
/// `http.StatusText` (a subset of common codes).
fn status_text(code: i32) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        status: 0,
        err: None,
    });
    p.run()?;
    Ok(())
}
