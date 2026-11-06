use fluent_bundle::FluentArgs;
use iced::alignment::Vertical;
use iced::widget::{button, row, text, tooltip};
use iced::{Element, Length};
use sparky_storage::{Operation, Volume, VolumeKind};
use sparky_ui::style::{self, Highlight, SMALL_TEXT, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::icons::IconKind;
use crate::i18n;
use crate::view::icon::sidebar_icon;

pub(super) fn device<'a>(
    files: &'a Files,
    volume: &'a Volume,
    index: usize,
) -> Element<'a, Message> {
    let skin = files.skin;
    let kind = match volume.kind() {
        VolumeKind::Mtp => IconKind::Phone,
        VolumeKind::Block if volume.removable => IconKind::Removable,
        VolumeKind::Block => IconKind::Drive,
    };
    let label = text(display_name(volume)).size(TEXT_SIZE);
    let selected = files.sidebar_page.is_none()
        && volume
            .mountpoint
            .as_ref()
            .is_some_and(|point| files.location.starts_with(point));
    let highlight = Highlight::when(selected);
    let body = row![sidebar_icon(files, kind, 22.0), label]
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
    if let Some(action) = device_action(volume) {
        let icon = sidebar_icon(files, IconKind::Eject, 18.0);
        let message = match action {
            DeviceAction::Eject => Message::EjectVolume(volume.path.clone()),
            DeviceAction::Unmount => Message::UnmountVolume(volume.path.clone()),
        };
        let hint = match action {
            DeviceAction::Eject => "eject-hint",
            DeviceAction::Unmount => "unmount-hint",
        };
        let control = button(icon)
            .padding(5)
            .on_press_maybe((!busy).then_some(message))
            .style(move |_, status| style::entry(skin, Highlight::Plain, status));
        line = line.push(
            tooltip(
                control,
                text(i18n::text(hint)).size(SMALL_TEXT),
                tooltip::Position::Right,
            )
            .style(move |_| style::tooltip(skin)),
        );
    }
    line.width(Length::Fill).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeviceAction {
    Unmount,
    Eject,
}

pub(super) fn device_action(volume: &Volume) -> Option<DeviceAction> {
    if volume.mounted() {
        return Some(DeviceAction::Unmount);
    }
    if volume.can_eject() {
        return Some(DeviceAction::Eject);
    }
    None
}

pub(super) fn display_name(volume: &Volume) -> String {
    match volume.label.is_empty() {
        true if !volume.size.is_empty() => i18n::with("volume-of", "size", volume.size.clone()),
        true => volume.path.clone(),
        false => volume.label.clone(),
    }
}

pub(super) fn work_label(operation: Option<&Operation>) -> Option<String> {
    match operation {
        Some(Operation::Mount(_)) => Some(i18n::text("mounting")),
        Some(Operation::Unmount(_)) => Some(i18n::text("unmounting")),
        Some(Operation::Eject(_)) => Some(i18n::text("ejecting")),
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
