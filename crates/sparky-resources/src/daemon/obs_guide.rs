use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::SigSet;

use crate::context::{Sensed, Unavailable};

const GRACE: Duration = Duration::from_secs(30);
const PROGRAM: &str = "sparky-obs-guide";

pub trait Launch {
    fn launch(&mut self);
}

pub struct Guide<L> {
    launcher: L,
    seen: Option<(u32, Instant)>,
    handled: Option<u32>,
}

impl<L: Launch> Guide<L> {
    pub fn new(launcher: L) -> Self {
        Self {
            launcher,
            seen: None,
            handled: None,
        }
    }

    pub fn launcher(&self) -> &L {
        &self.launcher
    }

    pub fn consider(
        &mut self,
        capture: Option<(u32, Sensed)>,
        now: Instant,
        allowed: impl FnOnce() -> bool,
    ) {
        let Some((pid, sensed)) = capture else {
            return;
        };
        let since = match self.seen {
            Some((seen, at)) if seen == pid => at,
            _ => {
                self.seen = Some((pid, now));
                now
            }
        };
        if self.handled == Some(pid) || !due(sensed, now.duration_since(since)) {
            return;
        }
        self.handled = Some(pid);
        if allowed() {
            self.launcher.launch();
        }
    }
}

fn due(sensed: Sensed, open: Duration) -> bool {
    match sensed {
        Sensed::Unavailable(Unavailable::Disabled) => true,
        Sensed::Unavailable(Unavailable::Unreachable) => open >= GRACE,
        _ => false,
    }
}

pub struct Spawn;

impl Launch for Spawn {
    fn launch(&mut self) {
        match spawn(&mut Command::new(PROGRAM)) {
            Ok(mut child) => {
                thread::spawn(move || child.wait());
            }
            Err(error) => eprintln!("sparky-resources: aviso do OBS não abriu: {error}"),
        }
    }
}

pub fn spawn(command: &mut Command) -> io::Result<Child> {
    unsafe {
        command.pre_exec(unblock_signals);
    }
    command.spawn()
}

fn unblock_signals() -> io::Result<()> {
    SigSet::empty()
        .thread_set_mask()
        .map_err(|error| io::Error::from_raw_os_error(error as i32))
}

pub fn allowed() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = sparky_config::path(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), &home);
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| sparky_config::parse(&text).ok())
        .is_none_or(|config| config.resources.obs_guide)
}
