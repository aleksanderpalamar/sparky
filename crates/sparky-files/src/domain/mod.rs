pub mod application;
pub mod archive;
pub mod batch_name;
#[cfg(test)]
mod batch_name_tests;
pub mod clip;
pub mod clip_payload;
#[cfg(test)]
mod clip_payload_tests;
pub mod context;
pub mod debounce;
pub mod destination;
pub mod drop;
pub mod entry;
pub mod grid;
pub mod icons;
pub mod image;
pub mod launch;
pub mod listing;
pub mod naming;
pub mod order;
pub mod places;
pub mod properties;
pub mod removal;
pub mod search;
pub mod sidebar_location;
pub mod size;
pub mod tabs;
#[cfg(test)]
mod tabs_tests;
pub mod thumbnail;
pub mod thumbnail_key;
pub mod transfer;
pub mod trash_info;
pub mod undo;

#[cfg(test)]
mod thumbnail_tests;
pub mod wording;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod grid_tests;
#[cfg(test)]
mod icons_tests;
#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod order_tests;
