use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::{io, ptr, thread};

use crate::session::environment::Environment;
use crate::state::Sparky;

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

pub fn spawn(command: &str, environment: &Environment) {
    spawn_program("sh", &["-c".to_owned(), command.to_owned()], environment);
}

pub fn spawn_program(program: &str, arguments: &[String], environment: &Environment) {
    let mut process = Command::new(program);
    process
        .args(arguments)
        .env_remove("DISPLAY")
        .envs(environment.variables())
        .stdin(Stdio::null());
    unsafe {
        process.pre_exec(unblock_signals);
    }
    match process.spawn() {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
        }
        Err(error) => tracing::warn!("não deu para abrir {program} {arguments:?}: {error}"),
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
