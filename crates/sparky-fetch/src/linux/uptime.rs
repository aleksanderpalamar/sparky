use std::path::Path;

use crate::domain::status::{Status, Value};
use crate::linux::host::Host;
use crate::linux::{files, format};

const UPTIME: &str = "/proc/uptime";

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let text = files::read(host, Path::new(UPTIME))?;
    let seconds = seconds(&text).ok_or_else(|| files::malformed(Path::new(UPTIME)))?;
    Ok(Value::Text(format::duration(seconds)))
}

pub fn seconds(uptime: &str) -> Option<u64> {
    let first = uptime.split_whitespace().next()?;
    let whole = first.split_once('.').map_or(first, |(whole, _)| whole);
    whole.parse().ok()
}
