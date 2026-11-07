use std::path::Path;

use sparky_hardware::{gpu, pci};

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

const DEVICES: &str = "/sys/bus/pci/devices";

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let devices = files::list(host, Path::new(DEVICES))?;
    let read = |path: &Path| host.read(path).ok();
    let names = gpu::names(&devices, &pci::database(read), read);
    match names.is_empty() {
        true => Err(files::missing()),
        false => Ok(Value::Lines(names)),
    }
}
