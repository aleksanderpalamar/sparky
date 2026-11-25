use std::time::Instant;

use sparky_hardware::stat::CpuTimes;

use super::features::features;
use crate::context::{Load, Process, Scene, Window, recognize};
use crate::domain::{Activities, Activity, Features, Metric};
use crate::orchestration::planner::STALL;
use crate::ports::Files;
use crate::telemetry::processes::{ProcessTable, Sweep};
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
        let mut scene = Scene {
            processes: self
                .processes
                .update(self.collector.files(), now, Sweep::Routine),
            windows,
            load: load(&features, &sample),
        };
        let mut activities = recognize(&scene);
        if deciding(activities, &sample) {
            scene.processes = self
                .processes
                .update(self.collector.files(), now, Sweep::Decision);
            activities = recognize(&scene);
        }
        Observation {
            activities,
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

fn deciding(activities: Activities, sample: &SystemSample) -> bool {
    let sensitive =
        activities.contains(Activity::Gaming) || activities.contains(Activity::Recording);
    let contended = [&sample.pressure.cpu, &sample.pressure.io]
        .into_iter()
        .filter_map(|reading| reading.measured())
        .any(|psi| psi.some.avg10 >= STALL);
    sensitive && contended
}
