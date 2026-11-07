use std::path::Path;

use sparky_hardware::procfs;

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

const CPUINFO: &str = "/proc/cpuinfo";

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let text = files::read(host, Path::new(CPUINFO))?;
    describe(&text).map(Value::Text).ok_or_else(files::missing)
}

pub fn describe(cpuinfo: &str) -> Option<String> {
    let name = procfs::processor(cpuinfo)?;
    match procfs::threads(cpuinfo) {
        0 => Some(name),
        threads => Some(format!("{name} ({threads})")),
    }
}
