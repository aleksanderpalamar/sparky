use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gio::prelude::*;

use crate::domain::entry::{Entry, Kind};

const ATTRIBUTES: &str =
    "standard::name,standard::type,standard::size,time::modified,time::modified-usec";

pub fn remote_root() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|runtime| runtime.join("gvfs"))
}

pub fn serves(root: &Path, dir: &Path) -> bool {
    dir.starts_with(root) && dir != root
}

pub fn list(dir: &Path) -> io::Result<Vec<Entry>> {
    let children = gio::File::for_path(dir)
        .enumerate_children(
            ATTRIBUTES,
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        )
        .map_err(failure)?;
    children
        .map(|info| info.map(|info| entry_of(&info)).map_err(failure))
        .collect()
}

fn entry_of(info: &gio::FileInfo) -> Entry {
    Entry {
        name: info.name().to_string_lossy().into_owned(),
        kind: match info.file_type() {
            gio::FileType::Directory => Kind::Directory,
            _ => Kind::File,
        },
        size: u64::try_from(info.size()).unwrap_or(0),
        modified: moment_of(info),
    }
}

fn moment_of(info: &gio::FileInfo) -> Option<SystemTime> {
    if !info.has_attribute("time::modified") {
        return None;
    }
    let seconds = info.attribute_uint64("time::modified");
    let micros = u64::from(info.attribute_uint32("time::modified-usec"));
    UNIX_EPOCH.checked_add(Duration::from_secs(seconds) + Duration::from_micros(micros))
}

fn failure(error: gio::glib::Error) -> io::Error {
    let kind = match error.kind::<gio::IOErrorEnum>() {
        Some(gio::IOErrorEnum::NotFound) => io::ErrorKind::NotFound,
        Some(gio::IOErrorEnum::PermissionDenied) => io::ErrorKind::PermissionDenied,
        Some(gio::IOErrorEnum::TimedOut) => io::ErrorKind::TimedOut,
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, error.message().to_owned())
}
