use iced::widget::{Column, Row, button, column, container, responsive, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::i18n;

use super::{icon, navigation, styles};

const TOPICS: [(&str, &str); 6] = [
    ("topic-create", "create"),
    ("topic-work", "work"),
    ("topic-play", "game"),
    ("topic-social", "social"),
    ("topic-learn", "learn"),
    ("topic-develop", "develop"),
];

pub fn view(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    let mut content = column![].spacing(24).padding(24);
    if let Some(feature) = feature(store) {
        content = content.push(feature);
    }
    content = content.push(text(i18n::text("quick-search")).size(18));
    content = content.push(topics(store));
    content = content.push(text(i18n::text("catalog-apps")).size(18));
    if store.apps.is_empty() && !store.busy {
        content = content.push(text(i18n::text("browse-empty")).color(skin.dark_foreground));
    }
    content = content.push(responsive(move |size| grid(store, size.width)));
    if store.apps.len() > store.visible_count {
        content = content.push(navigation::control(
            store,
            &i18n::text("load-more"),
            Message::LoadMore,
        ));
    }
    if store.flathub_missing {
        content = content.push(navigation::control(
            store,
            &i18n::text("enable-flathub"),
            Message::EnableFlathub,
        ));
    }
    scrollable(content).height(Length::Fill).into()
}

fn feature(store: &Store) -> Option<Element<'_, Message>> {
    let count = store.apps.len().min(5);
    let index = store.feature_index.min(count.checked_sub(1)?);
    let app = store.apps.get(index)?;
    let skin = store.skin;
    let focus = button(
        column![
            icon::view(store, app, 96.0),
            text(&app.name).size(24).color(skin.bright_foreground),
            text(&app.summary).size(13).color(skin.foreground),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .on_press(Message::Select(index))
    .width(Length::Fill)
    .style(move |_, status| styles::tile(skin, status));
    let previous = navigation::control(store, "‹", Message::FeaturePrevious);
    let next = navigation::control(store, "›", Message::FeatureNext);
    Some(
        container(
            column![
                text(i18n::text("discover"))
                    .size(13)
                    .color(skin.dark_foreground),
                row![previous, focus, next]
                    .spacing(16)
                    .align_y(Alignment::Center),
                text(format!("{} / {}", index + 1, count))
                    .size(11)
                    .color(skin.dark_foreground),
            ]
            .spacing(14)
            .align_x(Alignment::Center),
        )
        .padding(20)
        .width(Length::Fill)
        .style(move |_| styles::feature(skin))
        .into(),
    )
}

fn topics(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    let rows = TOPICS.chunks(3).enumerate().map(|(row_index, chunk)| {
        let tiles = chunk
            .iter()
            .enumerate()
            .map(|(column_index, (key, query))| {
                let index = row_index * 3 + column_index;
                button(text(i18n::text(key)).size(16))
                    .padding(20)
                    .width(Length::Fill)
                    .on_press(Message::Topic((*query).to_owned()))
                    .style(move |_, status| styles::topic(skin, index, status))
                    .into()
            });
        Row::with_children(tiles).spacing(12).into()
    });
    Column::with_children(rows).spacing(12).into()
}

fn grid(store: &Store, width: f32) -> Element<'_, Message> {
    let columns = if width < 580.0 {
        1
    } else if width < 900.0 {
        2
    } else {
        3
    };
    let rows = (0..store.apps.len().min(store.visible_count))
        .collect::<Vec<_>>()
        .chunks(columns)
        .map(|indices| {
            let cards = indices.iter().map(|index| card(store, *index));
            Row::with_children(cards).spacing(12).into()
        })
        .collect::<Vec<Element<'_, Message>>>();
    Column::with_children(rows).spacing(12).into()
}

fn card(store: &Store, index: usize) -> Element<'_, Message> {
    let Some(app) = store.apps.get(index) else {
        return text("").into();
    };
    let skin = store.skin;
    button(
        row![
            icon::view(store, app, 48.0),
            column![
                text(&app.name).size(14),
                text(&app.summary).size(11).color(skin.foreground),
            ]
            .spacing(4)
            .width(Length::Fill),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding(14)
    .width(Length::Fill)
    .on_press(Message::Select(index))
    .style(move |_, status| styles::tile(skin, status))
    .into()
}
