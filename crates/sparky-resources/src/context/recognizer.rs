use std::collections::HashMap;

use super::names::{CALLS, COMPILERS, EDITORS, GAME_HOSTS, GAME_LAUNCHERS, RECORDERS, listed};
use super::scene::{Load, Process, Scene};
use crate::domain::{Activities, Activity};

const CRITICAL_STALL: f32 = 10.0;
const CRITICAL_AVAILABLE: f32 = 5.0;
const IDLE_BUSY: f32 = 5.0;
const ANCESTRY_LIMIT: usize = 32;

pub fn recognize(scene: &Scene) -> Activities {
    let mut activities = Activities::default();
    if critical(scene.load) {
        activities = activities.with(Activity::CriticalPressure);
    }
    if let Some(processes) = &scene.processes {
        activities = by_processes(activities, processes);
    }
    settle(activities, scene)
}

fn by_processes(activities: Activities, processes: &[Process]) -> Activities {
    let any = |list: &[&str]| processes.iter().any(|process| listed(list, &process.name));
    let mut found = activities;
    if Tree::new(processes).has_game() || any(&GAME_HOSTS) {
        found = found.with(Activity::Gaming);
    }
    for (list, activity) in [
        (&RECORDERS[..], Activity::Recording),
        (&CALLS[..], Activity::Communication),
        (&COMPILERS[..], Activity::Compiling),
        (&EDITORS[..], Activity::Development),
    ] {
        if any(list) {
            found = found.with(activity);
        }
    }
    found
}

fn settle(activities: Activities, scene: &Scene) -> Activities {
    if activities != Activities::default() {
        return activities;
    }
    let busy = scene.load.cpu_busy_percent;
    if busy.is_some_and(|busy| busy < IDLE_BUSY) {
        return activities.with(Activity::Idle);
    }
    if busy.is_none() && scene.processes.is_none() && scene.windows.is_none() {
        return activities.with(Activity::Unknown);
    }
    activities.with(Activity::Desktop)
}

fn critical(load: Load) -> bool {
    let stalled = load
        .memory_stall_avg10
        .is_some_and(|stall| stall >= CRITICAL_STALL);
    let starved = load
        .memory_available_percent
        .is_some_and(|available| available < CRITICAL_AVAILABLE);
    stalled || starved
}

struct Tree<'a> {
    parents: HashMap<u32, &'a Process>,
}

impl<'a> Tree<'a> {
    fn new(processes: &'a [Process]) -> Self {
        Self {
            parents: processes
                .iter()
                .map(|process| (process.pid, process))
                .collect(),
        }
    }

    fn has_game(&self) -> bool {
        self.parents.values().any(|process| {
            !listed(&GAME_LAUNCHERS, &process.name) && self.descends_from_launcher(process.pid)
        })
    }

    fn descends_from_launcher(&self, pid: u32) -> bool {
        let mut current = self.parents.get(&pid).map(|process| process.parent);
        for _ in 0..ANCESTRY_LIMIT {
            let Some(parent) = current.and_then(|pid| self.parents.get(&pid)) else {
                return false;
            };
            if listed(&GAME_LAUNCHERS, &parent.name) {
                return true;
            }
            current = Some(parent.parent);
        }
        false
    }
}
