pub mod desktop;
mod listing;
pub mod mtp;
pub mod system;
mod volume;

pub use listing::{external_volumes, parse};
pub use volume::{Operation, Volume, VolumeKind};

pub trait VolumeSource: Send + Sync {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String>;
}

#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
