use iced::widget::{Space, button, column, container, mouse_area, row, stack, text, text_input};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE, TITLE_SIZE};

use crate::app::message::Message;
use crate::app::state::Launcher;
use crate::app::update::NAME_INPUT;
use crate::app::view::menu;
use crate::domain::overlay::{Overlay, Purpose};
use crate::i18n;

const DIALOG_WIDTH: f32 = 340.0;

pub fn layer<'a>(launcher: &'a Launcher, base: Element<'a, Message>) -> Element<'a, Message> {
    let skin = launcher.skin;
    let front = match &launcher.overlay {
        Overlay::Closed => return base,
        Overlay::Menu(menu) => menu::layer(launcher, menu.point, menu.cursor),
        Overlay::Naming {
            purpose,
            text,
            refused,
        } => dialog(skin, purpose, text, *refused),
    };
    let backdrop = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(Message::Dismiss)
        .on_right_press(Message::Dismiss);
    stack![base, backdrop, front]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn dialog(skin: Skin, purpose: &Purpose, typed: &str, refused: bool) -> Element<'static, Message> {
    let title = match purpose {
        Purpose::Create { .. } => i18n::text("group-new-title"),
        Purpose::Rename(_) => i18n::text("group-rename-title"),
    };
    let input = text_input(&i18n::text("group-name-placeholder"), typed)
        .id(NAME_INPUT)
        .on_input(Message::Named)
        .on_submit(Message::ConfirmName)
        .size(TEXT_SIZE)
        .style(style::input)
        .padding(PADDING / 1.5);
    let hint = match refused {
        true => text(i18n::text("group-name-refused")).color(skin.urgent),
        false => text(""),
    };
    let actions = row![
        Space::new().width(Length::Fill),
        action(skin, i18n::text("group-cancel"), Message::Dismiss),
        action(skin, i18n::text("group-save"), Message::ConfirmName),
    ]
    .spacing(SPACING);
    let body = column![
        text(title).size(TITLE_SIZE).color(skin.bright_foreground),
        input,
        hint.size(SMALL_TEXT),
        actions
    ]
    .spacing(SPACING);
    let pane = container(body)
        .width(DIALOG_WIDTH)
        .padding(PADDING)
        .style(move |_| frame(skin));
    container(pane).center(Length::Fill).into()
}

fn action(skin: Skin, label: String, message: Message) -> Element<'static, Message> {
    button(text(label).size(13.0).color(skin.foreground))
        .padding([6.0, 14.0])
        .style(move |_, status| style::entry(skin, Highlight::Plain, status))
        .on_press(message)
        .into()
}

pub fn frame(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.raised)),
        text_color: Some(skin.foreground),
        border: Border {
            color: skin.accent,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
