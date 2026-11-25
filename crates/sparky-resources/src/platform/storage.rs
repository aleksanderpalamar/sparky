use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::persistence::checkpoint::LIMIT;
use crate::ports::Storage;

pub struct DiskStorage {
    directory: PathBuf,
}

impl DiskStorage {
    pub fn open(directory: PathBuf) -> io::Result<Self> {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)?;
        Ok(Self { directory })
    }

    pub fn from_env() -> Option<PathBuf> {
        let state = std::env::var("XDG_STATE_HOME")
            .ok()
            .filter(|path| path.starts_with('/'));
        let base = state.map(PathBuf::from).or_else(|| {
            let home = std::env::var("HOME")
                .ok()
                .filter(|path| path.starts_with('/'))?;
            Some(Path::new(&home).join(".local/state"))
        })?;
        Some(base.join("sparky/resources"))
    }

    fn settle(&self) -> io::Result<()> {
        File::open(&self.directory)?.sync_all()
    }

    fn path(&self, name: &str) -> io::Result<PathBuf> {
        let valid = !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.');
        match valid && !name.starts_with('.') {
            true => Ok(self.directory.join(name)),
            false => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }
}

impl Storage for DiskStorage {
    fn load(&self, name: &str) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(self.path(name)?)?
            .take(LIMIT as u64 + 1)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn save(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let target = self.path(name)?;
        let temporary = self.path(&format!("{name}.new"))?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, &target)?;
        self.settle()
    }

    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        fs::rename(self.path(from)?, self.path(to)?)?;
        self.settle()
    }

    fn remove(&self, name: &str) -> io::Result<()> {
        fs::remove_file(self.path(name)?)
    }
}
