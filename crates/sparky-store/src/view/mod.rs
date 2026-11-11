mod detail;
mod list;

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::domain::SearchSource;
use crate::i18n;

pub fn view(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    let heading = container(
        row![
            text(i18n::text("title"))
                .size(18)
                .color(skin.foreground)
                .width(Length::Fill),
            text(if store.busy {
                i18n::text("busy")
            } else {
                String::new()
            })
            .size(12)
            .color(skin.dark_foreground)
        ]
        .align_y(Alignment::Center),
    )
    .padding(14)
    .style(move |_| sparky_ui::style::panel(skin));
    let search = text_input(&i18n::text("search-placeholder"), &store.query)
        .on_input(Message::Query)
        .on_submit(Message::Search)
        .style(sparky_ui::style::input)
        .width(Length::Fill);
    let toolbar = row![
        search,
        control(i18n::text("search"), Message::Search, store.busy),
        control(i18n::text("browse"), Message::Browse, store.busy),
        control(i18n::text("installed"), Message::ShowInstalled, store.busy),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let filters = row![
        filter(store, SearchSource::Preferred, "preferred"),
        filter(store, SearchSource::Native, "native"),
    ]
    .spacing(8);
    let body = row![list::view(store), detail::view(store)]
        .spacing(12)
        .height(Length::Fill);
    let mut content = column![heading, toolbar, filters].spacing(12);
    if store.flathub_missing {
        content = content.push(control(
            i18n::text("enable-flathub"),
            Message::EnableFlathub,
            store.busy,
        ));
    }
    content = content.push(body);
    if let Some(error) = &store.error {
        content = content.push(text(error).size(12).color(skin.urgent));
    }
    if let Some(notice) = &store.notice {
        content = content.push(text(notice).size(12).color(skin.accent));
    }
    container(content.padding(12))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::window(skin))
        .into()
}

fn filter<'a>(store: &'a Store, source: SearchSource, key: &'static str) -> Element<'a, Message> {
    let skin = store.skin;
    let selected = sparky_ui::style::Highlight::when(store.source == source);
    button(text(i18n::text(key)).size(12))
        .on_press(Message::Source(source))
        .style(move |_, status| sparky_ui::style::entry(skin, selected, status))
        .into()
}

pub(super) fn control(
    label: String,
    message: Message,
    disabled: bool,
) -> Element<'static, Message> {
    let button = button(text(label).size(12)).padding(8);
    if disabled {
        return button.into();
    }
    button.on_press(message).into()
}
