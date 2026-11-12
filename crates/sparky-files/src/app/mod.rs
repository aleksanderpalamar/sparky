pub mod archive;
pub mod batch;
pub mod boot;
pub mod clipboard;
mod commands;
pub mod context;
mod details;
pub mod editing;
mod empty_trash;
pub mod events;
pub mod extraction;
mod job_report;
pub mod jobs;
pub mod keys;
pub mod message;
pub mod mode;
pub mod navigation;
pub mod presentation;
pub mod previews;
mod removal;
mod reveal;
pub mod search;
mod sidebar_navigation;
pub mod sorting;
pub mod state;
pub mod tabs;
pub mod transfer;
pub mod trash;
mod undo;
pub mod update;
mod virtual_commands;
pub mod volumes;

#[cfg(test)]
mod batch_tests;
#[cfg(test)]
mod clipboard_files_tests;
#[cfg(test)]
mod clipboard_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod details_tests;
#[cfg(test)]
mod disk_trash_tests;
#[cfg(test)]
mod empty_trash_tests;
#[cfg(test)]
mod extract_tests;
#[cfg(test)]
mod files_tests;
#[cfg(test)]
mod folder_change_tests;
#[cfg(test)]
mod jobs_tests;
#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod menu_actions_tests;
#[cfg(test)]
mod mtp_tests;
#[cfg(test)]
mod presentation_tests;
#[cfg(test)]
mod previews_stamp_tests;
#[cfg(test)]
mod previews_tests;
#[cfg(test)]
mod removal_tests;
#[cfg(test)]
mod reveal_tests;
#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod sidebar_tests;
#[cfg(test)]
mod sort_tests;
#[cfg(test)]
mod tabs_tests;
#[cfg(test)]
mod test_scene;
#[cfg(test)]
mod test_sources;
#[cfg(test)]
mod test_tasks;
#[cfg(test)]
mod trash_sidebar_tests;
#[cfg(test)]
mod trash_tests;
#[cfg(test)]
mod undo_tests;
#[cfg(test)]
mod volume_pending_tests;
#[cfg(test)]
mod volumes_tests;

use iced::Size;

use crate::app::state::Files;
use crate::view::{self, style};

pub fn run() -> iced::Result {
    iced::application(boot::boot, update::update, view::view)
        .title(view::title)
        .subscription(events::subscription)
        .default_font(sparky_ui::style::FONT)
        .theme(|files: &Files| files.skin.theme())
        .window(sparky_ui::window::settings(
            style::APPLICATION_ID,
            Size::new(style::WINDOW_WIDTH, style::WINDOW_HEIGHT),
        ))
        .run()
}
