use std::path::Path;

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    host.var("SHELL")
        .as_deref()
        .and_then(name)
        .map(Value::Text)
        .ok_or_else(files::missing)
}

pub fn name(shell: &str) -> Option<String> {
    Path::new(shell.trim())
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
}
