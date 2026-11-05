use super::{action, calendar};
use crate::app::{center_state::CenterState, message::Message};
use crate::i18n;
use iced::{
    Element, Length,
    widget::{column, container, row, rule, scrollable, text},
};
use sparky_notifications::{
    client::Command,
    presentation::{Density, card},
};
use sparky_theme::Skin;

pub fn view<'a>(skin: &Skin, center: &'a CenterState) -> Element<'a, Message> {
    row![
        notifications(skin, center),
        rule::vertical(1),
        container(calendar::view(skin, &center.calendar)).width(Length::FillPortion(5))
    ]
    .spacing(14)
    .height(Length::Fill)
    .into()
}

fn notifications<'a>(skin: &Skin, center: &'a CenterState) -> Element<'a, Message> {
    let ready = center.client.is_some() && center.error.is_none();
    let clear = match ready {
        true => Message::NotificationCommand(Command::Clear),
        false => Message::Ignore,
    };
    let header = row![
        text(i18n::text("notifications"))
            .size(13)
            .color(skin.foreground)
            .width(Length::Fill),
        action(skin, i18n::text("clear"), clear)
    ]
    .spacing(8);
    let items: Element<'a, Message> = match &center.error {
        Some(error) => text(format!("{}\n{error}", i18n::text("service-unavailable")))
            .size(12)
            .color(skin.urgent)
            .into(),
        None if center.snapshot.items.is_empty() => text(i18n::text("no-notifications"))
            .size(12)
            .color(skin.dark_foreground)
            .into(),
        None => column(center.snapshot.items.iter().map(|item| {
            card(
                skin,
                item,
                Density::History,
                |id, key| Message::NotificationCommand(Command::Invoke { id, key }),
                Message::NotificationCommand(Command::Dismiss(item.id)),
            )
        }))
        .spacing(8)
        .into(),
    };
    let label = match (ready, center.snapshot.dnd) {
        (false, _) => "dnd-unavailable",
        (true, true) => "dnd-enabled",
        (true, false) => "dnd-disabled",
    };
    let toggle = match ready {
        true => Message::NotificationCommand(Command::Dnd(!center.snapshot.dnd)),
        false => Message::Ignore,
    };
    column![
        header,
        scrollable(items).height(Length::Fill),
        rule::horizontal(1),
        action(skin, i18n::text(label), toggle),
        text(i18n::text("dnd-hint"))
            .size(10)
            .color(skin.dark_foreground)
    ]
    .spacing(8)
    .width(Length::FillPortion(6))
    .height(Length::Fill)
    .into()
}
