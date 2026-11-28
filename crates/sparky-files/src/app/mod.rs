pub mod archive;
mod background;
pub mod batch;
#[cfg(test)]
mod bookmark_keys_tests;
mod bookmark_menu;
#[cfg(test)]
mod bookmark_menu_tests;
mod bookmark_reorder;
pub mod bookmarks;
#[cfg(test)]
mod bookmarks_tests;
pub mod boot;
pub mod chooser;
pub mod clipboard;
mod commands;
pub mod context;
mod details;
pub mod drop;
pub mod editing;
mod empty_trash;
pub mod events;
pub mod extraction;
mod handoff;
mod history;
mod job_report;
pub mod jobs;
pub mod keys;
mod location;
pub mod message;
pub mod mode;
pub mod navigation;
pub mod open_with;
pub mod popups;
pub mod presentation;
pub mod previews;
mod removal;
mod reveal;
pub mod search;
mod services;
pub mod shell;
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
mod drop_tests;
#[cfg(test)]
mod empty_trash_tests;
#[cfg(test)]
mod extract_tests;
#[cfg(test)]
mod files_tests;
#[cfg(test)]
mod folder_change_tests;
#[cfg(test)]
mod handoff_tests;
#[cfg(test)]
mod history_tests;
#[cfg(test)]
mod jobs_tests;
#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod location_tests;
#[cfg(test)]
mod menu_actions_tests;
#[cfg(test)]
mod mtp_tests;
#[cfg(test)]
mod navigation_keys_tests;
#[cfg(test)]
mod open_with_tests;
#[cfg(test)]
mod presentation_tests;
#[cfg(test)]
mod previews_stamp_tests;
#[cfg(test)]
mod previews_tests;
#[cfg(test)]
mod removal_tests;
#[cfg(test)]
mod restore_toast_tests;
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

use iced::Task;
use iced_layershell::actions::IcedXdgWindowSettings;

use crate::app::message::Message;
use crate::app::shell::{Handler, Shell};
use crate::app::state::Files;
use crate::view::style;

pub fn run() -> Result<(), iced_layershell::Error> {
    serve(
        boot::boot,
        style::APPLICATION_ID,
        libsparky::desk::window(
            style::APPLICATION_ID,
            (style::WINDOW_WIDTH as u32, style::WINDOW_HEIGHT as u32),
            style::MIN_SIZE,
        ),
        update::update,
    )
}

pub fn serve(
    boot: impl Fn() -> (Files, Task<Message>) + 'static,
    namespace: &'static str,
    window: IcedXdgWindowSettings,
    handler: Handler,
) -> Result<(), iced_layershell::Error> {
    let start = move || {
        let (files, task) = boot();
        shell::open(files, task, window, handler)
    };
    iced_layershell::build_pattern::daemon(
        start,
        move || namespace.to_owned(),
        shell::update,
        shell::view,
    )
    .title(shell::title)
    .subscription(|shell: &Shell| events::subscription(&shell.files))
    .theme(shell::theme)
    .style(sparky_ui::surface::transparent)
    .settings(libsparky::desk::settings())
    .run()
}
