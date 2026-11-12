use std::collections::HashSet;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::naming::{Creation, creation, free_name, valid};
use crate::domain::transfer::{Action, Conflict, Step};
use crate::i18n;
use crate::ports::Disk;

#[derive(Debug)]
pub enum Failure {
    InvalidName,
    AlreadyExists(String),
    IntoItself(String),
    Io { name: String, source: io::Error },
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::InvalidName => write!(formatter, "{}", i18n::text("invalid-name")),
            Failure::AlreadyExists(name) => {
                write!(
                    formatter,
                    "{}",
                    i18n::with("item-exists", "name", name.clone())
                )
            }
            Failure::IntoItself(name) => {
                write!(
                    formatter,
                    "{}",
                    i18n::with("into-itself", "name", name.clone())
                )
            }
            Failure::Io { name, source } => write!(formatter, "{name}: {source}"),
        }
    }
}

pub struct Planned {
    pub steps: Vec<Step>,
    pub skipped: usize,
}

pub fn conflicts(clip: &Clip, into: &Path, disk: &dyn Disk) -> Vec<String> {
    let Ok(resolved_into) = disk.canonicalize(into) else {
        return Vec::new();
    };
    clip.paths()
        .iter()
        .filter(|source| !same_folder(source, &resolved_into, disk))
        .map(|source| file_name(source))
        .filter(|name| disk.exists(&into.join(name)))
        .collect()
}

pub fn plan(
    clip: &Clip,
    into: &Path,
    disk: &dyn Disk,
    conflict: Conflict,
) -> Result<Planned, Failure> {
    let resolved_into = disk.canonicalize(into).map_err(failed("destino"))?;
    let mut planned = Planned {
        steps: Vec::new(),
        skipped: 0,
    };
    let mut chosen = HashSet::new();
    for source in clip.paths() {
        let name = file_name(source);
        let resolved_source = source_path(source, disk).map_err(failed(&name))?;
        if resolved_into.starts_with(&resolved_source) {
            return Err(Failure::IntoItself(name));
        }
        let same = resolved_source.parent() == Some(resolved_into.as_path());
        let action = match clip {
            Clip::Copy(_) => Action::Copy,
            Clip::Cut(_) if same => continue,
            Clip::Cut(_) => Action::Move,
        };
        let taken =
            |candidate: &str| chosen.contains(candidate) || disk.exists(&into.join(candidate));
        let (target, replace) = match (taken(&name), same, conflict) {
            (false, _, _) => (name.clone(), false),
            (true, true, _) | (true, _, Conflict::KeepBoth) => (free_name(&name, taken), false),
            (true, false, Conflict::Skip) => {
                planned.skipped += 1;
                continue;
            }
            (true, false, Conflict::Replace) => (name.clone(), true),
        };
        if replace && resolved_source.starts_with(resolved_into.join(&target)) {
            return Err(Failure::IntoItself(name));
        }
        chosen.insert(target.clone());
        planned.steps.push(Step {
            source: source.clone(),
            target: into.join(target),
            action,
            replace,
        });
    }
    Ok(planned)
}

pub fn rename(dir: &Path, old: &str, new: &str, disk: &dyn Disk) -> Result<(), Failure> {
    if !valid(new) {
        return Err(Failure::InvalidName);
    }
    if old == new {
        return Ok(());
    }
    let target = dir.join(new);
    if disk.exists(&target) {
        return Err(Failure::AlreadyExists(new.to_owned()));
    }
    disk.relocate(&dir.join(old), &target).map_err(failed(old))
}

pub fn create(dir: &Path, input: &str, disk: &dyn Disk) -> Result<String, Failure> {
    let (name, kind) = match creation(input).ok_or(Failure::InvalidName)? {
        Creation::Folder(name) => (name, Kind::Directory),
        Creation::File(name) => (name, Kind::File),
    };
    create_as(dir, &name, kind, disk)
}

pub fn create_as(dir: &Path, input: &str, kind: Kind, disk: &dyn Disk) -> Result<String, Failure> {
    let name = input.trim();
    if !valid(name) {
        return Err(Failure::InvalidName);
    }
    let path = dir.join(name);
    if disk.exists(&path) {
        return Err(Failure::AlreadyExists(name.to_owned()));
    }
    let result = match kind {
        Kind::Directory => disk.create_dir(&path),
        Kind::File => disk.create_file(&path),
    };
    result.map(|()| name.to_owned()).map_err(failed(name))
}

pub fn trash(dir: &Path, names: &[String], disk: &dyn Disk) -> Result<usize, Failure> {
    for name in names {
        disk.trash(&dir.join(name)).map_err(failed(name))?;
    }
    Ok(names.len())
}

fn same_folder(source: &Path, resolved_into: &Path, disk: &dyn Disk) -> bool {
    source_path(source, disk).is_ok_and(|resolved| resolved.parent() == Some(resolved_into))
}

fn file_name(source: &Path) -> String {
    source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn source_path(source: &Path, disk: &dyn Disk) -> io::Result<PathBuf> {
    let parent = source.parent().unwrap_or(Path::new("."));
    let name = source
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "origem sem nome"))?;
    disk.canonicalize(parent).map(|path| path.join(name))
}

fn failed(name: &str) -> impl FnOnce(io::Error) -> Failure + '_ {
    move |source| Failure::Io {
        name: name.to_owned(),
        source,
    }
}
