use std::collections::{HashMap, HashSet};

use crate::context::Process;
use crate::context::names::{CALLS, GAME_HOSTS, GAME_LAUNCHERS, RECORDERS, listed};

const SHOWN: usize = 8;

const ESSENTIAL: [&str; 12] = [
    "systemd",
    "sparky",
    "sparky-resource",
    "sparky-bar",
    "Xwayland",
    "pipewire",
    "pipewire-pulse",
    "wireplumber",
    "pulseaudio",
    "dbus-daemon",
    "dbus-broker",
    "polkit-sparky",
];

pub fn protected(processes: &[Process], own: u32, chosen: &[String], focused: &[u32]) -> Vec<u32> {
    let roots: Vec<u32> = processes
        .iter()
        .filter(|process| {
            activity(&process.name) || chosen.iter().any(|name| name == &process.name)
        })
        .map(|process| process.pid)
        .chain(focused.iter().copied())
        .collect();
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for process in processes {
        children
            .entry(process.parent)
            .or_default()
            .push(process.pid);
    }
    let mut kept: HashSet<u32> = processes
        .iter()
        .filter(|process| essential(&process.name))
        .map(|process| process.pid)
        .chain([own])
        .collect();
    let mut pending = roots;
    let mut seen = HashSet::new();
    while let Some(pid) = pending.pop() {
        kept.insert(pid);
        if seen.insert(pid) {
            pending.extend(children.get(&pid).into_iter().flatten().copied());
        }
    }
    let mut kept: Vec<u32> = kept.into_iter().collect();
    kept.sort_unstable();
    kept
}

pub fn guarded(processes: &[Process], chosen: &[String], focused: &[String]) -> Vec<String> {
    let mut names: Vec<String> = processes
        .iter()
        .map(|process| process.name.as_str())
        .filter(|name| {
            listed(&RECORDERS, name)
                || listed(&CALLS, name)
                || chosen.iter().any(|chosen| chosen == name)
        })
        .map(str::to_owned)
        .chain(focused.iter().cloned())
        .collect();
    names.sort_unstable();
    names.dedup();
    names.truncate(SHOWN);
    names
}

fn activity(name: &str) -> bool {
    [&GAME_LAUNCHERS[..], &GAME_HOSTS, &RECORDERS, &CALLS]
        .iter()
        .any(|list| listed(list, name))
}

fn essential(name: &str) -> bool {
    listed(&ESSENTIAL, name) || name.starts_with("sparky")
}
