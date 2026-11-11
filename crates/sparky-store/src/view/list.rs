use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};

use crate::app::{Message, Page, Store};
use crate::i18n;

pub fn view(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    let mut entries = column![].spacing(4).width(Length::Fill);
    for (index, app) in store.apps.iter().take(store.visible_count).enumerate() {
        let label = column![
            text(&app.name).size(14),
            text(format!("{} · {}", app.id, app.summary))
                .size(11)
                .color(skin.dark_foreground),
        ]
        .spacing(3);
        let selected = sparky_ui::style::Highlight::when(store.selected == Some(index));
        entries = entries.push(
            button(label)
                .on_press(Message::Select(index))
                .padding(8)
                .width(Length::Fill)
                .style(move |_, status| sparky_ui::style::entry(skin, selected, status)),
        );
    }
    if store.apps.is_empty() && !store.busy {
        let key = match store.page {
            Page::Browse => "browse-empty",
            Page::Installed => "empty-installed",
            Page::Search => "no-results",
        };
        entries = entries.push(text(i18n::text(key)).color(skin.dark_foreground));
    }
    if store.apps.len() > store.visible_count {
        entries = entries.push(super::control(
            i18n::text("load-more"),
            Message::LoadMore,
            store.busy,
        ));
    }
    container(scrollable(entries).height(Length::Fill))
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(12)
        .style(move |_| sparky_ui::style::panel(skin))
        .into()
}
