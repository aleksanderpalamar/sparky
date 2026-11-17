use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use crate::domain::mode::Mode;

const REQUEST: &[u8] = b"close\n";

pub enum Claim {
    Owner(Instance),
    Signalled,
}

pub struct Instance {
    listener: UnixListener,
}

pub fn socket(runtime_dir: &Path, display: &str, mode: Mode) -> PathBuf {
    let kind = match mode {
        Mode::Apps => "apps",
        Mode::System => "system",
    };
    runtime_dir.join(format!("sparky-launcher-{kind}-{display}.sock"))
}

pub fn socket_from_env(mode: Mode) -> Option<PathBuf> {
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
    let display = std::env::var("WAYLAND_DISPLAY").ok()?;
    Some(socket(Path::new(&runtime_dir), &display, mode))
}

pub fn claim(path: &Path) -> io::Result<Claim> {
    if let Ok(mut running) = UnixStream::connect(path) {
        running.write_all(REQUEST)?;
        return Ok(Claim::Signalled);
    }
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    Ok(Claim::Owner(Instance {
        listener: UnixListener::bind(path)?,
    }))
}

impl Instance {
    pub fn wait(&self) -> io::Result<()> {
        loop {
            let (mut stream, _) = self.listener.accept()?;
            let mut request = Vec::new();
            if stream.read_to_end(&mut request).is_ok() && request == REQUEST {
                return Ok(());
            }
        }
    }
}
