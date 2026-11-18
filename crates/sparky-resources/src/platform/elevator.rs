use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use sparky_hardware::process::parse_stat;

use crate::elevation::{Refusal, Renice};
use crate::ports::{Elevator, ExecError, Files};

pub const HELPER: &str = "/usr/lib/sparky/sparky-resources-helper";
const PKEXEC: &str = "/usr/bin/pkexec";
const POLICY: &str = "/usr/share/polkit-1/actions/org.sparky.resources.policy";
const TIMEOUT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(20);
const NOT_AUTHORIZED: [i32; 2] = [126, 127];

pub struct Pkexec<F> {
    files: F,
}

impl<F: Files> Pkexec<F> {
    pub fn installed(files: F) -> Option<Self> {
        [HELPER, PKEXEC, POLICY]
            .iter()
            .all(|path| Path::new(path).exists())
            .then_some(Self { files })
    }
}

impl<F: Files> Elevator for Pkexec<F> {
    fn renice(&self, task: u32, value: i8) -> Result<(), ExecError> {
        let stat = self
            .files
            .read(Path::new(&format!("/proc/{task}/stat")))
            .map_err(|_| ExecError::Gone)?;
        let started = parse_stat(&stat).ok_or(ExecError::Gone)?.started;
        let request = Renice {
            task,
            started,
            value,
        };
        let child = Command::new(PKEXEC)
            .arg("--disable-internal-agent")
            .arg(HELPER)
            .args(request.arguments())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| ExecError::Unsupported)?;
        verdict(finished(child, TIMEOUT))
    }
}

pub fn verdict(code: Option<i32>) -> Result<(), ExecError> {
    let Some(code) = code else {
        return Err(ExecError::Failed);
    };
    if code == 0 {
        return Ok(());
    }
    if NOT_AUTHORIZED.contains(&code) {
        return Err(ExecError::Denied);
    }
    match Refusal::from_code(code) {
        Some(Refusal::Gone) => Err(ExecError::Gone),
        Some(Refusal::Failed) | None => Err(ExecError::Failed),
        Some(_) => Err(ExecError::Denied),
    }
}

pub fn finished(mut child: Child, timeout: Duration) -> Option<i32> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => return status.code(),
            Ok(None) => thread::sleep(POLL),
            Err(_) => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}
