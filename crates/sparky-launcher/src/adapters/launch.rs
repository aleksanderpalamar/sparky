use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::domain::app::{App, command_line};

const FALLBACK_TERMINAL: &str = "xterm";

pub fn launch(app: &App) -> io::Result<()> {
    let terminal = env::var("TERMINAL")
        .ok()
        .filter(|terminal| !terminal.trim().is_empty())
        .unwrap_or_else(|| FALLBACK_TERMINAL.to_owned());
    Command::new("sh")
        .arg("-c")
        .arg(command_line(app, &terminal))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map(|_| ())
}
