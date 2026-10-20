use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::thread;

pub fn spawn(command: &str, wayland_display: &OsStr) {
    let child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("WAYLAND_DISPLAY", wayland_display)
        .env("XDG_CURRENT_DESKTOP", "sparky")
        .env("XDG_SESSION_DESKTOP", "sparky")
        .env("XDG_SESSION_TYPE", "wayland")
        .env_remove("DISPLAY")
        .stdin(Stdio::null())
        .spawn();
    match child {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
        }
        Err(error) => tracing::warn!("não deu para abrir {command:?}: {error}"),
    }
}
