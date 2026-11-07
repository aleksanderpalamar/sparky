use std::path::Path;

use crate::domain::status::{Status, Value};
use crate::linux::host::Host;
use crate::linux::{files, format};

const ROOT: &str = "/";

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let root = Path::new(ROOT);
    let usage = host
        .usage(root)
        .map_err(|error| files::unreadable(root, &error))?;
    format::usage(usage.total.saturating_sub(usage.free), usage.total)
        .map(Value::Text)
        .ok_or_else(files::missing)
}
