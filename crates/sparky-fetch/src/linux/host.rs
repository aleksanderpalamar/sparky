use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub total: u64,
    pub free: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ran {
    Output(String),
    Missing,
    TimedOut,
    Failed(String),
}

pub trait Host: Send + Sync {
    fn read(&self, path: &Path) -> io::Result<String>;
    fn list(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn var(&self, name: &str) -> Option<String>;
    fn usage(&self, path: &Path) -> io::Result<Usage>;
    fn run(&self, program: &str, arguments: &[&str], timeout: Duration) -> Ran;
}
