use std::time::Instant;

use crate::application::Observation;
use crate::domain::{Activity, Authority};
use crate::orchestration::{Conductor, Contention, Moment, protected};
use crate::platform::{LinuxFiles, SystemExecutor};
use crate::ports::Storage;
use crate::telemetry::consumers::Consumers;

pub struct Orchestra<S> {
    pub conductor: Conductor<SystemExecutor, S>,
    consumers: Consumers,
    chosen: Vec<String>,
}

impl<S: Storage> Orchestra<S> {
    pub fn new(conductor: Conductor<SystemExecutor, S>) -> Self {
        Self {
            conductor,
            consumers: Consumers::default(),
            chosen: Vec::new(),
        }
    }

    pub fn protect(&mut self, names: &[String]) {
        if self.chosen != names {
            self.chosen = names.to_vec();
        }
    }

    pub fn tick(&mut self, observation: &Observation, authority: Authority, seconds: u64) {
        let contention = Contention {
            cpu_stall: observation
                .sample
                .pressure
                .cpu
                .measured()
                .map(|psi| psi.some.avg10),
            io_stall: observation
                .sample
                .pressure
                .io
                .measured()
                .map(|psi| psi.some.avg10),
        };
        let sensitive = observation.activities.contains(Activity::Gaming)
            || observation.activities.contains(Activity::Recording);
        let contended = [contention.cpu_stall, contention.io_stall]
            .iter()
            .flatten()
            .any(|stall| *stall >= crate::orchestration::planner::STALL);
        let processes = observation.processes.as_deref().unwrap_or_default();
        let consumers = match sensitive && contended {
            true => self.consumers.measure(
                &LinuxFiles,
                &processes
                    .iter()
                    .map(|process| process.pid)
                    .collect::<Vec<_>>(),
            ),
            false => Vec::new(),
        };
        let focused: Vec<u32> = observation
            .windows
            .iter()
            .flatten()
            .filter(|window| window.focused || window.fullscreen)
            .filter_map(|window| window.pid)
            .collect();
        let shielded = protected(processes, std::process::id(), &self.chosen, &focused);
        let confidence = if observation.processes.is_some() {
            1.0
        } else {
            0.0
        };
        self.conductor.tick(&Moment {
            at: Instant::now(),
            seconds,
            activities: observation.activities,
            contention,
            consumers: &consumers,
            protected: &shielded,
            authority,
            confidence,
        });
    }
}
