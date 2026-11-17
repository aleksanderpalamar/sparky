use iced::alignment::Vertical;
use iced::widget::{Column, button, column, container, mouse_area, row, rule, scrollable, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, PADDING, SIDEBAR_WIDTH, SMALL_TEXT, SPACING};

use super::sidebar_device::{device, work_label};
use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::drop::DropTarget;
use crate::domain::icons::IconKind;
use crate::domain::sidebar_location::SidebarLocation;
use crate::i18n;
use crate::view::icon::sidebar_icon;

pub fn sidebar(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let places = ordered_places(&files.places);
    let home = places
        .first()
        .map(|(index, place)| place_button(files, *index, place));
    let folders = places
        .iter()
        .skip(1)
        .map(|(index, place)| place_button(files, *index, place));
    let devices = files
        .volumes
        .iter()
        .enumerate()
        .map(|(index, volume)| device(files, volume, index));
    let mut content = column![
        Column::with_children(home).spacing(3),
        virtual_button(
            files,
            SidebarLocation::Recent,
            IconKind::Recent,
            "place-recent"
        ),
        virtual_button(
            files,
            SidebarLocation::Favorites,
            IconKind::Favorite,
            "place-favorites"
        ),
        virtual_button(
            files,
            SidebarLocation::Network,
            IconKind::Network,
            "place-network"
        ),
        crate::view::trash_sidebar::buttons(files),
        rule::horizontal(1),
        Column::with_children(folders).spacing(3),
        rule::horizontal(1),
        super::sidebar_bookmarks::section(files),
        rule::horizontal(1),
        text(i18n::text("devices"))
            .size(SMALL_TEXT)
            .color(skin.bright_foreground),
        Column::with_children(devices).spacing(5)
    ]
    .spacing(SPACING);
    let progress = work_label(files.volume_work.as_ref());
    if let Some(label) = progress.clone() {
        content = content.push(text(label).size(SMALL_TEXT).color(skin.foreground));
    }
    if let Some(error) = &files.volume_error {
        content = content.push(text(error).size(SMALL_TEXT).color(skin.urgent));
    }
    if files.volumes.is_empty() && progress.is_none() && files.volume_error.is_none() {
        content = content.push(
            text(i18n::text("no-volumes"))
                .size(SMALL_TEXT)
                .color(skin.foreground),
        );
    }
    container(scrollable(content).height(Length::Fill))
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill)
        .padding(PADDING)
        .style(move |_| style::panel(skin))
        .into()
}

fn virtual_button<'a>(
    files: &'a Files,
    page: SidebarLocation,
    kind: IconKind,
    label: &'static str,
) -> Element<'a, Message> {
    let skin = files.skin;
    let content = row![
        sidebar_icon(files, kind, 20.0),
        text(i18n::text(label)).size(13.0)
    ]
    .spacing(6)
    .align_y(Vertical::Center);
    button(content)
        .width(Length::Fill)
        .on_press(Message::Sidebar(page))
        .style(move |_, status| {
            style::entry(
                skin,
                Highlight::when(files.sidebar_page == Some(page)),
                status,
            )
        })
        .into()
}

fn place_button<'a>(
    files: &'a Files,
    index: usize,
    place: &'a crate::domain::places::Place,
) -> Element<'a, Message> {
    let skin = files.skin;
    let label = row![
        sidebar_icon(files, place.icon, 20.0),
        text(i18n::text(place.key)).size(13.0).width(Length::Fill)
    ]
    .spacing(6)
    .align_y(Vertical::Center);
    let target = DropTarget::Place(index);
    let current = files.sidebar_page.is_none() && place.path == files.location;
    let highlight = Highlight::when(current || files.is_drop_target(target));
    let control = button(label)
        .width(Length::Fill)
        .on_press(Message::Place(index))
        .style(move |_, status| style::entry(skin, highlight, status));
    mouse_area(control)
        .on_enter(Message::Hover(target))
        .on_exit(Message::Unhover(target))
        .into()
}

pub(super) fn ordered_places(
    places: &[crate::domain::places::Place],
) -> Vec<(usize, &crate::domain::places::Place)> {
    const ORDER: [&str; 6] = [
        "place-home",
        "place-documents",
        "place-music",
        "place-pictures",
        "place-videos",
        "place-downloads",
    ];
    ORDER
        .iter()
        .filter_map(|key| {
            places
                .iter()
                .enumerate()
                .find(|(_, place)| place.key == *key)
        })
        .collect()
}
