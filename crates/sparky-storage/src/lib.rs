pub mod system;
mod listing;
mod volume;

pub use listing::{external_volumes, parse};
pub use volume::{Operation, Volume};

pub trait VolumeSource: Send + Sync {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String>;
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod system_tests;
