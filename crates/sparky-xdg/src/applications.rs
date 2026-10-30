use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::desktop_entry::DesktopEntry;
use crate::programs::installed;

const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

pub fn directories(data_home: Option<&str>, home: &str, data_dirs: Option<&str>) -> Vec<PathBuf> {
    let filled = |value: Option<&str>| {
        value
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    let user = filled(data_home).unwrap_or_else(|| format!("{home}/.local/share"));
    let shared = filled(data_dirs).unwrap_or_else(|| DEFAULT_DATA_DIRS.to_owned());
    std::iter::once(user.as_str())
        .chain(shared.split(':'))
        .filter(|directory| !directory.is_empty())
        .map(|directory| Path::new(directory).join("applications"))
        .collect()
}

pub fn desktop_id(relative: &Path) -> Option<String> {
    let text = relative.to_str()?;
    let stem = text.strip_suffix(".desktop")?;
    Some(stem.replace('/', "-"))
}

pub fn installed_entries() -> Vec<(String, DesktopEntry)> {
    let home = env::var("HOME").unwrap_or_default();
    let data_home = env::var("XDG_DATA_HOME").ok();
    let data_dirs = env::var("XDG_DATA_DIRS").ok();
    entries_in(&directories(
        data_home.as_deref(),
        &home,
        data_dirs.as_deref(),
    ))
}

pub fn entries_in(directories: &[PathBuf]) -> Vec<(String, DesktopEntry)> {
    let mut seen: BTreeMap<String, Option<DesktopEntry>> = BTreeMap::new();
    for directory in directories {
        for (id, path) in desktop_files(directory) {
            if let Entry::Vacant(slot) = seen.entry(id) {
                slot.insert(read(&path));
            }
        }
    }
    seen.into_iter()
        .filter_map(|(id, entry)| entry.map(|entry| (id, entry)))
        .collect()
}

fn read(path: &Path) -> Option<DesktopEntry> {
    let entry = DesktopEntry::parse(&fs::read_to_string(path).ok()?)?;
    entry.installed(installed).then_some(entry)
}

fn desktop_files(root: &Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(items) = fs::read_dir(&directory) else {
            continue;
        };
        for path in items.filter_map(Result::ok).map(|item| item.path()) {
            match path.is_dir() {
                true => pending.push(path),
                false => {
                    let id = path.strip_prefix(root).ok().and_then(desktop_id);
                    found.extend(id.map(|id| (id, path)));
                }
            }
        }
    }
    found
}
