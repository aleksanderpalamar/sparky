use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::i18n;

pub fn view(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    container(
        column![
            text("↻").size(64).color(skin.accent),
            text(i18n::text("updates")).size(22),
            text(i18n::text("updates-unavailable"))
                .size(13)
                .color(skin.foreground),
        ]
        .spacing(16)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}
