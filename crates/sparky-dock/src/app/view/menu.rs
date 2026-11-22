use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{SMALL_TEXT, TEXT_SIZE};

use super::style;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::layout::MENU_WIDTH;
use crate::i18n::text as tr;

const PADDING: f32 = 8.0;

pub fn menu<'a>(dock: &'a Dock, id: &str) -> Option<Element<'a, Message>> {
    let skin = dock.skin;
    let entries = dock.entries();
    let index = entries.iter().position(|(app, _)| app.id == id)?;
    let (app, _) = entries.get(index)?;
    let layout = dock.layout();
    let surface = dock
        .surface
        .map_or(MENU_WIDTH, |geometry| geometry.size.0 as f32);
    let center = *layout.centers(entries.len(), surface).get(index)?;
    let left = (center - MENU_WIDTH / 2.0).clamp(0.0, surface - MENU_WIDTH);
    let remove = button(text(tr("menu-remove")).size(TEXT_SIZE))
        .width(Length::Fill)
        .padding([6.0, PADDING])
        .on_press(Message::Unpin(id.to_owned()))
        .style(move |_, status| style::menu_entry(skin, status));
    let card = container(
        column![
            text(app.name.clone())
                .size(SMALL_TEXT)
                .color(skin.dark_foreground),
            remove
        ]
        .spacing(4),
    )
    .width(MENU_WIDTH)
    .padding(PADDING)
    .style(move |_| style::menu(skin));
    let placed = row![Space::new().width(left), card];
    Some(
        container(placed)
            .width(surface)
            .padding(iced::Padding::ZERO.bottom(6))
            .into(),
    )
}
