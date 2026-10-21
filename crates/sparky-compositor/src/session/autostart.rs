use std::collections::BTreeMap;

use sparky_xdg::desktop_entry::{DesktopEntry, command_of};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub name: String,
    pub contents: String,
}

pub fn commands(directories: &[Vec<Candidate>], desktop: &str, present: impl Fn(&str) -> bool) -> Vec<String> {
    let mut chosen: BTreeMap<&str, &str> = BTreeMap::new();
    for candidate in directories.iter().flatten() {
        chosen.entry(&candidate.name).or_insert(&candidate.contents);
    }
    chosen
        .values()
        .filter_map(|contents| DesktopEntry::parse(contents))
        .filter(|entry| wanted(entry, desktop, &present))
        .filter_map(|entry| entry.exec.map(|exec| command_of(&exec)))
        .filter(|command| !command.is_empty())
        .collect()
}

fn wanted(entry: &DesktopEntry, desktop: &str, present: &impl Fn(&str) -> bool) -> bool {
    let shown = entry.only_show_in.is_empty() || entry.only_show_in.iter().any(|name| name == desktop);
    let excluded = entry.not_show_in.iter().any(|name| name == desktop);
    let installed = entry.try_exec.as_deref().is_none_or(present);
    entry.enabled && !entry.hidden && shown && !excluded && installed
}
