use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::domain::{Action, Application, Manager, Source};
use crate::i18n;

use super::{icon, navigation, styles};

pub fn view(store: &Store) -> Element<'_, Message> {
    let Some(app) = store.selected_app() else {
        return text("").into();
    };
    let mut content = column![header(store, app), preview(store, app), facts(store, app)]
        .spacing(20)
        .padding(24);
    if let Some((pending, action)) = &store.pending {
        content = content.push(confirmation(store, pending, *action));
    }
    scrollable(content).height(Length::Fill).into()
}

fn header<'a>(store: &'a Store, app: &'a Application) -> Element<'a, Message> {
    let skin = store.skin;
    let identity = row![
        icon::view(store, app, 96.0),
        column![
            text(&app.name).size(24).color(skin.bright_foreground),
            text(&app.summary).size(13).color(skin.foreground),
            text(source_name(app.source)).size(12).color(skin.accent),
        ]
        .spacing(8)
        .width(Length::Fill),
    ]
    .spacing(20)
    .align_y(Alignment::Center)
    .width(Length::Fill);
    let actions = if app.installed {
        row![
            action(store, "update", Message::Ask(Action::Update)),
            navigation::control(store, &i18n::text("remove"), Message::Ask(Action::Remove)),
        ]
        .spacing(8)
    } else {
        row![action(store, "install", Message::Ask(Action::Install))]
    };
    container(
        row![identity, actions]
            .spacing(24)
            .align_y(Alignment::Center),
    )
    .padding(20)
    .width(Length::Fill)
    .style(move |_| styles::surface(skin))
    .into()
}

fn action<'a>(store: &Store, key: &'static str, message: Message) -> Element<'a, Message> {
    let skin = store.skin;
    let button = button(text(i18n::text(key)).size(13))
        .padding([10, 20])
        .style(move |_, status| styles::action(skin, status));
    if store.busy {
        return button.into();
    }
    button.on_press(message).into()
}

fn preview<'a>(store: &'a Store, app: &'a Application) -> Element<'a, Message> {
    let skin = store.skin;
    container(
        column![
            icon::view(store, app, 112.0),
            text(i18n::text("preview-unavailable"))
                .size(12)
                .color(skin.foreground),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .height(220)
    .width(Length::Fill)
    .center_x(Length::Fill)
    .center_y(220)
    .style(move |_| styles::feature(skin))
    .into()
}

fn facts<'a>(store: &Store, app: &'a Application) -> Element<'a, Message> {
    let skin = store.skin;
    let version = app.version.clone().unwrap_or_else(|| i18n::text("unknown"));
    let cells = [
        ("source-label", source_name(app.source)),
        ("version-label", version),
        (
            "status-label",
            i18n::text(if app.installed {
                "installed-state"
            } else {
                "available-state"
            }),
        ),
    ];
    let facts = cells.into_iter().map(|(key, value)| {
        container(
            column![
                text(i18n::text(key)).size(11).color(skin.dark_foreground),
                text(value).size(14).color(skin.bright_foreground),
            ]
            .spacing(8),
        )
        .padding(16)
        .width(Length::Fill)
        .style(move |_| styles::surface(skin))
        .into()
    });
    column![
        text(i18n::text("details")).size(18),
        iced::widget::Row::with_children(facts).spacing(12),
        container(
            column![
                text(i18n::text("application-id"))
                    .size(11)
                    .color(skin.dark_foreground),
                text(&app.id).size(13),
            ]
            .spacing(8),
        )
        .padding(16)
        .width(Length::Fill)
        .style(move |_| styles::surface(skin)),
    ]
    .spacing(12)
    .into()
}

fn confirmation<'a>(store: &Store, app: &Application, action: Action) -> Element<'a, Message> {
    let skin = store.skin;
    let key = match action {
        Action::Install => "confirm-install",
        Action::Update => "confirm-update",
        Action::Remove => "confirm-remove",
    };
    let mut content = column![text(i18n::with(key, "name", app.name.clone())).size(14)].spacing(12);
    if action == Action::Update && app.source == Source::Native(Manager::Pacman) {
        content = content.push(text(i18n::text("pacman-system-update")).size(12));
    }
    content = content.push(
        row![
            navigation::control(store, &i18n::text("cancel"), Message::Cancel),
            action_button(store, Message::Confirm),
        ]
        .spacing(8),
    );
    container(content)
        .padding(20)
        .width(Length::Fill)
        .style(move |_| styles::feature(skin))
        .into()
}

fn action_button<'a>(store: &Store, message: Message) -> Element<'a, Message> {
    let skin = store.skin;
    button(text(i18n::text("confirm")).size(12))
        .padding([8, 20])
        .on_press(message)
        .style(move |_, status| styles::action(skin, status))
        .into()
}

fn source_name(source: Source) -> String {
    match source {
        Source::Flathub(_) => i18n::text("flathub"),
        Source::Native(manager) => manager.label().to_owned(),
    }
}
