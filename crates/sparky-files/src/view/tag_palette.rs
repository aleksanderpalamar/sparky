use iced::widget::{Row, Space, button, column, container, text, tooltip};
use iced::{Background, Border, Color, Element};
use sparky_theme::Skin;
use sparky_ui::style::{self, SMALL_TEXT};

use crate::app::message::Message;
use crate::domain::context::{Action, PALETTE_HEIGHT};
use crate::domain::tag_color::{ALL, TagColor};
use crate::i18n;

const SWATCH: f32 = 20.0;

pub fn palette(skin: Skin, selected: Option<TagColor>) -> Element<'static, Message> {
    let caption = text(i18n::text("menu-color-tag"))
        .size(SMALL_TEXT)
        .color(skin.dark_foreground);
    let swatches = Row::with_children(
        ALL.into_iter()
            .map(|color| swatch(skin, color, selected == Some(color))),
    )
    .spacing(6);
    container(column![caption, swatches].spacing(6))
        .height(PALETTE_HEIGHT)
        .padding([6, 10])
        .into()
}

pub fn tint(color: TagColor) -> Color {
    let (red, green, blue) = color.rgb();
    Color::from_rgb8(red, green, blue)
}

fn swatch(skin: Skin, color: TagColor, selected: bool) -> Element<'static, Message> {
    let fill = tint(color);
    let body = button(Space::new().width(SWATCH).height(SWATCH))
        .padding(0)
        .on_press(Message::ContextAction(Action::Tag(Some(color))))
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let outline = match selected || hovered {
                true => skin.foreground,
                false => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(fill)),
                border: Border {
                    color: outline,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..button::Style::default()
            }
        });
    tooltip(
        body,
        text(i18n::text(color.label())).size(SMALL_TEXT),
        tooltip::Position::Bottom,
    )
    .style(move |_| style::tooltip(skin))
    .into()
}
