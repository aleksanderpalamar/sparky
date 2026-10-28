use super::{icon::icon, style::action};
use crate::domain::{Card, Urgency};
use iced::{
    Border, Element, Length,
    widget::{column, container, row, text},
};
use sparky_theme::Skin;

#[derive(Clone, Copy)]
pub enum Density {
    Popup,
    History,
}

pub fn card<'a, M: Clone + 'a>(
    skin: &Skin,
    card: &'a Card,
    density: Density,
    invoke: impl Fn(u32, String) -> M,
    dismiss: M,
) -> Element<'a, M> {
    let border = if card.notification.urgency == Urgency::Critical {
        skin.urgent
    } else {
        skin.selection
    };
    let heading = row![
        text(&card.notification.app)
            .size(11)
            .color(skin.accent)
            .width(Length::Fill),
        text(time(card.created_ms)).size(10).color(skin.foreground),
        action(skin, "×", dismiss)
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);
    let title = text(&card.notification.title)
        .size(13)
        .color(skin.bright_foreground);
    let limit = match density {
        Density::Popup => 240,
        Density::History => 4000,
    };
    let body = excerpt(&card.notification.body, limit);
    let content = column![title, text(body).size(12).color(skin.foreground)]
        .spacing(5)
        .width(Length::Fill);
    let mut content = column![
        heading,
        row![icon(skin, &card.notification), content].spacing(8)
    ]
    .spacing(7);
    if card.active && !card.notification.actions.is_empty() {
        let buttons = card
            .notification
            .actions
            .iter()
            .map(|entry| action(skin, &entry.label, invoke(card.id, entry.key.clone())));
        content = content.push(column(buttons).spacing(4));
    }
    let skin = *skin;
    container(content)
        .padding(10)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(skin.lighter_background.into()),
            border: Border {
                width: 1.0,
                color: border,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn time(utc: u64) -> String {
    chrono::DateTime::from_timestamp_millis(utc.min(i64::MAX as u64) as i64)
        .map(|time| {
            time.with_timezone(&chrono::Local)
                .format("%H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

fn excerpt(value: &str, max: usize) -> String {
    let Some((end, _)) = value.char_indices().nth(max) else {
        return value.to_owned();
    };
    format!("{}…", &value[..end])
}
