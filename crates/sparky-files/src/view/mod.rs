pub mod batch_preview;
pub mod context;
pub mod delete_dialog;
pub mod grid;
pub mod header;
pub mod icon;
pub mod labels;
pub mod list;
pub mod open_with;
mod open_with_list;
pub mod properties;
pub mod search_status;
pub mod sidebar;
mod sidebar_bookmarks;
mod sidebar_device;
#[cfg(test)]
mod sidebar_tests;
pub mod status;
pub mod style;
pub mod tabs;
pub mod toast;
pub mod trash_sidebar;
pub mod virtual_page;

use iced::widget::{column, container, row};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::grid::ViewMode;
use crate::i18n;

pub fn view(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let content = match (files.sidebar_page, files.view_mode) {
        (Some(_), _) => virtual_page::view(files),
        (None, ViewMode::Grid) => grid::grid(files),
        (None, ViewMode::List) => list::list(files),
    };
    let main = match tabs::bar(files) {
        Some(bar) => column![bar, header::header(files), content],
        None => column![header::header(files), content],
    }
    .width(Length::Fill)
    .height(Length::Fill);
    let body = row![sidebar::sidebar(files), main].height(Length::Fill);
    let base = container(column![body, status::status(files)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::window(skin));
    let base = toast::layer(files, base.into());
    let base = delete_dialog::layer(files, base);
    context::layer(files, base)
}

pub fn title(files: &Files) -> String {
    if files.sidebar_page.is_some() {
        return i18n::with("window-title", "location", virtual_page::label(files));
    }
    if files.in_trash() {
        return i18n::text("window-title-trash");
    }
    i18n::with("window-title", "location", files.shown(&files.location))
}
