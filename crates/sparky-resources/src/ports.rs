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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    Gone,
    Denied,
    Unsupported,
    Failed,
}

pub trait Executor {
    fn tasks(&self, target: crate::domain::Identity) -> Result<Vec<u32>, ExecError>;
    fn read(
        &self,
        task: u32,
        kind: crate::domain::ChangeKind,
    ) -> Result<crate::domain::Change, ExecError>;
    fn write(&self, task: u32, change: &crate::domain::Change) -> Result<(), ExecError>;
}
