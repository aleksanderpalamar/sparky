use smithay::backend::allocator::{Format, Fourcc, Modifier};
use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use smithay::utils::Transform;
use sparky_compositor::state::Sparky;
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub type Error = Box<dyn std::error::Error>;

pub struct Server {
    child: Child,
    directory: tempfile::TempDir,
}

impl Server {
    pub fn signal(&self, name: &str) -> std::io::Result<()> {
        fs::write(self.directory.path().join(name), "")
    }
    pub fn connect(scenario: &str) -> Result<(Self, wayland_client::Connection), Error> {
        let directory = tempfile::tempdir()?;
        let child = Command::new(std::env::current_exe()?)
            .args(["--exact", "fixture::private_capture_server", "--nocapture"])
            .env("SPARKY_CAPTURE_FIXTURE", directory.path())
            .env("SPARKY_CAPTURE_SCENARIO", scenario)
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
            if let Ok(name) = fs::read_to_string(server.directory.path().join("ready")) {
                let socket =
                    std::os::unix::net::UnixStream::connect(server.directory.path().join(name))?;
                return Ok((server, wayland_client::Connection::from_socket(socket)?));
            }
            if let Some(status) = server.child.try_wait()? {
                return Err(format!("fixture encerrou: {status}").into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err("fixture de captura não iniciou".into())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn private_capture_server() -> Result<(), Error> {
    let Some(directory) = std::env::var_os("SPARKY_CAPTURE_FIXTURE") else {
        return Ok(());
    };
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(30));
        std::process::exit(2);
    });
    let mut event_loop = EventLoop::try_new()?;
    let display = Display::new()?;
    let mut state = Sparky::new(&mut event_loop, display)?;
    let output = Output::new(
        "capture-fixture".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "fixture".into(),
            model: "fixture".into(),
        },
    );
    output.create_global::<Sparky>(&state.display_handle);
    let mode = Mode {
        size: (800, 600).into(),
        refresh: 60_000,
    };
    output.change_current_state(
        Some(mode),
        Some(Transform::Normal),
        None,
        Some((0, 0).into()),
    );
    state.space.map_output(&output, (0, 0));
    state.image_capture.formats.set(
        Some(0x1234),
        [
            Format {
                code: Fourcc::Argb8888,
                modifier: Modifier::Linear,
            },
            Format {
                code: Fourcc::Argb8888,
                modifier: Modifier::Invalid,
            },
            Format {
                code: Fourcc::Abgr8888,
                modifier: Modifier::Linear,
            },
        ],
    );
    if std::env::var("SPARKY_CAPTURE_SCENARIO").as_deref() == Ok("locked") {
        state.lock.engage();
    }
    if std::env::var("SPARKY_CAPTURE_SCENARIO").as_deref() == Ok("no-formats") {
        state.image_capture.formats.entries.clear();
    }
    state
        .desktop
        .add_monitor("capture-fixture", sparky_tiling::Rect::new(0, 0, 800, 600));
    let directory = PathBuf::from(directory);
    fs::write(
        directory.join("ready"),
        state.socket_name.as_encoded_bytes(),
    )?;
    loop {
        if directory.join("resize").exists() {
            fs::remove_file(directory.join("resize"))?;
            output.change_current_state(
                Some(Mode {
                    size: (640, 480).into(),
                    refresh: 60_000,
                }),
                None,
                None,
                None,
            );
        }
        if directory.join("remove").exists() {
            fs::remove_file(directory.join("remove"))?;
            state.space.unmap_output(&output);
        }
        event_loop.dispatch(Duration::from_millis(5), &mut state)?;
        super::submission::dispatch(&mut state, &directory)?;
        state.after_dispatch();
    }
}
