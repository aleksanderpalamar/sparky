use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::adapters::home_trash::HomeTrash;
use crate::ports::{TrashDisks, TrashStore};

const MOUNTS: &str = "/proc/self/mounts";
const SKIPPED: [&str; 14] = [
    "proc",
    "sysfs",
    "devtmpfs",
    "devpts",
    "tmpfs",
    "cgroup",
    "cgroup2",
    "securityfs",
    "debugfs",
    "nfs",
    "nfs4",
    "cifs",
    "smb3",
    "fuse.sshfs",
];

#[derive(Clone)]
pub struct DiskTrashes {
    home: HomeTrash,
    uid: u32,
}

impl DiskTrashes {
    pub fn new(home: HomeTrash) -> Self {
        Self {
            home,
            uid: rustix::process::getuid().as_raw(),
        }
    }

    pub fn for_item(&self, path: &Path) -> io::Result<HomeTrash> {
        let item = fs::symlink_metadata(path)?.dev();
        if device(self.home.root()) == Some(item) {
            return Ok(self.home.clone());
        }
        let topdir = mount_root(path, |candidate| {
            fs::symlink_metadata(candidate).ok().map(|meta| meta.dev())
        });
        let root = prepare(&topdir, self.uid)?;
        Ok(HomeTrash::on_disk(root, topdir))
    }

    fn candidates(&self, topdir: &Path) -> [PathBuf; 2] {
        [
            topdir.join(".Trash").join(self.uid.to_string()),
            topdir.join(format!(".Trash-{}", self.uid)),
        ]
    }
}

impl TrashDisks for DiskTrashes {
    fn others(&self) -> Vec<Arc<dyn TrashStore>> {
        let mounts = fs::read_to_string(MOUNTS).unwrap_or_default();
        mount_points(&mounts)
            .into_iter()
            .flat_map(|topdir| self.candidates(&topdir).map(|root| (root, topdir.clone())))
            .filter(|(root, _)| root != self.home.root() && has_items(&root.join("files")))
            .map(|(root, topdir)| Arc::new(HomeTrash::on_disk(root, topdir)) as Arc<dyn TrashStore>)
            .collect()
    }
}

pub fn mount_root(path: &Path, device: impl Fn(&Path) -> Option<u64>) -> PathBuf {
    let own = device(path);
    let mut current = path.to_path_buf();
    while let Some(parent) = current.parent().filter(|parent| device(parent) == own) {
        current = parent.to_path_buf();
    }
    current
}

pub fn prepare(topdir: &Path, uid: u32) -> io::Result<PathBuf> {
    let shared = topdir.join(".Trash");
    let sticky = fs::symlink_metadata(&shared)
        .is_ok_and(|meta| meta.file_type().is_dir() && meta.mode() & 0o1000 != 0);
    if sticky {
        let root = shared.join(uid.to_string());
        if private(&root, uid).is_ok() {
            return Ok(root);
        }
    }
    let root = topdir.join(format!(".Trash-{uid}"));
    private(&root, uid).map(|()| root)
}

pub fn mount_points(mounts: &str) -> Vec<PathBuf> {
    mounts
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(' ');
            let point = fields.nth(1)?;
            let kind = fields.next()?;
            (!SKIPPED.contains(&kind)).then(|| PathBuf::from(unescape(point)))
        })
        .collect()
}

fn private(root: &Path, uid: u32) -> io::Result<()> {
    match fs::symlink_metadata(root) {
        Ok(meta) if meta.file_type().is_dir() && meta.uid() == uid => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            crate::i18n::text("trash-invalid-directory"),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            DirBuilder::new().mode(0o700).create(root)
        }
        Err(error) => Err(error),
    }
}

fn device(path: &Path) -> Option<u64> {
    path.ancestors()
        .find_map(|ancestor| fs::metadata(ancestor).ok())
        .map(|meta| meta.dev())
}

fn has_items(files: &Path) -> bool {
    fs::read_dir(files).is_ok_and(|mut items| items.next().is_some())
}

fn unescape(field: &str) -> String {
    let mut text = String::new();
    let mut rest = field;
    while let Some(index) = rest.find('\\') {
        text.push_str(&rest[..index]);
        let code = rest
            .get(index + 1..index + 4)
            .and_then(|code| u8::from_str_radix(code, 8).ok());
        match code {
            Some(byte) => {
                text.push(char::from(byte));
                rest = &rest[index + 4..];
            }
            None => {
                text.push('\\');
                rest = &rest[index + 1..];
            }
        }
    }
    text.push_str(rest);
    text
}
