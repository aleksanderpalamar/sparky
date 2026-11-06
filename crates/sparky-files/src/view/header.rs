use iced::alignment::Vertical;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Element, Length};

use crate::app::message::{Message, TrashEvent};
use crate::app::state::Files;
use crate::domain::grid::ViewMode;
use crate::domain::listing::Hidden;
use crate::i18n;
use sparky_ui::style::{self, Highlight};
use sparky_ui::style::{HEADER_HEIGHT, PADDING, SMALL_TEXT, SPACING, TITLE_SIZE};

pub fn header(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let label = if files.sidebar_page.is_some() {
        crate::view::virtual_page::label(files)
    } else if files.in_trash() {
        i18n::text("trash")
    } else {
        files.shown(&files.location)
    };
    let path = text(label)
        .size(TITLE_SIZE)
        .color(skin.bright_foreground)
        .width(Length::Fill);
    let tags = tags(files)
        .into_iter()
        .map(|tag| text(tag).size(SMALL_TEXT).color(skin.cyan).into());
    let modes = row![
        view_button(
            files,
            ViewMode::Grid,
            i18n::text("grid"),
            i18n::text("grid-hint")
        ),
        view_button(
            files,
            ViewMode::List,
            i18n::text("list"),
            i18n::text("list-hint")
        )
    ]
    .spacing(4);
    let mut heading = row![path].extend(tags);
    if files.in_trash() {
        heading = heading.push(trash_button(
            files,
            i18n::text("restore-item"),
            Message::Trash(TrashEvent::RestoreSelected),
            files.listing.selected().is_some() && !files.emptying,
        ));
        heading = heading.push(trash_button(
            files,
            i18n::text("menu-restore-all"),
            Message::Trash(TrashEvent::RestoreAll),
            files.listing.total_len() > 0 && !files.emptying,
        ));
        heading = heading.push(trash_button(
            files,
            i18n::text("empty-trash"),
            Message::Trash(TrashEvent::Empty),
            files.listing.total_len() > 0 && !files.emptying,
        ));
    }
    if files.sidebar_page.is_none() && !files.in_trash() && files.listing.selected().is_some() {
        heading = heading.push(trash_button(
            files,
            i18n::text("favorite-action"),
            Message::FavoriteSelected,
            true,
        ));
    }
    if files.sidebar_page.is_none() {
        heading = heading.push(modes);
    }
    container(heading.spacing(SPACING * 2.0).align_y(Vertical::Center))
        .height(HEADER_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .into()
}

fn trash_button<'a>(
    files: &'a Files,
    label: String,
    message: Message,
    enabled: bool,
) -> Element<'a, Message> {
    let skin = files.skin;
    button(text(label).size(SMALL_TEXT))
        .padding([6, 8])
        .on_press_maybe(enabled.then_some(message))
        .style(move |_, status| style::entry(skin, Highlight::Plain, status))
        .into()
}

fn view_button<'a>(
    files: &'a Files,
    mode: ViewMode,
    label: String,
    help: String,
) -> Element<'a, Message> {
    let skin = files.skin;
    let highlight = Highlight::when(files.view_mode == mode);
    let control = button(text(label).size(SMALL_TEXT))
        .padding([6, 8])
        .on_press(Message::View(mode))
        .style(move |_, status| style::entry(skin, highlight, status));
    tooltip(
        control,
        text(help).size(SMALL_TEXT),
        tooltip::Position::Bottom,
    )
    .style(move |_| style::tooltip(skin))
    .into()
}

fn tags(files: &Files) -> Vec<String> {
    let filter = files.listing.filter();
    let mut tags = Vec::new();
    if !filter.is_empty() {
        tags.push(i18n::with("filter-tag", "filter", filter.to_owned()));
    }
    if files.listing.hidden() == Hidden::Shown {
        tags.push(i18n::text("hidden-visible"));
    }
    tags
}
