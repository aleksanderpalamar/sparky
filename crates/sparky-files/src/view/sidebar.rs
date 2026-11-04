use fluent_bundle::FluentArgs;
use iced::alignment::Vertical;
use iced::widget::{Column, button, column, container, row, rule, scrollable, text, tooltip};
use iced::{Element, Length};
use sparky_storage::{Operation, Volume, VolumeKind};
use sparky_ui::style::{self, Highlight, PADDING, SIDEBAR_WIDTH, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::icons::IconKind;
use crate::i18n;
use crate::view::icon::icon;

pub fn sidebar(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let places = files.places.iter().enumerate().map(|(index, place)| {
        let label = row![
            text((index + 1).to_string())
                .size(SMALL_TEXT)
                .color(skin.accent)
                .width(14),
            icon(files, place.icon, 20.0),
            text(i18n::text(place.key)).size(13.0)
        ]
        .spacing(6)
        .align_y(Vertical::Center);
        let highlight = Highlight::when(place.path == files.location);
        button(label)
            .width(Length::Fill)
            .on_press(Message::Place(index))
            .style(move |_, status| style::entry(skin, highlight, status))
            .into()
    });
    let devices = files
        .volumes
        .iter()
        .enumerate()
        .map(|(index, volume)| device(files, volume, index));
    let mut content = column![
        text(i18n::text("places"))
            .size(SMALL_TEXT)
            .color(skin.bright_foreground),
        Column::with_children(places).spacing(3),
        crate::view::trash_sidebar::button(files),
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

fn device<'a>(files: &'a Files, volume: &'a Volume, index: usize) -> Element<'a, Message> {
    let skin = files.skin;
    let kind = match volume.kind() {
        VolumeKind::Mtp => IconKind::Phone,
        VolumeKind::Block if volume.removable => IconKind::Removable,
        VolumeKind::Block => IconKind::Drive,
    };
    let name = match volume.label.is_empty() {
        true if !volume.size.is_empty() => i18n::with("volume-of", "size", volume.size.clone()),
        true => volume.path.clone(),
        false => volume.label.clone(),
    };
    let label = column![
        text(name).size(TEXT_SIZE),
        text(device_detail(volume))
            .size(SMALL_TEXT)
            .color(skin.foreground)
    ]
    .spacing(2);
    let selected = volume
        .mountpoint
        .as_ref()
        .is_some_and(|point| files.location.starts_with(point));
    let highlight = Highlight::when(selected);
    let body = row![icon(files, kind, 22.0), label]
        .spacing(6)
        .align_y(Vertical::Center);
    let busy = files.volume_work.is_some();
    let open = button(body)
        .width(Length::Fill)
        .on_press_maybe((!busy).then(|| Message::OpenVolume(volume.path.clone())))
        .style(move |_, status| style::entry(skin, highlight, status));
    let help = volume_help(volume, index);
    let open = tooltip(open, text(help).size(SMALL_TEXT), tooltip::Position::Right)
        .style(move |_| style::tooltip(skin));
    let mut line = row![open].align_y(Vertical::Center).spacing(2);
    if volume.mounted() {
        let eject = button(icon(files, IconKind::Eject, 18.0))
            .padding(5)
            .on_press_maybe((!busy).then(|| Message::UnmountVolume(volume.path.clone())))
            .style(move |_, status| style::entry(skin, Highlight::Plain, status));
        line = line.push(
            tooltip(
                eject,
                text(i18n::text("unmount-hint")).size(SMALL_TEXT),
                tooltip::Position::Right,
            )
            .style(move |_| style::tooltip(skin)),
        );
    }
    line.width(Length::Fill).into()
}

pub(super) fn device_detail(volume: &Volume) -> String {
    let state = match volume.mounted() {
        true => "mounted",
        false => "state-not-mounted",
    };
    let summary = match volume.kind() {
        VolumeKind::Mtp => "MTP",
        VolumeKind::Block => &volume.size,
    };
    match summary.is_empty() {
        true => i18n::text(state),
        false => format!("{summary} · {}", i18n::text(state)),
    }
}

pub(super) fn work_label(operation: Option<&Operation>) -> Option<String> {
    match operation {
        Some(Operation::Mount(_)) => Some(i18n::text("mounting")),
        Some(Operation::Unmount(_)) => Some(i18n::text("unmounting")),
        Some(Operation::List) | None => None,
    }
}

fn volume_help(volume: &Volume, index: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("path", volume.path.clone());
    args.set("filesystem", volume.filesystem.clone());
    args.set(
        "mountpoint",
        volume
            .mountpoint
            .clone()
            .unwrap_or_else(|| i18n::text("not-mounted")),
    );
    args.set("index", (index + 1).to_string());
    i18n::message("volume-hint", Some(&args))
}
