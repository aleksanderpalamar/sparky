use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, PADDING, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::sidebar_location::SidebarLocation;
use crate::i18n;

pub fn label(files: &Files) -> String {
    match files.sidebar_page {
        Some(SidebarLocation::Recent) => i18n::text("place-recent"),
        Some(SidebarLocation::Favorites) => i18n::text("place-favorites"),
        Some(SidebarLocation::Network) => i18n::text("place-network"),
        None => String::new(),
    }
}

pub fn view(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    if files.network_pending.is_some() {
        return text(i18n::text("network-loading")).size(TEXT_SIZE).into();
    }
    if files.sidebar_items.is_empty() {
        return text(i18n::text("sidebar-empty")).size(TEXT_SIZE).into();
    }
    let items = files.sidebar_items.iter().enumerate().map(|(index, item)| {
        let open = button(text(&item.name).size(TEXT_SIZE).width(Length::Fill))
            .width(Length::Fill)
            .on_press(Message::SidebarItem(index))
            .style(move |_, status| {
                style::entry(skin, Highlight::when(files.sidebar_cursor == index), status)
            });
        let mut line = row![open].spacing(SPACING);
        if files.sidebar_page == Some(SidebarLocation::Favorites) {
            line = line.push(
                button(text(i18n::text("favorite-remove")))
                    .on_press(Message::RemoveFavorite(index))
                    .style(move |_, status| style::entry(skin, Highlight::Plain, status)),
            );
        }
        line.into()
    });
    scrollable(column(items).spacing(SPACING).padding(PADDING))
        .height(Length::Fill)
        .into()
}
