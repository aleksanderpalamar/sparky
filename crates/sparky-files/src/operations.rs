use std::fmt;
use std::io;
use std::path::Path;

use crate::domain::clip::Clip;
use crate::domain::entry::Kind;
use crate::domain::naming::{Creation, creation, free_name, valid};
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
            Failure::InvalidName => write!(formatter, "nome inválido"),
            Failure::AlreadyExists(name) => write!(formatter, "já existe um item chamado {name}"),
            Failure::IntoItself(name) => {
                write!(formatter, "não dá para colar {name} dentro dela mesma")
            }
            Failure::Io { name, source } => write!(formatter, "{name}: {source}"),
        }
    }
}

pub fn paste(clip: &Clip, into: &Path, disk: &dyn Disk) -> Result<Vec<String>, Failure> {
    clip.paths()
        .iter()
        .map(|source| paste_one(clip, source, into, disk))
        .collect()
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

fn paste_one(clip: &Clip, source: &Path, into: &Path, disk: &dyn Disk) -> Result<String, Failure> {
    let name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if into.starts_with(source) {
        return Err(Failure::IntoItself(name));
    }
    if matches!(clip, Clip::Cut(_)) && source.parent() == Some(into) {
        return Ok(name);
    }
    let target = free_name(&name, |candidate| disk.exists(&into.join(candidate)));
    let result = match clip {
        Clip::Copy(_) => disk.copy(source, &into.join(&target)),
        Clip::Cut(_) => disk.relocate(source, &into.join(&target)),
    };
    result.map(|()| target).map_err(failed(&name))
}

fn failed(name: &str) -> impl FnOnce(io::Error) -> Failure + '_ {
    move |source| Failure::Io {
        name: name.to_owned(),
        source,
    }
}
