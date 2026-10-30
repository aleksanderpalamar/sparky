use std::sync::OnceLock;

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, image, row, svg, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use crate::adapters::icons::Icons;
use crate::app::message::Message;
use crate::domain::tray::TrayIcon;

const SWITCH_WIDTH: f32 = 42.0;
const SWITCH_HEIGHT: f32 = 22.0;
const SWITCH_KNOB: f32 = 16.0;

pub fn icon<'a>(skin: &Skin, name: &str, size: f32) -> Element<'a, Message> {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    let icons = ICONS.get_or_init(Icons::current);
    let color = skin.foreground;
    match icons.resolve(name, None) {
        Some(TrayIcon::Symbolic(path)) => svg(path)
            .width(size)
            .height(size)
            .style(move |_, _| svg::Style { color: Some(color) })
            .into(),
        Some(TrayIcon::File(path)) if path.extension().is_some_and(|ext| ext == "svg") => {
            svg(path).width(size).height(size).into()
        }
        Some(TrayIcon::File(path)) => image(path).width(size).height(size).into(),
        _ => text("◇").size(size).color(color).into(),
    }
}

pub fn heading<'a>(
    skin: &Skin,
    icon_name: &str,
    title: String,
    subtitle: String,
    toggle: Option<(bool, Message)>,
) -> Element<'a, Message> {
    let labels = column![
        text(title).size(14).color(skin.foreground),
        text(subtitle).size(11).color(skin.dark_foreground),
    ]
    .spacing(3)
    .width(Length::Fill);
    let mut row = row![icon(skin, icon_name, 22.0), labels]
        .spacing(12)
        .align_y(Alignment::Center);
    if let Some((on, message)) = toggle {
        row = row.push(switch(*skin, on, message));
    }
    row.into()
}

pub fn section<'a>(skin: &Skin, label: &'static str) -> Element<'a, Message> {
    text(label).size(10).color(skin.dark_foreground).into()
}

pub fn choice<'a>(
    skin: &Skin,
    icon_name: &str,
    label: String,
    trailing: Option<&'static str>,
    selected: bool,
    message: Option<Message>,
) -> Element<'a, Message> {
    let mut content = row![
        icon(skin, icon_name, 16.0),
        text(label)
            .size(12)
            .color(skin.foreground)
            .width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    if let Some(trailing) = trailing {
        content = content.push(text(trailing).size(11).color(skin.dark_foreground));
    }
    let skin = *skin;
    button(container(content).padding([7, 6]).width(Length::Fill))
        .padding(0)
        .on_press_maybe(message)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: (selected || hovered).then(|| skin.fill(0.16).into()),
                text_color: skin.foreground,
                border: iced::Border {
                    color: if selected {
                        skin.foreground
                    } else {
                        skin.background
                    },
                    width: if selected { 1.0 } else { 0.0 },
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .into()
}

pub(super) fn switch<'a>(skin: Skin, on: bool, message: Message) -> Element<'a, Message> {
    let (knob_color, side) = if on {
        (skin.foreground, Horizontal::Right)
    } else {
        (skin.dark_foreground, Horizontal::Left)
    };
    let knob = container(column![])
        .width(SWITCH_KNOB)
        .height(SWITCH_KNOB)
        .style(move |_| container::Style {
            background: Some(knob_color.into()),
            ..Default::default()
        });
    button(container(knob).width(Length::Fill).align_x(side))
        .width(SWITCH_WIDTH)
        .height(SWITCH_HEIGHT)
        .padding(3)
        .on_press(message)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(skin.fill(if hovered { 0.24 } else { 0.16 }).into()),
                border: iced::Border {
                    color: skin.foreground,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
