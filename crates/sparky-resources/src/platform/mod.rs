pub mod elevator;
#[cfg(test)]
mod elevator_tests;
pub mod executor;
#[cfg(test)]
mod executor_tests;
pub mod files;
#[cfg(test)]
mod files_tests;
pub mod storage;
#[cfg(test)]
mod storage_tests;
mod syscalls;

pub use elevator::Pkexec;
pub use executor::LinuxExecutor;
pub use files::LinuxFiles;
pub use storage::DiskStorage;
pub use syscalls::{LinuxPriority, clock_ticks};

pub type SystemExecutor = LinuxExecutor<LinuxFiles, Option<Pkexec<LinuxFiles>>>;
