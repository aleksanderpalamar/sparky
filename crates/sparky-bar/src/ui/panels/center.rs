use super::{action, calendar};
use crate::app::{center_state::CenterState, message::Message};
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
        text("Notificações")
            .size(13)
            .color(skin.foreground)
            .width(Length::Fill),
        action(skin, "Limpar", clear)
    ]
    .spacing(8);
    let items: Element<'a, Message> = match &center.error {
        Some(error) => text(format!("Serviço indisponível\n{error}"))
            .size(12)
            .color(skin.urgent)
            .into(),
        None if center.snapshot.items.is_empty() => text("Nenhuma notificação nesta sessão")
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
        (false, _) => "Não perturbe · indisponível",
        (true, true) => "Não perturbe · ativado",
        (true, false) => "Não perturbe · desativado",
    };
    let toggle = match ready {
        true => Message::NotificationCommand(Command::Dnd(!center.snapshot.dnd)),
        false => Message::Ignore,
    };
    column![
        header,
        scrollable(items).height(Length::Fill),
        rule::horizontal(1),
        action(skin, label, toggle),
        text("Silencia pop-ups e mantém o histórico")
            .size(10)
            .color(skin.dark_foreground)
    ]
    .spacing(8)
    .width(Length::FillPortion(6))
    .height(Length::Fill)
    .into()
}
