mod detail;
mod explore;
mod icon;
mod list;
mod navigation;
mod styles;
mod updates;

#[cfg(test)]
mod render_tests;

use iced::widget::{column, container, text};
use iced::{Element, Length};

use crate::app::{Page, Store};

pub fn view(store: &Store) -> Element<'_, crate::app::Message> {
    let skin = store.skin;
    let content = if store.selected_app().is_some() {
        detail::view(store)
    } else {
        match store.page {
            Page::Browse => explore::view(store),
            Page::Search => list::search(store),
            Page::Installed => list::installed(store),
            Page::Updates => updates::view(store),
        }
    };
    let mut page = column![navigation::view(store), content]
        .spacing(0)
        .height(Length::Fill);
    if let Some(error) = &store.error {
        page = page.push(status(error, skin.urgent));
    }
    if let Some(notice) = &store.notice {
        page = page.push(status(notice, skin.accent));
    }
    container(page)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::window(skin))
        .into()
}

fn status(message: &str, color: iced::Color) -> Element<'_, crate::app::Message> {
    container(text(message).size(12).color(color))
        .width(Length::Fill)
        .padding([8, 24])
        .into()
}
