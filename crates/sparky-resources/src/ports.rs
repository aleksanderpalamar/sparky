use std::io;
use std::path::Path;

pub trait Files {
    fn read(&self, path: &Path) -> io::Result<String>;
    fn entries(&self, directory: &Path) -> io::Result<Vec<String>>;
}
