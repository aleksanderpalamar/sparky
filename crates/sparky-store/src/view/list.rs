use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::domain::Action;
use crate::i18n;

use super::{icon, navigation, styles};

pub fn search(store: &Store) -> Element<'_, Message> {
    listing(store, "search-results", "no-results", false)
}

pub fn installed(store: &Store) -> Element<'_, Message> {
    listing(store, "installed", "empty-installed", true)
}

fn listing<'a>(
    store: &'a Store,
    title: &'static str,
    empty: &'static str,
    can_remove: bool,
) -> Element<'a, Message> {
    let skin = store.skin;
    let mut content = column![text(i18n::text(title)).size(20)]
        .spacing(12)
        .padding(24);
    if store.apps.is_empty() && !store.busy {
        content = content.push(text(i18n::text(empty)).color(skin.dark_foreground));
    }
    for (index, app) in store.apps.iter().take(store.visible_count).enumerate() {
        let name = button(
            row![
                icon::view(store, app, 56.0),
                column![
                    text(&app.name).size(15),
                    text(&app.id).size(11).color(skin.dark_foreground),
                    text(&app.summary).size(11).color(skin.dark_foreground),
                ]
                .spacing(3)
                .width(Length::Fill),
            ]
            .spacing(14)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(12)
        .on_press(Message::Select(index))
        .style(move |_, status| styles::tile(skin, status));
        let mut entry = row![name].spacing(8).align_y(Alignment::Center);
        if can_remove {
            entry = entry.push(
                button(text(i18n::text("remove")).size(12))
                    .padding(12)
                    .on_press(Message::AskFor(index, Action::Remove))
                    .style(move |_, status| styles::tile(skin, status)),
            );
        }
        content = content.push(
            container(entry)
                .width(Length::Fill)
                .style(move |_| styles::surface(skin)),
        );
    }
    if store.apps.len() > store.visible_count {
        content = content.push(navigation::control(
            store,
            &i18n::text("load-more"),
            Message::LoadMore,
        ));
    }
    scrollable(content).height(Length::Fill).into()
}
