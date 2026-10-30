pub mod stage;
pub mod status;
pub mod style;

#[cfg(test)]
mod render_tests;

use iced::widget::{Text, canvas, column, container, text};
use iced::{Element, Length};
use sparky_ui::style::TEXT_SIZE;

use crate::app::message::Message;
use crate::app::state::{Shown, Viewer};
use stage::Stage;

pub fn view(viewer: &Viewer) -> Element<'_, Message> {
    let skin = viewer.skin;
    container(column![content(viewer), status::status(viewer)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::window(skin))
        .into()
}

fn content(viewer: &Viewer) -> Element<'_, Message> {
    let skin = viewer.skin;
    match &viewer.shown {
        Shown::Ready(ready) => canvas(Stage { ready })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        Shown::Loading => centered(text("Carregando…").color(skin.dark_foreground)),
        Shown::Failed(error) => centered(text(error.clone()).color(skin.urgent)),
        Shown::Empty(hint) => centered(text(*hint).color(skin.dark_foreground)),
    }
}

fn centered(message: Text<'_>) -> Element<'_, Message> {
    container(message.size(TEXT_SIZE))
        .center(Length::Fill)
        .into()
}
