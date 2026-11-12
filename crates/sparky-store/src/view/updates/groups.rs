use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::domain::updates::{UpdateGroup, UpdateItem, UpdateKind, UpdateSource, UpdateTarget};
use crate::domain::{Installation, Manager};
use crate::i18n;

use super::super::{navigation, styles};

pub fn view<'a>(store: &'a Store, group: &'a UpdateGroup) -> Element<'a, Message> {
    let skin = store.skin;
    let mut heading = row![
        text(source_label(group.source))
            .size(16)
            .width(Length::Fill),
        text(group.items.len().to_string())
            .size(12)
            .color(skin.dark_foreground),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    if !group.items.is_empty() {
        heading = heading.push(navigation::control(
            store,
            &i18n::text("update-group"),
            Message::AskUpdate(group_target(group.source)),
        ));
    }
    let mut entries = column![heading].spacing(8);
    if group.source == UpdateSource::Native(Manager::Apt) {
        entries = entries.push(
            text(i18n::text("apt-index-note"))
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    if let Some(error) = &group.error {
        entries = entries.push(text(error).size(12).color(skin.urgent));
    }
    for item in &group.items {
        entries = entries.push(item_view(store, item));
    }
    container(entries)
        .padding(18)
        .width(Length::Fill)
        .style(move |_| styles::surface(skin))
        .into()
}

fn item_view<'a>(store: &'a Store, item: &'a UpdateItem) -> Element<'a, Message> {
    let skin = store.skin;
    let mut entry = row![
        column![
            text(&item.name).size(14),
            text(version_change(item))
                .size(11)
                .color(skin.dark_foreground),
            text(kind_label(item.kind))
                .size(11)
                .color(skin.dark_foreground),
        ]
        .spacing(3)
        .width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    if item.source != UpdateSource::Native(Manager::Pacman) {
        entry = entry.push(navigation::control(
            store,
            &i18n::text("update"),
            Message::AskUpdate(item.target()),
        ));
    }
    container(entry)
        .padding(12)
        .width(Length::Fill)
        .style(move |_| styles::surface(skin))
        .into()
}

fn group_target(source: UpdateSource) -> UpdateTarget {
    match source {
        UpdateSource::Flatpak(scope) => UpdateTarget::Flatpak {
            scope,
            reference: None,
        },
        UpdateSource::Native(manager) => UpdateTarget::Native {
            manager,
            package: None,
        },
    }
}

pub(super) fn source_label(source: UpdateSource) -> String {
    match source {
        UpdateSource::Flatpak(Installation::User) => i18n::text("flatpak-user"),
        UpdateSource::Flatpak(Installation::System) => i18n::text("flatpak-system"),
        UpdateSource::Native(manager) => {
            format!("{} · {}", i18n::text("system-updates"), manager.label())
        }
    }
}

pub(super) fn target_label(target: &UpdateTarget) -> String {
    match target {
        UpdateTarget::Flatpak { scope, .. } => source_label(UpdateSource::Flatpak(*scope)),
        UpdateTarget::Native { manager, .. } => source_label(UpdateSource::Native(*manager)),
    }
}

fn kind_label(kind: UpdateKind) -> String {
    i18n::text(match kind {
        UpdateKind::Application => "flatpak-application",
        UpdateKind::Runtime => "flatpak-runtime",
        UpdateKind::System => "system-package",
    })
}

fn version_change(item: &UpdateItem) -> String {
    let current = item.installed.as_deref().unwrap_or("?");
    let available = item.available.as_deref().unwrap_or("?");
    format!("{current} → {available}")
}
