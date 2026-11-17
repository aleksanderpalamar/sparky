use std::path::{Path, PathBuf};

use super::Purpose;
use crate::domain::entry::Kind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Choose(Vec<PathBuf>),
    Enter(PathBuf),
    Confirm(Vec<PathBuf>),
    Nothing,
}

pub fn decide(
    purpose: &Purpose,
    location: &Path,
    selected: &[(PathBuf, Kind)],
    exists: impl Fn(&Path) -> bool,
) -> Decision {
    match purpose {
        Purpose::Open {
            multiple,
            directory: false,
        } => open_files(selected, *multiple),
        Purpose::Open {
            multiple,
            directory: true,
        } => open_folders(location, selected, *multiple),
        Purpose::Save { name } => save(location, selected, name, exists),
        Purpose::SaveFiles { names } => save_files(location, selected, names, exists),
    }
}

fn open_files(selected: &[(PathBuf, Kind)], multiple: bool) -> Decision {
    let files = of_kind(selected, Kind::File);
    match (files.as_slice(), single_folder(selected)) {
        ([], Some(folder)) => Decision::Enter(folder),
        ([], None) => Decision::Nothing,
        (_, _) => Decision::Choose(limited(files, multiple)),
    }
}

fn open_folders(location: &Path, selected: &[(PathBuf, Kind)], multiple: bool) -> Decision {
    let folders = of_kind(selected, Kind::Directory);
    match folders.is_empty() {
        true => Decision::Choose(vec![location.to_path_buf()]),
        false => Decision::Choose(limited(folders, multiple)),
    }
}

fn save(
    location: &Path,
    selected: &[(PathBuf, Kind)],
    name: &str,
    exists: impl Fn(&Path) -> bool,
) -> Decision {
    let name = name.trim();
    if !valid_name(name) {
        return single_folder(selected).map_or(Decision::Nothing, Decision::Enter);
    }
    checked(vec![location.join(name)], exists)
}

fn save_files(
    location: &Path,
    selected: &[(PathBuf, Kind)],
    names: &[String],
    exists: impl Fn(&Path) -> bool,
) -> Decision {
    let folder = single_folder(selected).unwrap_or_else(|| location.to_path_buf());
    let paths = names
        .iter()
        .filter(|name| valid_name(name))
        .map(|name| folder.join(name))
        .collect();
    checked(paths, exists)
}

fn checked(paths: Vec<PathBuf>, exists: impl Fn(&Path) -> bool) -> Decision {
    match paths.iter().any(|path| exists(path)) {
        true => Decision::Confirm(paths),
        false => Decision::Choose(paths),
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/')
}

fn of_kind(selected: &[(PathBuf, Kind)], kind: Kind) -> Vec<PathBuf> {
    selected
        .iter()
        .filter(|(_, found)| *found == kind)
        .map(|(path, _)| path.clone())
        .collect()
}

fn single_folder(selected: &[(PathBuf, Kind)]) -> Option<PathBuf> {
    match selected {
        [(path, Kind::Directory)] => Some(path.clone()),
        _ => None,
    }
}

fn limited(mut paths: Vec<PathBuf>, multiple: bool) -> Vec<PathBuf> {
    if !multiple {
        paths.truncate(1);
    }
    paths
}
