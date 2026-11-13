use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::symlink;
use std::path::Path;

use rustix::fs::copy_file_range;
use rustix::io::Errno;

const CHUNK: usize = 8 * 1024 * 1024;
const BUFFER: usize = 1024 * 1024;

pub fn copy_tree(from: &Path, to: &Path, advance: &mut dyn FnMut(u64) -> bool) -> io::Result<()> {
    let metadata = fs::symlink_metadata(from)?;
    if metadata.is_symlink() {
        return symlink(fs::read_link(from)?, to);
    }
    if !metadata.is_dir() {
        return copy_file(from, to, &metadata, advance);
    }
    fs::create_dir(to)?;
    for item in fs::read_dir(from)? {
        let item = item?;
        copy_tree(&item.path(), &to.join(item.file_name()), advance)?;
    }
    Ok(())
}

pub fn size(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    if !metadata.is_dir() {
        return metadata.len();
    }
    fs::read_dir(path)
        .map(|items| {
            items
                .filter_map(Result::ok)
                .map(|item| size(&item.path()))
                .sum()
        })
        .unwrap_or(0)
}

pub fn remove_tree(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path)?.is_dir() {
        true => fs::remove_dir_all(path),
        false => fs::remove_file(path),
    }
}

fn copy_file(
    from: &Path,
    to: &Path,
    metadata: &Metadata,
    advance: &mut dyn FnMut(u64) -> bool,
) -> io::Result<()> {
    let mut input = File::open(from)?;
    let mut output = OpenOptions::new().write(true).create_new(true).open(to)?;
    let mut buffer = Vec::new();
    loop {
        let copied = chunk(&mut input, &mut output, &mut buffer)?;
        if copied == 0 {
            break;
        }
        if !advance(copied) {
            return Err(io::ErrorKind::Interrupted.into());
        }
    }
    output.set_permissions(metadata.permissions())
}

fn chunk(input: &mut File, output: &mut File, buffer: &mut Vec<u8>) -> io::Result<u64> {
    if buffer.is_empty() {
        match copy_file_range(&*input, None, &*output, None, CHUNK) {
            Ok(copied) => return Ok(copied as u64),
            Err(Errno::XDEV | Errno::INVAL | Errno::NOSYS | Errno::OPNOTSUPP | Errno::PERM) => {
                buffer.resize(BUFFER, 0);
            }
            Err(error) => return Err(error.into()),
        }
    }
    let read = input.read(buffer)?;
    output.write_all(buffer.get(..read).unwrap_or_default())?;
    Ok(read as u64)
}
