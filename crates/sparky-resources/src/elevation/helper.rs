use std::io;
use std::path::Path;

use sparky_hardware::process::parse_stat;

use super::admission::{Facts, admit, owner};
use super::request::{Refusal, Renice};
use crate::ports::{Files, Priority};

pub fn serve(
    arguments: &[String],
    caller: Option<u32>,
    files: &impl Files,
    priority: &impl Priority,
) -> Result<(), Refusal> {
    let request = Renice::parse(arguments)?;
    admit(&request, &facts(request.task, caller, files))?;
    priority
        .set_nice(request.task, request.value)
        .map_err(refusal)?;
    match priority.nice(request.task) {
        Ok(value) if value == request.value => Ok(()),
        Ok(_) => Err(Refusal::Failed),
        Err(error) => Err(refusal(error)),
    }
}

fn facts(task: u32, caller: Option<u32>, files: &impl Files) -> Facts {
    let read = |name: &str| files.read(Path::new(&format!("/proc/{task}/{name}"))).ok();
    Facts {
        caller,
        owner: read("status").as_deref().and_then(owner),
        started: read("stat")
            .as_deref()
            .and_then(parse_stat)
            .map(|stat| stat.started),
    }
}

fn refusal(error: io::Error) -> Refusal {
    match error.raw_os_error() {
        Some(libc::ESRCH) => Refusal::Gone,
        _ => Refusal::Failed,
    }
}
