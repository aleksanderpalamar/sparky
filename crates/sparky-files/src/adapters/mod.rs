pub mod app_icons;
pub mod archive;
pub mod bookmark_file;
#[cfg(test)]
mod bookmark_file_tests;
pub mod clipboard;
pub mod clipboard_request;
pub mod clipboard_server;
#[cfg(test)]
mod clipboard_tests;
pub mod compositor_drag;
pub mod disk_trash;
#[cfg(test)]
mod disk_trash_tests;
pub mod extract;
#[cfg(test)]
mod extract_tests;
pub mod file_manager1;
#[cfg(test)]
mod file_manager1_tests;
pub mod finder;
#[cfg(test)]
mod finder_tests;
pub mod gio_listing;
#[cfg(test)]
mod gio_listing_tests;
pub mod home_trash;
pub mod icons;
pub mod local_disk;
mod sidebar_network;
mod sidebar_recent;
pub mod sidebar_source;
#[cfg(test)]
mod sidebar_source_tests;
pub mod staged;
#[cfg(test)]
mod staged_tests;
pub mod thumbnail;
pub mod thumbnail_cache;
#[cfg(test)]
mod thumbnail_cache_tests;
pub mod thumbnail_png;
pub mod transfer;
#[cfg(test)]
mod transfer_tests;
pub mod tree;
pub mod watch;
#[cfg(test)]
mod watch_tests;
pub mod xdg_open;

#[cfg(test)]
mod home_trash_tests;
#[cfg(test)]
mod local_disk_tests;
