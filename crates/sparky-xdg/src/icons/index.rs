use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path};

use super::rank::{Details, rank};

const THEME_SECTION: &str = "Icon Theme";

pub struct Index {
    pub directories: Vec<String>,
    pub inherits: Vec<String>,
}

pub fn read(theme: &Path, pixels: Option<u32>) -> Option<Index> {
    let contents = fs::read_to_string(theme.join("index.theme")).ok()?;
    Some(parse(&contents, pixels))
}

fn parse(contents: &str, pixels: Option<u32>) -> Index {
    let mut section = String::new();
    let mut inherits = Vec::new();
    let mut directories = Vec::new();
    let mut details: HashMap<String, Details> = HashMap::new();
    for line in contents.lines().map(str::trim) {
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
        {
            section = name.to_owned();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match (section.as_str(), key) {
            (THEME_SECTION, "Inherits") => inherits.extend(list(value)),
            (THEME_SECTION, "Directories" | "ScaledDirectories") => directories.extend(list(value)),
            (THEME_SECTION, _) | ("", _) => {}
            (directory, key) => details
                .entry(directory.to_owned())
                .or_default()
                .set(key, value),
        }
    }
    let mut seen = BTreeSet::new();
    directories.retain(|directory| valid(directory) && seen.insert(directory.clone()));
    directories.sort_by_key(|directory| rank(directory, details.get(directory), pixels));
    Index {
        directories,
        inherits,
    }
}

fn list(value: &str) -> impl Iterator<Item = String> + '_ {
    value.split(',').map(str::trim).map(str::to_owned)
}

fn valid(directory: &str) -> bool {
    !directory.is_empty()
        && Path::new(directory)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
