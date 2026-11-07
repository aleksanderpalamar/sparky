use std::path::Path;

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

const OSTYPE: &str = "/proc/sys/kernel/ostype";
const OSRELEASE: &str = "/proc/sys/kernel/osrelease";

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let kind = files::read(host, Path::new(OSTYPE))?;
    let release = files::read(host, Path::new(OSRELEASE))?;
    let parts: Vec<&str> = [kind.trim(), release.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();
    match parts.is_empty() {
        true => Err(files::malformed(Path::new(OSRELEASE))),
        false => Ok(Value::Text(parts.join(" "))),
    }
}
