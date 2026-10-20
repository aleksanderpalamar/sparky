use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io, ptr, thread};

use crate::state::Sparky;

pub struct Environment {
    pub wayland_display: OsString,
    pub x_display: Option<u32>,
    pub ipc_socket: Option<PathBuf>,
}

impl Sparky {
    pub fn child_environment(&self) -> Environment {
        Environment {
            wayland_display: self.socket_name.clone(),
            x_display: self.x_display,
            ipc_socket: self.ipc_path(),
        }
    }

    pub fn spawn(&self, command: &str) {
        spawn(command, &self.child_environment());
    }
}

fn unblock_signals() -> io::Result<()> {
    let mut empty = std::mem::MaybeUninit::<libc::sigset_t>::uninit();
    let result = unsafe {
        libc::sigemptyset(empty.as_mut_ptr());
        libc::pthread_sigmask(libc::SIG_SETMASK, empty.as_ptr(), ptr::null_mut())
    };
    match result {
        0 => Ok(()),
        code => Err(io::Error::from_raw_os_error(code)),
    }
}

pub fn spawn(command: &str, environment: &Environment) {
    let mut process = Command::new("sh");
    process
        .arg("-c")
        .arg(command)
        .env("WAYLAND_DISPLAY", &environment.wayland_display)
        .env("XDG_CURRENT_DESKTOP", "sparky")
        .env("XDG_SESSION_DESKTOP", "sparky")
        .env("XDG_SESSION_TYPE", "wayland")
        .stdin(Stdio::null());
    match environment.x_display {
        Some(number) => process.env("DISPLAY", format!(":{number}")),
        None => process.env_remove("DISPLAY"),
    };
    if let Some(path) = &environment.ipc_socket {
        process.env(sparky_ipc::socket::VARIABLE, path);
    }
    unsafe {
        process.pre_exec(unblock_signals);
    }
    match process.spawn() {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
        }
        Err(error) => tracing::warn!("não deu para abrir {command:?}: {error}"),
    }
}
