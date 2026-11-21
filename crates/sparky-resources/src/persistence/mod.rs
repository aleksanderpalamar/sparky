pub mod checkpoint;
#[cfg(test)]
mod checkpoint_tests;
#[cfg(test)]
pub(crate) mod fake_storage;
pub mod knowledge;
#[cfg(test)]
mod knowledge_tests;

pub use checkpoint::{Checkpoint, CheckpointError, Kind};
pub use knowledge::{Knowledge, Restored};
