use std::time::Instant;

use sparky_hardware::stat::CpuTimes;

use super::features::features;
use crate::context::names::{RECORDERS, listed};
use crate::context::{Load, Process, Scene, Sensed, Window, recognize};
use crate::domain::{Activities, Activity, Features, Metric};
use crate::orchestration::planner::STALL;
use crate::ports::{CaptureSensor, Files};
use crate::telemetry::processes::{ProcessTable, Sweep};
use crate::telemetry::{Collector, SystemSample};

pub struct Observation {
    pub activities: Activities,
    pub features: Features,
    pub sample: SystemSample,
    pub processes: Option<Vec<Process>>,
    pub windows: Option<Vec<Window>>,
    pub capture: Option<(u32, Sensed)>,
}

pub struct Observer<F> {
    collector: Collector<F>,
    previous_cpu: Option<CpuTimes>,
    processes: ProcessTable,
    sensor: Option<Box<dyn CaptureSensor + Send>>,
}

impl<F: Files> Observer<F> {
    pub fn new(collector: Collector<F>) -> Self {
        Self {
            collector,
            previous_cpu: None,
            processes: ProcessTable::default(),
            sensor: None,
        }
    }

    pub fn sensing(mut self, sensor: Box<dyn CaptureSensor + Send>) -> Self {
        self.sensor = Some(sensor);
        self
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
            capture: None,
        };
        let capture = self.sense(scene.processes.as_deref(), now);
        scene.capture = capture;
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
            capture,
        }
    }
}

impl<F> Observer<F> {
    fn sense(&mut self, processes: Option<&[Process]>, now: Instant) -> Option<(u32, Sensed)> {
        let mut recorders = processes?
            .iter()
            .filter(|process| listed(&RECORDERS, &process.name));
        let pid = recorders.next()?.pid;
        if recorders.next().is_some() {
            return None;
        }
        Some((pid, self.sensor.as_mut()?.sense(now)))
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
