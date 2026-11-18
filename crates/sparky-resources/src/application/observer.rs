use std::time::Instant;

use sparky_hardware::stat::{CpuTimes, usage};

use crate::context::{Load, Scene, Window, recognize};
use crate::domain::Activities;
use crate::ports::Files;
use crate::telemetry::processes::ProcessTable;
use crate::telemetry::{Collector, SystemSample};

pub struct Observation {
    pub activities: Activities,
    pub sample: SystemSample,
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
        let scene = Scene {
            processes: self.processes.update(self.collector.files(), now),
            windows,
            load: self.load(&sample),
        };
        Observation {
            activities: recognize(&scene),
            sample,
        }
    }

    fn load(&mut self, sample: &SystemSample) -> Load {
        let current = sample.cpu.measured().copied();
        let busy = self
            .previous_cpu
            .zip(current)
            .map(|(before, after)| usage(before, after));
        self.previous_cpu = current.or(self.previous_cpu);
        let memory = sample.memory.measured();
        Load {
            cpu_busy_percent: busy,
            memory_available_percent: memory
                .filter(|memory| memory.total_kib > 0)
                .map(|memory| memory.available_kib as f32 / memory.total_kib as f32 * 100.0),
            memory_stall_avg10: sample.pressure.memory.measured().map(|psi| psi.some.avg10),
        }
    }
}
