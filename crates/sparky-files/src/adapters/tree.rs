use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;

pub fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(from)?;
    if metadata.is_symlink() {
        return symlink(fs::read_link(from)?, to);
    }
    if !metadata.is_dir() {
        return fs::copy(from, to).map(|_| ());
    }
    fs::create_dir(to)?;
    for item in fs::read_dir(from)? {
        let item = item?;
        copy_tree(&item.path(), &to.join(item.file_name()))?;
    }
    Ok(())
}

pub fn relocate(from: &Path, to: &Path) -> io::Result<()> {
    match fs::rename(from, to) {
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
            copy_tree(from, to)?;
            remove_tree(from)
        }
        other => other,
    }
}

fn remove_tree(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path)?.is_dir() {
        true => fs::remove_dir_all(path),
        false => fs::remove_file(path),
    }
}
