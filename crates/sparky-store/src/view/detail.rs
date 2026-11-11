use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::app::{Message, Store};
use crate::domain::{Action, Manager, Source};
use crate::i18n;

use super::control;

pub fn view(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    let Some(app) = store.selected_app() else {
        return container(text(i18n::text("select-app")).color(skin.dark_foreground))
            .padding(16)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(move |_| sparky_ui::style::panel(skin))
            .into();
    };
    let source = match app.source {
        Source::Flathub(_) => i18n::text("flathub"),
        Source::Native(manager) => manager.label().to_owned(),
    };
    let state = if app.installed {
        "installed-state"
    } else {
        "available-state"
    };
    let mut content = column![
        text(&app.name).size(18).color(skin.foreground),
        text(&app.id).size(11).color(skin.dark_foreground),
        text(&app.summary).size(13),
        text(i18n::with("source", "value", source)).size(12),
        text(i18n::text(state)).size(12).color(skin.accent),
    ]
    .spacing(12);
    if let Some(version) = &app.version {
        content = content.push(text(i18n::with("version", "value", version.clone())).size(12));
    }
    if app.installed {
        content = content.push(
            row![
                control(
                    i18n::text("update"),
                    Message::Ask(Action::Update),
                    store.busy
                ),
                control(
                    i18n::text("remove"),
                    Message::Ask(Action::Remove),
                    store.busy
                ),
            ]
            .spacing(8),
        );
    } else {
        content = content.push(control(
            i18n::text("install"),
            Message::Ask(Action::Install),
            store.busy,
        ));
    }
    if let Some((pending, action)) = &store.pending {
        let key = match action {
            Action::Install => "confirm-install",
            Action::Update => "confirm-update",
            Action::Remove => "confirm-remove",
        };
        content =
            content.push(text(i18n::with(key, "name", pending.name.clone())).color(skin.urgent));
        if *action == Action::Update && pending.source == Source::Native(Manager::Pacman) {
            content = content.push(text(i18n::text("pacman-system-update")).size(12));
        }
        content = content.push(
            row![
                control(i18n::text("cancel"), Message::Cancel, store.busy),
                control(i18n::text("confirm"), Message::Confirm, store.busy),
            ]
            .spacing(8),
        );
    }
    container(content)
        .padding(16)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::panel(skin))
        .into()
}
