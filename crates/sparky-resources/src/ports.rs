use std::io;
use std::path::Path;

pub trait Files {
    fn read(&self, path: &Path) -> io::Result<String>;
    fn entries(&self, directory: &Path) -> io::Result<Vec<String>>;
}

pub trait Storage {
    fn load(&self, name: &str) -> io::Result<Vec<u8>>;
    fn save(&self, name: &str, bytes: &[u8]) -> io::Result<()>;
    fn rename(&self, from: &str, to: &str) -> io::Result<()>;
    fn remove(&self, name: &str) -> io::Result<()>;
}
