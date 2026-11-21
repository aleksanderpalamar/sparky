use std::fs;
use std::io;
use std::path::Path;

use crate::ports::Files;

pub struct LinuxFiles;

impl Files for LinuxFiles {
    fn read(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn entries(&self, directory: &Path) -> io::Result<Vec<String>> {
        fs::read_dir(directory)?
            .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
            .collect()
    }
}
