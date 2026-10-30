use std::collections::BTreeMap;

use super::history::History;
use super::measure::Reading;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pair {
    pub first: History,
    pub second: History,
}

impl Pair {
    fn push(&mut self, first: f64, second: f64) {
        self.first.push(first as f32);
        self.second.push(second as f32);
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Histories {
    pub cpu: History,
    pub cores: Vec<History>,
    pub memory: History,
    pub swap: History,
    pub disk: Pair,
    pub disks: BTreeMap<String, Pair>,
    pub network: Pair,
    pub networks: BTreeMap<String, Pair>,
    pub gpus: Vec<Pair>,
}

impl Histories {
    pub fn record(&mut self, reading: &Reading) {
        self.cpu.push(reading.cpu);
        self.cores
            .resize_with(reading.cores.len(), History::default);
        for (history, usage) in self.cores.iter_mut().zip(&reading.cores) {
            history.push(*usage);
        }
        if let Some(memory) = reading.memory {
            self.memory.push(memory.percent());
            self.swap.push(share(memory.swap_used(), memory.swap_total));
        }
        let (read, write) = reading
            .disks
            .iter()
            .fold((0.0, 0.0), |(read, write), disk| {
                (read + disk.read, write + disk.write)
            });
        self.disk.push(read, write);
        for disk in &reading.disks {
            let pair = self
                .disks
                .entry(disk.sample.counters.name.clone())
                .or_default();
            pair.push(disk.read, disk.write);
        }
        let (received, sent) = reading
            .networks
            .iter()
            .fold((0.0, 0.0), |(received, sent), net| {
                (received + net.received, sent + net.sent)
            });
        self.network.push(received, sent);
        for net in &reading.networks {
            self.networks
                .entry(net.name.clone())
                .or_default()
                .push(net.received, net.sent);
        }
        self.gpus.resize_with(reading.gpus.len(), Pair::default);
        for (pair, gpu) in self.gpus.iter_mut().zip(&reading.gpus) {
            let vram = share(
                gpu.vram_used.unwrap_or_default(),
                gpu.vram_total.unwrap_or_default(),
            );
            pair.push(f64::from(gpu.busy.unwrap_or_default()), f64::from(vram));
        }
    }
}

fn share(part: u64, total: u64) -> f32 {
    match total {
        0 => 0.0,
        total => (part as f64 / total as f64 * 100.0) as f32,
    }
}
