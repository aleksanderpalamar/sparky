use std::fs::{self, DirBuilder, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::ports::store::VaultStore;

const PRIVATE_DIR: u32 = 0o700;
const PRIVATE_FILE: u32 = 0o600;

pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn from_env() -> Option<Self> {
        let data_home = std::env::var("XDG_DATA_HOME").ok();
        let home = std::env::var("HOME").unwrap_or_default();
        location(data_home.as_deref(), &home).map(Self::new)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn location(data_home: Option<&str>, home: &str) -> Option<PathBuf> {
    let base = match data_home.filter(|path| path.starts_with('/')) {
        Some(path) => PathBuf::from(path),
        None if home.starts_with('/') => Path::new(home).join(".local").join("share"),
        None => return None,
    };
    Some(base.join("sparky").join("keyrings").join("login.json"))
}

impl VaultStore for FileStore {
    fn load(&self) -> io::Result<Option<String>> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn save(&self, contents: &str) -> io::Result<()> {
        let folder = self.path.parent().ok_or_else(|| io::Error::other("o cofre precisa de uma pasta"))?;
        DirBuilder::new().recursive(true).mode(PRIVATE_DIR).create(folder)?;
        fs::set_permissions(folder, Permissions::from_mode(PRIVATE_DIR))?;
        let temporary = self.path.with_extension("json.tmp");
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(PRIVATE_FILE).open(&temporary)?;
        file.set_permissions(Permissions::from_mode(PRIVATE_FILE))?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        File::open(folder)?.sync_all()
    }
}
