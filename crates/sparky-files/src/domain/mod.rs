pub mod application;
pub mod clip;
pub mod clip_payload;
#[cfg(test)]
mod clip_payload_tests;
pub mod context;
pub mod debounce;
pub mod destination;
pub mod entry;
pub mod grid;
pub mod icons;
pub mod image;
pub mod listing;
pub mod naming;
pub mod places;
pub mod properties;
pub mod removal;
pub mod sidebar_location;
pub mod size;
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
