use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::thread;

pub fn spawn(command: &str, wayland_display: &OsStr, x_display: Option<u32>) {
    let mut process = Command::new("sh");
    process
        .arg("-c")
        .arg(command)
        .env("WAYLAND_DISPLAY", wayland_display)
        .env("XDG_CURRENT_DESKTOP", "sparky")
        .env("XDG_SESSION_DESKTOP", "sparky")
        .env("XDG_SESSION_TYPE", "wayland")
        .stdin(Stdio::null());
    match x_display {
        Some(number) => process.env("DISPLAY", format!(":{number}")),
        None => process.env_remove("DISPLAY"),
    };
    let child = process.spawn();
    match child {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
        }
        Err(error) => tracing::warn!("não deu para abrir {command:?}: {error}"),
    }
}
