use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct Index {
    pub directories: Vec<PathBuf>,
    pub inherits: Vec<String>,
}

pub fn read(root: &Path, pixels: Option<u32>) -> Index {
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
        match line
            .split_once('=')
            .map(|(key, value)| (key.trim(), value.trim()))
        {
            Some(("Inherits", value)) => {
                inherits.extend(value.split(',').map(str::trim).map(str::to_owned))
            }
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
    directories.sort_by_key(|directory| rank(directory, pixels));
    directories.dedup();
    Index {
        directories: directories
            .into_iter()
            .map(|directory| root.join(directory))
            .collect(),
        inherits,
    }
}

fn rank(directory: &str, pixels: Option<u32>) -> (u8, u32) {
    let size = directory.split('/').next().unwrap_or_default();
    if let Some(pixels) = pixels {
        return physical_size(size).map_or((3, 0), |size| {
            if size >= pixels {
                (0, size - pixels)
            } else {
                (2, pixels - size)
            }
        });
    }
    (
        0,
        [
            "48x48", "64x64", "scalable", "32x32", "256x256", "24x24", "22x22", "16x16",
        ]
        .iter()
        .position(|candidate| *candidate == size)
        .unwrap_or(8) as u32,
    )
}

fn physical_size(size: &str) -> Option<u32> {
    if size == "scalable" {
        return Some(u32::MAX);
    }
    let (dimension, scale) = size.split_once('@').unwrap_or((size, "1x"));
    let (width, height) = dimension.split_once('x')?;
    if width != height {
        return None;
    }
    let scale = scale.strip_suffix('x')?.parse::<u32>().ok()?;
    width.parse::<u32>().ok()?.checked_mul(scale)
}
