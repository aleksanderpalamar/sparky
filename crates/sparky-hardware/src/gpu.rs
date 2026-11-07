use std::path::{Path, PathBuf};

use crate::pci;

const DISPLAY_CLASS: u32 = 0x03;

pub fn names(
    devices: &[PathBuf],
    database: &str,
    read: impl Fn(&Path) -> Option<String>,
) -> Vec<String> {
    let mut paths = devices.to_vec();
    paths.sort();
    paths
        .iter()
        .filter_map(|path| name(&Attributes { path, read: &read }, database))
        .collect()
}

struct Attributes<'a, R> {
    path: &'a Path,
    read: &'a R,
}

impl<R: Fn(&Path) -> Option<String>> Attributes<'_, R> {
    fn text(&self, name: &str) -> Option<String> {
        (self.read)(&self.path.join(name))
            .map(|text| text.trim().to_owned())
            .filter(|text| !text.is_empty())
    }

    fn hex(&self, name: &str) -> Option<u32> {
        let text = self.text(name)?;
        u32::from_str_radix(text.strip_prefix("0x").unwrap_or(&text), 16).ok()
    }
}

fn name<R: Fn(&Path) -> Option<String>>(
    device: &Attributes<'_, R>,
    database: &str,
) -> Option<String> {
    if device.hex("class")? >> 16 != DISPLAY_CLASS {
        return None;
    }
    if let Some(name) = device.text("product_name") {
        return Some(name);
    }
    let vendor = device.hex("vendor")?.try_into().ok()?;
    let model = device.hex("device")?.try_into().ok()?;
    Some(pci::name(database, vendor, model))
}

#[cfg(test)]
#[path = "gpu_tests.rs"]
mod tests;
