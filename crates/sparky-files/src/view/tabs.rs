use std::path::Path;

use iced::alignment::Vertical;
use iced::widget::{Row, button, container, mouse_area, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT};

use crate::app::message::Message;
use crate::app::search::Search;
use crate::app::state::Files;
use crate::domain::drop::DropTarget;
use crate::domain::sidebar_location::SidebarLocation;
use crate::i18n;
use crate::view::virtual_page::page_label;

pub fn titles(files: &Files) -> Vec<String> {
    (0..files.tabs.len())
        .map(|index| match files.tabs.stored(index) {
            Some(tab) => label(files, &tab.location, tab.sidebar_page, tab.search.as_ref()),
            None => label(
                files,
                &files.location,
                files.sidebar_page,
                files.search.as_ref(),
            ),
        })
        .collect()
}

pub fn bar(files: &Files) -> Option<Element<'_, Message>> {
    if files.tabs.len() < 2 {
        return None;
    }
    let skin = files.skin;
    let active = files.tabs.active();
    let tabs = titles(files).into_iter().enumerate().map(|(index, title)| {
        let target = DropTarget::Tab(index);
        let highlight = Highlight::when(index == active || files.is_drop_target(target));
        let name = mouse_area(
            button(text(title).size(SMALL_TEXT))
                .padding([6, 8])
                .width(Length::Fill)
                .on_press(Message::Tab(index))
                .style(move |_, status| style::entry(skin, highlight, status)),
        )
        .on_enter(Message::Hover(target))
        .on_exit(Message::Unhover(target));
        let close = button(text("×").size(SMALL_TEXT))
            .padding([6, 8])
            .on_press(Message::CloseTab(index))
            .style(move |_, status| style::entry(skin, highlight, status));
        row![name, close]
            .align_y(Vertical::Center)
            .width(Length::FillPortion(1))
            .into()
    });
    Some(
        container(Row::with_children(tabs).spacing(4))
            .padding([4.0, PADDING])
            .width(Length::Fill)
            .into(),
    )
}

fn label(
    files: &Files,
    location: &Path,
    page: Option<SidebarLocation>,
    search: Option<&Search>,
) -> String {
    if let Some(search) = search {
        return i18n::with("tab-search", "query", search.query.clone());
    }
    if let Some(page) = page {
        return page_label(page);
    }
    let trash = std::iter::once(&files.services.trash)
        .chain(&files.other_trashes)
        .any(|trash| trash.directory() == location);
    if trash {
        return i18n::text("trash");
    }
    location
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| location.display().to_string())
}
