#[cfg(test)]
pub(crate) mod fake_executor;
pub mod journal;
#[cfg(test)]
mod journal_tests;

pub use journal::{Applied, Failure, Outcome, apply, restore};
