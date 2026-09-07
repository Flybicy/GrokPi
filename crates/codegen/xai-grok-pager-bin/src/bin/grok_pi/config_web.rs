//! `grok-pi config`: loopback WebUI for Pi provider/model configuration.
//!
//! Mirrors the CC-lite `cclite config` flow: a single-page editor served on
//! 127.0.0.1 that edits Pi's `models.json` through the same snapshot /
//! backup / atomic-write transaction used by the in-TUI `/pi-models` editor,
//! so a running Pi picks up changes on its next model reload.
//!
//! The page handles plaintext API keys. There is no login form on purpose:
//! the listener is loopback-exclusive and the process dies with the
//! `grok-pi config` command. It still enforces a Host / Origin allowlist,
//! because without it any website the user visits could use DNS rebinding to
//! read `/api/config`.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::{Context, Result, bail};
use xai_grok_pager::pi_model_config::{PiModelConfigSnapshot, PiModelsFile};

/// Default port, and the window scanned when it is already taken.
pub const DEFAULT_PORT: u16 = 31415;
const PORT_SCAN_ATTEMPTS: u16 = 20;
const MAX_BODY_BYTES: u64 = 1_000_000;

const PAGE: &str = include_str!("config_web.html");

pub fn run(port: Option<u16>, lan: bool, no_open: bool) -> Result<()> {
    let bind_host = if lan { "0.0.0.0" } else { "127.0.0.1" };
    let first = port.unwrap_or(DEFAULT_PORT);
    let mut listener = None;
    for offset in 0..PORT_SCAN_ATTEMPTS {
        let candidate = first.saturating_add(offset);
        match TcpListener::bind((bind_host, candidate)) {
            Ok(l) => {
                listener = Some((l, candidate));
                break;
            }
            Err(_) => continue,
        }
    }
    let (listener, bound_port) = listener.with_context(|| {
        format!("no free port in range {first}..{}", first + PORT_SCAN_ATTEMPTS)
    })?;

    let url = format!("http://127.0.0.1:{bound_port}/");
    eprintln!("grok-pi config WebUI: {url}");
    eprintln!("Editing providers/models writes Pi's models.json (with backup). Ctrl+C to stop.");
    if lan {
        eprintln!("--lan: listening on all interfaces; models.json API keys are reachable from the LAN.");
    } else if !no_open {
        open_browser(&url);
    }

    for stream in listener.incoming() {
        match stream {
            Ok(conn) => {
                std::thread::spawn(move || {
                    if let Err(err) = handle_connection(conn, lan) {
                        eprintln!("grok-pi config: request error: {err:#}");
                    }
                });
            }
            Err(err) => eprintln!("grok-pi config: accept error: {err}"),
        }
    }
    Ok(())
}

#[cfg(windows)]
fn open_browser(url: &str) {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) {
    let _ = std::process::Command::new("open").arg(url).spawn();
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_browser(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn handle_connection(conn: TcpStream, lan: bool) -> Result<()> {
    let mut reader = BufReader::new(conn.try_clone()?);
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(());
    }
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or_default().to_string();

    let mut host: Option<String> = None;
    let mut origin: Option<String> = None;
    let mut content_length: u64 = 0;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            match name.trim().to_ascii_lowercase().as_str() {
                "host" => host = Some(value.trim().to_string()),
                "origin" => origin = Some(value.trim().to_string()),
                "content-length" => content_length = value.trim().parse().unwrap_or(0),
                _ => {}
            }
        }
    }

    if !lan {
        if !is_allowed_host(host.as_deref()) || !is_allowed_origin(origin.as_deref()) {
            return respond(&mut conn, 403, "application/json", b"{\"error\":\"forbidden\"}");
        }
    }

    let body = if content_length > 0 {
        if content_length > MAX_BODY_BYTES {
            bail!("request body too large");
        }
        let mut buf = vec![0u8; content_length as usize];
        reader.read_exact(&mut buf)?;
        buf
    } else {
        Vec::new()
    };

    match (method.as_str(), target.as_str()) {
        ("GET", "/" | "/index.html") => respond(
            &mut conn,
            200,
            "text/html; charset=utf-8",
            PAGE.as_bytes(),
        ),
        ("GET", "/api/config") => match load_state() {
            Ok(payload) => respond_json(&mut conn, 200, &payload),
            Err(err) => respond_error(&mut conn, 500, &format!("{err:#}")),
        },
        ("PUT", "/api/config") => match save_state(&body) {
            Ok(payload) => respond_json(&mut conn, 200, &payload),
            Err(err) => respond_error(&mut conn, 400, &format!("{err:#}")),
        },
        _ => respond_error(&mut conn, 404, "not found"),
    }
}

fn load_state() -> Result<serde_json::Value> {
    let snapshot = PiModelConfigSnapshot::load()?;
    Ok(serde_json::json!({
        "path": snapshot.path.display().to_string(),
        "document": snapshot.document,
    }))
}

fn save_state(body: &[u8]) -> Result<serde_json::Value> {
    let document: PiModelsFile =
        serde_json::from_slice(body).context("request body is not a valid models.json document")?;
    let mut snapshot = PiModelConfigSnapshot::load()?;
    snapshot.document = document;
    let report = snapshot.save()?;
    Ok(serde_json::json!({
        "path": report.path.display().to_string(),
        "backup": report.backup.map(|p| p.display().to_string()),
    }))
}

/// Reject requests whose Host header is not a loopback literal. An attacker
/// page can point a hostname it controls at 127.0.0.1, but it cannot forge
/// this header from the browser.
fn is_allowed_host(host: Option<&str>) -> bool {
    let Some(host) = host else { return false };
    let host = if host.starts_with('[') {
        // IPv6 literal, e.g. "[::1]:31415".
        format!("{}]", host.split(']').next().unwrap_or_default())
    } else {
        host.split(':').next().unwrap_or_default().to_string()
    };
    host == "127.0.0.1" || host == "[::1]" || host == "localhost"
}

/// Same-origin fetches from our own page omit Origin in some runtimes; only
/// reject when one is present and does not point back at loopback.
fn is_allowed_origin(origin: Option<&str>) -> bool {
    let Some(origin) = origin else { return true };
    // http://host[:port]/... — strip scheme and path, reuse the Host rules.
    let Some((_, after_scheme)) = origin.split_once("://") else {
        return false;
    };
    let authority = after_scheme.split('/').next().unwrap_or_default();
    is_allowed_host(Some(authority))
}

fn respond(conn: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) -> Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: {content_type}\r\ncache-control: no-store\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    );
    conn.write_all(head.as_bytes())?;
    conn.write_all(body)?;
    Ok(())
}

fn respond_json(conn: &mut TcpStream, status: u16, payload: &serde_json::Value) -> Result<()> {
    respond(conn, status, "application/json; charset=utf-8", payload.to_string().as_bytes())
}

fn respond_error(conn: &mut TcpStream, status: u16, message: &str) -> Result<()> {
    respond_json(conn, status, &serde_json::json!({ "error": message }))
}
