use std::fmt;

use crate::domain::apps::AppEntry;

pub trait SystemFiles: Send + Sync {
    fn read(&self, path: &str) -> Option<String>;
    fn list(&self, path: &str) -> Vec<String>;
    fn link(&self, path: &str) -> Option<String>;
}

pub trait Filesystems: Send + Sync {
    fn usage(&self, mount: &str) -> Option<(u64, u64)>;
}

pub trait Catalog: Send + Sync {
    fn apps(&self) -> Vec<AppEntry>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    Terminate,
    Kill,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalError {
    Denied,
    Gone,
    Failed(String),
}

impl fmt::Display for SignalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalError::Denied => formatter.write_str(&crate::i18n::text("stop-denied")),
            SignalError::Gone => formatter.write_str(&crate::i18n::text("process-gone")),
            SignalError::Failed(reason) => {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("reason", reason.as_str());
                formatter.write_str(&crate::i18n::message("stop-failed", Some(&args)))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    pub pid: u32,
    pub started: u64,
}

pub trait Claim: Send + Sync {
    fn send(&self, stop: Stop) -> Result<(), SignalError>;
}

pub trait Signals: Send + Sync {
    fn claim(&self, identity: Identity) -> Result<Box<dyn Claim>, SignalError>;
}
