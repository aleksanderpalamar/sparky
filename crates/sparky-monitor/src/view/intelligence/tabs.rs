use iced::widget::{Row, button, text};
use iced::{Background, Element};
use sparky_theme::Skin;
use sparky_ui::style::{PADDING, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::domain::decision::Lens;
use crate::i18n::text as tr;

pub fn tabs<'a>(skin: Skin, current: Lens) -> Element<'a, Message> {
    let buttons = Lens::ALL.into_iter().map(|lens| {
        let chosen = lens == current;
        let color = match chosen {
            true => skin.bright_foreground,
            false => skin.dark_foreground,
        };
        button(text(tr(lens.key())).size(TEXT_SIZE).color(color))
            .padding([SPACING * 0.75, PADDING * 1.25])
            .on_press(Message::Look(lens))
            .style(move |_, status| {
                let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
                let background = match (chosen, hovered) {
                    (true, _) => Some(Background::Color(skin.selection)),
                    (false, true) => Some(Background::Color(skin.fill(0.06))),
                    (false, false) => None,
                };
                button::Style {
                    background,
                    text_color: color,
                    ..button::Style::default()
                }
            })
            .into()
    });
    Row::with_children(buttons).spacing(SPACING / 2.0).into()
}
