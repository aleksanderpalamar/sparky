pub mod desktop_entry;
pub mod icons;
pub mod programs;
pub mod tilde;
pub mod user_dirs;

#[cfg(test)]
mod desktop_entry_tests;
#[cfg(test)]
mod icons_tests;

pub use tilde::tilde;
pub use user_dirs::{UserDir, UserDirs};
