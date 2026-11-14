use iced::alignment::Vertical;
use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Background, Element, Length, Padding};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, SMALL_TEXT};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::bookmarks::Bookmark;
use crate::domain::drop::DropTarget;
use crate::domain::icons::IconKind;
use crate::i18n;
use crate::view::icon::tinted_sidebar_icon;
use crate::view::tag_palette::tint;

const ROW_PADDING: Padding = Padding {
    top: 5.0,
    right: 10.0,
    bottom: 5.0,
    left: 10.0,
};

pub fn section(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let header = text(i18n::text("bookmarks"))
        .size(SMALL_TEXT)
        .color(skin.bright_foreground)
        .width(Length::Fill);
    let rows: Vec<_> = files
        .bookmarks
        .items
        .local()
        .map(|(index, bookmark)| bookmark_row(files, index, bookmark))
        .collect();
    let body = match rows.is_empty() {
        true => placeholder(files),
        false => Column::with_children(rows).spacing(3).into(),
    };
    column![target(header.into(), DropTarget::Bookmarks), body]
        .spacing(3)
        .into()
}

fn placeholder(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let highlight = Highlight::when(files.is_drop_target(DropTarget::Bookmarks));
    let hint = container(
        text(i18n::text("bookmarks-empty"))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    )
    .width(Length::Fill)
    .padding(ROW_PADDING)
    .style(move |_| style::row(skin, highlight));
    target(hint.into(), DropTarget::Bookmarks)
}

fn bookmark_row<'a>(
    files: &'a Files,
    index: usize,
    bookmark: &'a Bookmark,
) -> Element<'a, Message> {
    let skin = files.skin;
    let current = files.sidebar_page.is_none() && bookmark.path() == Some(files.location.as_path());
    let selected = current || files.is_bookmark_target(index);
    let hovered = files.bookmarks.hovered == Some(index);
    let label = row![
        tinted_sidebar_icon(files, IconKind::Folder, 20.0, bookmark.color.map(tint)),
        text(bookmark.name()).size(13.0).width(Length::Fill)
    ]
    .spacing(6)
    .align_y(Vertical::Center);
    let body = container(label)
        .width(Length::Fill)
        .padding(ROW_PADDING)
        .style(move |_| look(skin, selected, hovered));
    mouse_area(body)
        .on_press(Message::BookmarkPressed(index))
        .on_right_press(Message::ContextBookmark(index))
        .on_enter(Message::Hover(DropTarget::Bookmark(index)))
        .on_exit(Message::Unhover(DropTarget::Bookmark(index)))
        .into()
}

fn target(content: Element<'_, Message>, target: DropTarget) -> Element<'_, Message> {
    mouse_area(content)
        .on_enter(Message::Hover(target))
        .on_exit(Message::Unhover(target))
        .into()
}

fn look(skin: Skin, selected: bool, hovered: bool) -> container::Style {
    match (selected, hovered) {
        (true, _) => container::Style {
            text_color: Some(skin.foreground),
            ..style::row(skin, Highlight::Selected)
        },
        (false, true) => container::Style {
            background: Some(Background::Color(skin.background)),
            text_color: Some(skin.foreground),
            ..container::Style::default()
        },
        (false, false) => container::Style {
            text_color: Some(skin.foreground),
            ..container::Style::default()
        },
    }
}
