use std::collections::{BTreeMap, HashMap};

use super::usage::{AppRow, ProcessRow, Usage};

const GENERIC: [&str; 13] = [
    "sh", "bash", "zsh", "dash", "env", "python", "python3", "perl", "flatpak", "java", "sudo",
    "pkexec", "gjs",
];
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub program: String,
}

pub fn program_of(exec: &str) -> Option<String> {
    let token = exec
        .split_whitespace()
        .find(|token| *token != "env" && !assignment(token))?;
    let name = token.trim_matches('"').rsplit('/').next()?;
    (!name.is_empty() && !GENERIC.contains(&name)).then(|| name.to_owned())
}

fn assignment(token: &str) -> bool {
    token.contains('=') && !token.starts_with('-')
}

pub fn group(processes: &[ProcessRow], catalog: &[AppEntry], user: u32) -> Vec<AppRow> {
    let programs: HashMap<&str, &AppEntry> = catalog
        .iter()
        .rev()
        .map(|entry| (entry.program.as_str(), entry))
        .collect();
    let parents: HashMap<u32, &ProcessRow> = processes.iter().map(|row| (row.pid, row)).collect();
    let mut apps: BTreeMap<&str, AppRow> = BTreeMap::new();
    for process in processes.iter().filter(|row| row.user == Some(user)) {
        let Some(entry) = owner(process, &parents, &programs) else {
            continue;
        };
        let app = apps.entry(entry.id.as_str()).or_insert_with(|| AppRow {
            id: entry.id.clone(),
            name: entry.name.clone(),
            icon: entry.icon.clone(),
            pids: Vec::new(),
            usage: Usage::default(),
        });
        app.pids.push(process.pid);
        app.usage = app.usage + process.usage;
    }
    apps.into_values().collect()
}

fn owner<'a>(
    process: &ProcessRow,
    parents: &HashMap<u32, &ProcessRow>,
    programs: &HashMap<&str, &'a AppEntry>,
) -> Option<&'a AppEntry> {
    let mut current = Some(process);
    for _ in 0..MAX_DEPTH {
        let row = current?;
        if let Some(entry) = programs.get(row.program.as_str()) {
            return Some(entry);
        }
        current = parents
            .get(&row.ppid)
            .copied()
            .filter(|parent| parent.pid != row.pid);
    }
    None
}
