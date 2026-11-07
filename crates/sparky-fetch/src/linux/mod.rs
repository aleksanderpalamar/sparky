mod command;
mod cpu;
mod disk;
mod files;
mod format;
mod gpu;
mod gtk;
mod host;
mod kernel;
mod memory;
mod os;
mod packages;
mod probe;
mod shell;
mod system;
mod tally;
mod terminal;
mod uptime;

#[cfg(test)]
mod fake;
#[cfg(test)]
mod format_tests;
#[cfg(test)]
mod hardware_tests;
#[cfg(test)]
mod packages_tests;
#[cfg(test)]
mod probe_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod text_tests;

use std::sync::Arc;

use crate::config::settings::Settings;
use crate::domain::field::Field;
use crate::sources::Source;
use host::Host;
use probe::{Collect, Probe};

static PROBES: [(Field, Collect); 12] = [
    (Field::Os, os::collect),
    (Field::Kernel, kernel::collect),
    (Field::Uptime, uptime::collect),
    (Field::Cpu, cpu::collect),
    (Field::Gpu, gpu::collect),
    (Field::Memory, memory::collect),
    (Field::Disk, disk::collect),
    (Field::Terminal, terminal::collect),
    (Field::Shell, shell::collect),
    (Field::Packages, packages::collect),
    (Field::Icons, gtk::icons),
    (Field::Font, gtk::font),
];

pub fn sources(_settings: &Settings) -> Vec<Arc<dyn Source>> {
    wired(Arc::new(system::System))
}

fn wired(host: Arc<dyn Host>) -> Vec<Arc<dyn Source>> {
    PROBES
        .iter()
        .map(|(field, collect)| {
            Arc::new(Probe::new(field, Arc::clone(&host), *collect)) as Arc<dyn Source>
        })
        .collect()
}
