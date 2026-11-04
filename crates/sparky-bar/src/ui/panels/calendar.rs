use super::action;
use crate::{
    app::message::Message,
    domain::calendar::{Action, Calendar, Day},
};
use chrono::Datelike;
use fluent_bundle::FluentArgs;
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
    let weekdays = row([
        "weekday-short-monday",
        "weekday-short-tuesday",
        "weekday-short-wednesday",
        "weekday-short-thursday",
        "weekday-short-friday",
        "weekday-short-saturday",
        "weekday-short-sunday",
    ]
    .into_iter()
    .map(|label| {
        container(text(crate::i18n::text(label)).size(11).color(skin.accent))
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
        text(today_label(calendar)).size(12).color(skin.foreground),
        weekdays,
        weeks,
        action(
            skin,
            crate::i18n::text("today"),
            Message::Calendar(Action::Today)
        )
    ]
    .spacing(10)
    .width(Length::Fill)
    .into()
}

fn today_label(calendar: &Calendar) -> String {
    let today = calendar.today();
    let mut args = FluentArgs::new();
    args.set("day", format!("{:02}", today.day()));
    args.set("month", format!("{:02}", today.month()));
    args.set("year", today.year().to_string());
    crate::i18n::message("today-date", Some(&args))
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
