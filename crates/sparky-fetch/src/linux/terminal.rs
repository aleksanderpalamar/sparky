use std::path::PathBuf;

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

const SKIPPED: [&str; 6] = ["zsh", "bash", "fish", "sh", "dash", "sparky-fetch"];
const DEPTH: usize = 32;
const INIT: u32 = 1;

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    if let Some(program) = host
        .var("TERM_PROGRAM")
        .filter(|name| !name.trim().is_empty())
    {
        return Ok(Value::Text(program.trim().to_owned()));
    }
    let mut process = parent_of(host, "self")?;
    for _ in 0..DEPTH {
        let name = command(host, process)?;
        if !SKIPPED.contains(&name.as_str()) {
            return Ok(Value::Text(name));
        }
        process = parent_of(host, &process.to_string())?;
    }
    Err(files::missing())
}

pub fn parent(stat: &str) -> Option<u32> {
    let (_, rest) = stat.rsplit_once(')')?;
    rest.split_whitespace().nth(1)?.parse().ok()
}

fn parent_of(host: &dyn Host, process: &str) -> Result<u32, Status> {
    let path = PathBuf::from(format!("/proc/{process}/stat"));
    let stat = files::read(host, &path)?;
    match parent(&stat) {
        Some(pid) if pid > INIT => Ok(pid),
        Some(_) => Err(files::missing()),
        None => Err(files::malformed(&path)),
    }
}

fn command(host: &dyn Host, process: u32) -> Result<String, Status> {
    let path = PathBuf::from(format!("/proc/{process}/comm"));
    let name = files::read(host, &path)?;
    let name = name.trim().trim_start_matches('-');
    match name.is_empty() {
        true => Err(files::malformed(&path)),
        false => Ok(name.to_owned()),
    }
}
