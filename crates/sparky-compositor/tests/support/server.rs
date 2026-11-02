use std::{
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use super::ready;

pub type Error = Box<dyn std::error::Error>;

pub const DIRECTORY: &str = "SPARKY_FIXTURE";
pub const SCENARIO: &str = "SPARKY_FIXTURE_SCENARIO";

pub struct Server {
    child: Child,
    directory: tempfile::TempDir,
}

impl Server {
    pub fn file(&self, name: &str) -> std::path::PathBuf {
        self.directory.path().join(name)
    }

    pub fn connect(
        fixture: &str,
        scenario: &str,
    ) -> Result<(Self, wayland_client::Connection), Error> {
        let directory = tempfile::tempdir()?;
        let child = Command::new(std::env::current_exe()?)
            .args(["--exact", fixture, "--nocapture"])
            .env(DIRECTORY, directory.path())
            .env(SCENARIO, scenario)
            .env("XDG_RUNTIME_DIR", directory.path())
            .env("XDG_CONFIG_HOME", directory.path())
            .env("HOME", directory.path())
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("DISPLAY")
            .env("DBUS_SESSION_BUS_ADDRESS", "unix:path=/nonexistent")
            .env("DBUS_SYSTEM_BUS_ADDRESS", "unix:path=/nonexistent")
            .stdout(Stdio::null())
            .spawn()?;
        let mut server = Self { child, directory };
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(15) {
            if let Some(path) = ready::socket(server.directory.path()) {
                let socket = std::os::unix::net::UnixStream::connect(path)?;
                return Ok((server, wayland_client::Connection::from_socket(socket)?));
            }
            if let Some(status) = server.child.try_wait()? {
                return Err(format!("fixture encerrou: {status}").into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err("fixture não iniciou".into())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn wait<T>(
    queue: &mut wayland_client::EventQueue<T>,
    state: &mut T,
    ready: impl Fn(&T) -> bool,
    missing: &str,
) -> Result<(), Error> {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        queue.roundtrip(state)?;
        if ready(state) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err(missing.into())
}
