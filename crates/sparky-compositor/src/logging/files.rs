use std::ffi::OsString;
use std::fs::{self, DirBuilder, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const BACKUPS: usize = 3;

pub fn state_directory(state: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    state
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            home.map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|path| path.join(".local/state"))
        })
        .map(|path| path.join("sparky/logs"))
}

pub struct LogFile {
    directory: PathBuf,
    file: File,
    _lock: File,
    size: u64,
    limit: u64,
}

impl LogFile {
    pub fn open(directory: &Path, limit: u64) -> io::Result<Self> {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)?;
        if !fs::symlink_metadata(directory)?.file_type().is_dir() {
            return Err(io::Error::other("diretório de logs inválido"));
        }
        fs::set_permissions(directory, Permissions::from_mode(0o700))?;
        let lock = private_file(&directory.join("compositor.lock"))?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let path = directory.join("compositor.log");
        let previous = private_file(&path)?;
        if previous.metadata()?.len() > 0 {
            rotate(directory)?;
        }
        let file = private_file(&path)?;
        Ok(Self {
            directory: directory.into(),
            file,
            _lock: lock,
            size: 0,
            limit,
        })
    }
}

impl Write for LogFile {
    fn write(&mut self, record: &[u8]) -> io::Result<usize> {
        if record.len() as u64 > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "registro maior que o limite do log",
            ));
        }
        if self.size + record.len() as u64 > self.limit {
            self.file.sync_data()?;
            rotate(&self.directory)?;
            self.file = private_file(&self.directory.join("compositor.log"))?;
            self.size = 0;
        }
        self.file.write_all(record)?;
        self.size += record.len() as u64;
        Ok(record.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.sync_data()
    }
}

fn private_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::other("log não é um arquivo regular"));
    }
    file.set_permissions(Permissions::from_mode(0o600))?;
    Ok(file)
}

fn rotate(directory: &Path) -> io::Result<()> {
    for index in (0..BACKUPS).rev() {
        let source = match index {
            0 => directory.join("compositor.log"),
            _ => directory.join(format!("compositor.log.{index}")),
        };
        let target = directory.join(format!("compositor.log.{}", index + 1));
        match fs::rename(source, target) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
