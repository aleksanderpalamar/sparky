use std::time::Instant;

use sparky_hardware::stat::CpuTimes;

use super::features::features;
use crate::context::{Load, Process, Scene, Window, recognize};
use crate::domain::{Activities, Features, Metric};
use crate::ports::Files;
use crate::telemetry::processes::ProcessTable;
use crate::telemetry::{Collector, SystemSample};

pub struct Observation {
    pub activities: Activities,
    pub features: Features,
    pub sample: SystemSample,
    pub processes: Option<Vec<Process>>,
    pub windows: Option<Vec<Window>>,
}

pub struct Observer<F> {
    collector: Collector<F>,
    previous_cpu: Option<CpuTimes>,
    processes: ProcessTable,
}

impl<F: Files> Observer<F> {
    pub fn new(collector: Collector<F>) -> Self {
        Self {
            collector,
            previous_cpu: None,
            processes: ProcessTable::default(),
        }
    }

    pub fn observe(&mut self, now: Instant, windows: Option<Vec<Window>>) -> Observation {
        let sample = self.collector.system();
        let features = features(self.previous_cpu, &sample);
        self.previous_cpu = sample.cpu.measured().copied().or(self.previous_cpu);
        let scene = Scene {
            processes: self.processes.update(self.collector.files(), now),
            windows,
            load: load(&features, &sample),
        };
        Observation {
            activities: recognize(&scene),
            features,
            sample,
            processes: scene.processes,
            windows: scene.windows,
        }
    }
}

fn load(features: &Features, sample: &SystemSample) -> Load {
    Load {
        cpu_busy_percent: features.get(Metric::CpuBusy),
        memory_available_percent: features.get(Metric::MemoryUsed).map(|used| 100.0 - used),
        memory_stall_avg10: sample.pressure.memory.measured().map(|psi| psi.some.avg10),
    }
}
