use super::action;
use crate::{
    app::message::Message,
    domain::calendar::{Action, Calendar, Day},
};
use chrono::Datelike;
use iced::{
    Alignment, Border, Element, Length,
    widget::{column, container, row, text},
};
use sparky_theme::Skin;

pub fn view<'a>(skin: &Skin, calendar: &Calendar) -> Element<'a, Message> {
    let heading = row![
        action(skin, "‹", Message::Calendar(Action::Previous)),
        container(text(calendar.title()).size(13).color(skin.foreground)).center_x(Length::Fill),
        action(skin, "›", Message::Calendar(Action::Next))
    ]
    .spacing(4)
    .align_y(Alignment::Center);
    let weekdays = row(["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"]
        .into_iter()
        .map(|label| {
            container(text(label).size(11).color(skin.accent))
                .center_x(Length::Fill)
                .into()
        }))
    .spacing(2);
    let days = calendar.days();
    let weeks = column(days.chunks(7).map(|week| {
        row(week.iter().map(|day| cell(skin, *day)))
            .spacing(2)
            .into()
    }))
    .spacing(2);
    column![
        heading,
        text(calendar.today().format("Hoje · %d/%m/%Y").to_string())
            .size(12)
            .color(skin.foreground),
        weekdays,
        weeks,
        action(skin, "Hoje", Message::Calendar(Action::Today))
    ]
    .spacing(10)
    .width(Length::Fill)
    .into()
}

fn cell<'a>(skin: &Skin, day: Day) -> Element<'a, Message> {
    let color = match (day.today, day.current) {
        (true, _) => skin.bright_foreground,
        (false, true) => skin.foreground,
        _ => skin.dark_foreground,
    };
    let skin = *skin;
    container(text(day.date.day().to_string()).size(12).color(color))
        .center_x(Length::Fill)
        .center_y(28)
        .style(move |_| container::Style {
            background: day.today.then(|| skin.fill(0.18).into()),
            border: Border {
                color: skin.accent,
                width: if day.today { 1.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
