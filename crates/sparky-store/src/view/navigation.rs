use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Page, Store};
use crate::domain::SearchSource;
use crate::i18n;

pub fn view(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    if let Some(app) = store.selected_app() {
        return container(
            row![
                control(store, "←", Message::Back),
                text(&app.name).size(18).width(Length::Fill),
                text(i18n::text("title"))
                    .size(12)
                    .color(skin.dark_foreground),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        )
        .padding([12, 24])
        .width(Length::Fill)
        .style(move |_| super::styles::surface(skin))
        .into();
    }
    let tabs = row![
        tab(store, "browse", Page::Browse, Message::Browse),
        tab(store, "installed", Page::Installed, Message::ShowInstalled),
        tab(store, "updates", Page::Updates, Message::ShowUpdates),
    ]
    .spacing(8);
    let heading = row![
        text(i18n::text("title"))
            .size(18)
            .color(skin.bright_foreground)
            .width(Length::Fill),
        tabs,
    ]
    .align_y(Alignment::Center);
    let search = text_input(&i18n::text("search-placeholder"), &store.query)
        .on_input(Message::Query)
        .on_submit(Message::Search)
        .style(sparky_ui::style::input)
        .width(Length::Fill);
    let tools = row![
        search,
        control(store, &i18n::text("search"), Message::Search),
        source(store, SearchSource::Preferred, "preferred"),
        source(store, SearchSource::Native, "native"),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    container(column![heading, tools].spacing(12))
        .padding([12, 24])
        .width(Length::Fill)
        .style(move |_| super::styles::surface(skin))
        .into()
}

fn tab<'a>(
    store: &'a Store,
    key: &'static str,
    page: Page,
    message: Message,
) -> Element<'a, Message> {
    let skin = store.skin;
    let selected = store.page == page;
    let button = button(text(i18n::text(key)).size(13))
        .padding([8, 16])
        .style(move |_, status| super::styles::tab(skin, selected, status));
    if store.busy {
        return button.into();
    }
    button.on_press(message).into()
}

fn source<'a>(store: &'a Store, source: SearchSource, key: &'static str) -> Element<'a, Message> {
    let skin = store.skin;
    button(text(i18n::text(key)).size(12))
        .padding([8, 12])
        .on_press(Message::Source(source))
        .style(move |_, status| super::styles::tab(skin, store.source == source, status))
        .into()
}

pub(super) fn control<'a>(store: &Store, label: &str, message: Message) -> Element<'a, Message> {
    let skin = store.skin;
    let button = button(text(label.to_owned()).size(12))
        .padding([8, 12])
        .style(move |_, status| super::styles::tile(skin, status));
    if store.busy {
        return button.into();
    }
    button.on_press(message).into()
}
