pub mod archive;
pub mod clipboard;
pub mod home_trash;
pub mod icons;
pub mod local_disk;
mod sidebar_network;
mod sidebar_recent;
pub mod sidebar_source;
#[cfg(test)]
mod sidebar_source_tests;
pub mod thumbnail;
pub mod tree;
pub mod xdg_open;

#[cfg(test)]
mod home_trash_tests;
#[cfg(test)]
mod local_disk_tests;
