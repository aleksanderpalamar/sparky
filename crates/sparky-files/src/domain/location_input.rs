use std::path::{Component, Path, PathBuf};

use sparky_ui::natural;

use crate::domain::destination;

pub const COMPLETIONS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Typed {
    Folder(PathBuf),
    Remote(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub folder: PathBuf,
    pub prefix: String,
    pub kept: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub kept: String,
    pub names: Vec<String>,
    pub selected: usize,
}

impl Completion {
    pub fn text(&self) -> String {
        let name = self.names.get(self.selected).map_or("", String::as_str);
        format!("{}{name}", self.kept)
    }

    pub fn step(&mut self, forward: bool) {
        let count = self.names.len().max(1);
        self.selected = match forward {
            true => (self.selected + 1) % count,
            false => (self.selected + count - 1) % count,
        };
    }
}

pub fn typed(input: &str, current: &Path, home: &Path) -> Option<Typed> {
    let input = input.trim();
    if let Some(uri) = remote(input) {
        return Some(Typed::Remote(uri.to_owned()));
    }
    let local = input.strip_prefix("file://").unwrap_or(input);
    destination::resolve(local, current, home).map(|path| Typed::Folder(normalize(&path)))
}

pub fn normalize(path: &Path) -> PathBuf {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                clean.pop();
            }
            other => clean.push(other),
        }
    }
    clean
}

pub fn pattern(input: &str, resolved: &Path) -> Option<Pattern> {
    let split = input.rfind('/').map_or(0, |index| index + 1);
    let (kept, prefix) = input.split_at(split);
    let folder = match prefix.is_empty() {
        true => resolved.to_path_buf(),
        false => resolved.parent()?.to_path_buf(),
    };
    Some(Pattern {
        folder,
        prefix: prefix.to_owned(),
        kept: kept.to_owned(),
    })
}

pub fn matches(prefix: &str, names: impl IntoIterator<Item = String>) -> Vec<String> {
    let wanted = prefix.to_lowercase();
    let mut found: Vec<String> = names
        .into_iter()
        .filter(|name| !prefix.is_empty() || !name.starts_with('.'))
        .filter(|name| name.to_lowercase().starts_with(&wanted))
        .collect();
    found.sort_by(|left, right| natural::compare(left, right));
    found.truncate(COMPLETIONS);
    found
}

fn remote(input: &str) -> Option<&str> {
    let (scheme, rest) = input.split_once("://")?;
    let valid = !scheme.is_empty()
        && scheme
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || "+-.".contains(letter));
    let host = !rest.is_empty() && !rest.starts_with('/');
    (valid && host && scheme != "file").then_some(input)
}
