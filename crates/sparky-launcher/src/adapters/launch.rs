use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use sparky_ipc::{Client, Reply, Request, SessionAction};

use crate::domain::app::{App, Runs, command_line};

const FALLBACK_TERMINAL: &str = "xterm";

pub fn launch(app: &App) -> io::Result<()> {
    if let Runs::Session(action) = app.runs {
        return ask(action);
    }
    let terminal = env::var("TERMINAL")
        .ok()
        .filter(|terminal| !terminal.trim().is_empty())
        .unwrap_or_else(|| FALLBACK_TERMINAL.to_owned());
    match command_line(app, &terminal) {
        Some(line) => start(&line),
        None => Ok(()),
    }
}

fn start(line: &str) -> io::Result<()> {
    Command::new("sh")
        .arg("-c")
        .arg(line)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map(|_| ())
}

fn ask(action: SessionAction) -> io::Result<()> {
    let mut client = Client::from_env().map_err(io::Error::other)?;
    match client.request(&Request::Session { action }).map_err(io::Error::other)? {
        Reply::Error { message } => Err(io::Error::other(message)),
        _ => Ok(()),
    }
}
