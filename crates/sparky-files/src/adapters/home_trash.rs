use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::domain::naming::free_name;
use crate::domain::trash_info::info;

pub struct HomeTrash {
    root: PathBuf,
}

impl HomeTrash {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn from_env() -> Option<Self> {
        let data = env::var("XDG_DATA_HOME")
            .ok()
            .or_else(|| env::var("HOME").ok().map(|home| format!("{home}/.local/share")))?;
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
            files.join(candidate).symlink_metadata().is_ok() || infos.join(format!("{candidate}.trashinfo")).exists()
        });
        let info_path = infos.join(format!("{free}.trashinfo"));
        let absolute = std::path::absolute(path)?;
        let mut record = OpenOptions::new().write(true).create_new(true).open(&info_path)?;
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
