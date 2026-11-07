use std::path::Path;

use sparky_hardware::procfs;

use crate::domain::status::{Status, Value};
use crate::linux::host::Host;
use crate::linux::{files, format};

const MEMINFO: &str = "/proc/meminfo";
const KIB: u64 = 1024;

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let text = files::read(host, Path::new(MEMINFO))?;
    describe(&text)
        .map(Value::Text)
        .ok_or_else(|| files::malformed(Path::new(MEMINFO)))
}

pub fn describe(meminfo: &str) -> Option<String> {
    let total = procfs::total_memory_kib(meminfo)?;
    let available = procfs::available_memory_kib(meminfo)?;
    let used = total.saturating_sub(available);
    format::usage(used.saturating_mul(KIB), total.saturating_mul(KIB))
}
