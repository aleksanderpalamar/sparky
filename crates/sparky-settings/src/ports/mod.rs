pub mod commands;
#[cfg(test)]
pub mod fake;

use std::fmt;
use std::io;
use std::path::PathBuf;

use sparky_ipc::OutputInfo;

pub trait Store {
    fn load(&self) -> io::Result<Option<String>>;

    fn save(&self, contents: &str) -> io::Result<()>;

    fn location(&self) -> PathBuf;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositorError {
    Unreachable(String),
    Refused(String),
}

impl fmt::Display for CompositorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositorError::Unreachable(reason) => write!(formatter, "o sparky não respondeu ({reason})"),
            CompositorError::Refused(reason) => write!(formatter, "o sparky recusou: {reason}"),
        }
    }
}

pub trait Compositor {
    fn reload(&self) -> Result<(), CompositorError>;

    fn outputs(&self) -> Result<Vec<OutputInfo>, CompositorError>;
}
