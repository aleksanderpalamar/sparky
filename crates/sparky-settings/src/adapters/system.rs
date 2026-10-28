use std::fs;

use crate::adapters::hardware;
use crate::domain::about::{About, pretty_name};
use crate::domain::keymaps::Keymaps;

const RULES: &str = "/usr/share/X11/xkb/rules/evdev.lst";

pub fn about() -> About {
    let read = |path: &str| {
        fs::read_to_string(path)
            .ok()
            .map(|text| text.trim().to_owned())
    };
    About {
        distribution: read("/etc/os-release").as_deref().and_then(pretty_name),
        kernel: read("/proc/sys/kernel/osrelease"),
        host: read("/proc/sys/kernel/hostname"),
        hardware: hardware::load(),
    }
}

pub fn keymaps() -> Keymaps {
    fs::read_to_string(RULES)
        .map(|rules| Keymaps::parse(&rules))
        .unwrap_or_default()
}
