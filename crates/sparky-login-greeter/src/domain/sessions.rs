use std::collections::HashSet;

use sparky_xdg::desktop_entry::DesktopEntry;

const DEFAULT: &str = "sparky";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFile {
    pub id: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Sessions {
    list: Vec<Session>,
    chosen: usize,
}

pub fn sessions(
    files: Vec<SessionFile>,
    locale: &str,
    installed: impl Fn(&str) -> bool,
) -> Vec<Session> {
    let mut seen = HashSet::new();
    let mut list: Vec<Session> = files
        .into_iter()
        .filter(|file| seen.insert(file.id.clone()))
        .filter_map(|file| session(file, locale, &installed))
        .collect();
    list.sort_by_key(|session| session.name.to_lowercase());
    list
}

fn session(file: SessionFile, locale: &str, installed: &impl Fn(&str) -> bool) -> Option<Session> {
    let entry = DesktopEntry::parse(&file.contents)?;
    if entry.hidden || entry.no_display || entry.exec.is_none() || !entry.installed(installed) {
        return None;
    }
    let name = entry.name.get(locale)?.to_owned();
    Some(Session { id: file.id, name })
}

impl Sessions {
    pub fn new(list: Vec<Session>, preferred: Option<&str>) -> Option<Self> {
        if list.is_empty() {
            return None;
        }
        let position = |id: &str| list.iter().position(|session| session.id == id);
        let chosen = preferred
            .and_then(position)
            .or_else(|| position(DEFAULT))
            .unwrap_or(0);
        Some(Self { list, chosen })
    }

    pub fn current(&self) -> &Session {
        &self.list[self.chosen]
    }

    pub fn next(&mut self) {
        self.chosen = (self.chosen + 1) % self.list.len();
    }

    pub fn label(&self) -> String {
        format!("Session ({})", self.current().name)
    }
}
