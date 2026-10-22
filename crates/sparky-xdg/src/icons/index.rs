use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct Index {
    pub directories: Vec<PathBuf>,
    pub inherits: Vec<String>,
}

pub fn read(root: &Path) -> Index {
    let contents = fs::read_to_string(root.join("index.theme")).unwrap_or_default();
    let mut current = false;
    let mut inherits = Vec::new();
    let mut directories = Vec::new();
    for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
            current = line == "[Icon Theme]";
            continue;
        }
        if !current {
            continue;
        }
        match line.split_once('=').map(|(key, value)| (key.trim(), value.trim())) {
            Some(("Inherits", value)) => inherits.extend(value.split(',').map(str::trim).map(str::to_owned)),
            Some(("Directories" | "ScaledDirectories", value)) => {
                directories.extend(value.split(',').map(str::trim).map(str::to_owned))
            }
            _ => {}
        }
    }
    directories.retain(|directory| {
        !directory.is_empty()
            && Path::new(directory)
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
    });
    directories.sort_by_key(|directory| rank(directory));
    directories.dedup();
    Index {
        directories: directories.into_iter().map(|directory| root.join(directory)).collect(),
        inherits,
    }
}

fn rank(directory: &str) -> usize {
    let size = directory.split('/').next().unwrap_or_default();
    [
        "48x48", "64x64", "scalable", "32x32", "256x256", "24x24", "22x22", "16x16",
    ]
    .iter()
    .position(|candidate| *candidate == size)
    .unwrap_or(8)
}
