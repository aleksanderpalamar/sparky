use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::domain::status::{Reason, Status};
use crate::i18n;
use crate::linux::host::Host;

pub fn read(host: &dyn Host, path: &Path) -> Result<String, Status> {
    host.read(path).map_err(|error| unreadable(path, &error))
}

pub fn list(host: &dyn Host, path: &Path) -> Result<Vec<PathBuf>, Status> {
    host.list(path).map_err(|error| unreadable(path, &error))
}

pub fn first(host: &dyn Host, paths: &[PathBuf]) -> Result<(PathBuf, String), Status> {
    for path in paths {
        match host.read(path) {
            Ok(text) => return Ok((path.clone(), text)),
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(unreadable(path, &error)),
        }
    }
    Err(missing())
}

pub fn unreadable(path: &Path, error: &io::Error) -> Status {
    if error.kind() == ErrorKind::NotFound {
        return missing();
    }
    let path = path.display().to_string();
    let error = error.to_string();
    Status::failed(i18n::pairs(
        "error-read",
        &[("path", path.as_str()), ("error", error.as_str())],
    ))
}

pub fn malformed(path: &Path) -> Status {
    Status::failed(i18n::with(
        "error-format",
        "path",
        &path.display().to_string(),
    ))
}

pub fn missing() -> Status {
    Status::unavailable(Reason::Missing)
}

pub fn home(host: &dyn Host, variable: &str, fallback: &str) -> Option<PathBuf> {
    let explicit = host.var(variable).filter(|value| value.starts_with('/'));
    explicit
        .map(PathBuf::from)
        .or_else(|| host.var("HOME").map(|home| Path::new(&home).join(fallback)))
}
