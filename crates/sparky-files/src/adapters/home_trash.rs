use rustix::fs::{CWD, RenameFlags, renameat_with};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Component;
use std::path::{Path, PathBuf};

use crate::domain::naming::free_name;
use crate::domain::trash_info::{info, original};
use crate::ports::{EmptyReport, TrashStore};

#[derive(Clone)]
pub struct HomeTrash {
    root: PathBuf,
}

impl TrashStore for HomeTrash {
    fn open(&self) -> io::Result<PathBuf> {
        let path = self.directory();
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn directory(&self) -> PathBuf {
        self.root.join("files")
    }

    fn names(&self) -> io::Result<Vec<OsString>> {
        let entries = match fs::read_dir(self.directory()) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        entries
            .map(|entry| entry.map(|item| item.file_name()))
            .collect()
    }

    fn restore(&self, name: &OsStr) -> io::Result<PathBuf> {
        let path = Path::new(name);
        if !matches!(path.components().next(), Some(Component::Normal(_)))
            || path.components().count() != 1
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nome inválido na lixeira",
            ));
        }
        let source = self.directory().join(name);
        let mut record_name = name.to_os_string();
        record_name.push(".trashinfo");
        let record = self.root.join("info").join(record_name);
        if fs::metadata(&record)?.len() > 16 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "registro da lixeira muito grande",
            ));
        }
        let target = original(&fs::read_to_string(&record)?)?;
        renameat_with(CWD, &source, CWD, &target, RenameFlags::NOREPLACE)
            .map_err(io::Error::from)?;
        fs::remove_file(record)?;
        Ok(target)
    }

    fn empty(&self) -> io::Result<EmptyReport> {
        let files = self.directory();
        let infos = self.root.join("info");
        if !safe_directory(&files)? {
            return Ok(EmptyReport::default());
        }
        safe_directory(&infos)?;
        let mut report = EmptyReport::default();
        for name in self.names()? {
            let path = files.join(&name);
            if let Err(error) = remove_item(&path) {
                report
                    .errors
                    .push(format!("{}: {error}", name.to_string_lossy()));
                continue;
            }
            report.removed += 1;
            let mut record = name.clone();
            record.push(".trashinfo");
            if let Err(error) = fs::remove_file(infos.join(record))
                && error.kind() != io::ErrorKind::NotFound
            {
                report
                    .errors
                    .push(format!("registro de {}: {error}", name.to_string_lossy()));
            }
        }
        Ok(report)
    }
}

fn safe_directory(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pasta da lixeira inválida",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn remove_item(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.file_type().is_dir() {
        return fs::remove_dir_all(path);
    }
    fs::remove_file(path)
}

impl HomeTrash {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn from_env() -> Option<Self> {
        let data = env::var("XDG_DATA_HOME").ok().or_else(|| {
            env::var("HOME")
                .ok()
                .map(|home| format!("{home}/.local/share"))
        })?;
        Some(Self::new(Path::new(&data).join("Trash")))
    }

    pub fn put(&self, path: &Path) -> io::Result<()> {
        let files = self.root.join("files");
        let infos = self.root.join("info");
        fs::create_dir_all(&files)?;
        fs::create_dir_all(&infos)?;
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::other("caminho sem nome"))?
            .to_string_lossy();
        let free = free_name(&name, |candidate| {
            files.join(candidate).symlink_metadata().is_ok()
                || infos.join(format!("{candidate}.trashinfo")).exists()
        });
        let info_path = infos.join(format!("{free}.trashinfo"));
        let absolute = std::path::absolute(path)?;
        let mut record = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&info_path)?;
        record.write_all(info(&absolute, &chrono::Local::now().naive_local()).as_bytes())?;
        fs::rename(path, files.join(&free)).or_else(|error| {
            fs::remove_file(&info_path)?;
            Err(explained(error))
        })
    }
}

fn explained(error: io::Error) -> io::Error {
    match error.kind() {
        io::ErrorKind::CrossesDevices => io::Error::other("a lixeira fica em outro disco"),
        _ => error,
    }
}
