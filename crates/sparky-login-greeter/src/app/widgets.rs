use iced::alignment::Horizontal;
use iced::widget::svg::Handle;
use iced::widget::text::Span;
use iced::widget::{button, container, mouse_area, rich_text, span, stack, svg, text, text_input};
use iced::{Element, Font, Length, Padding};

use crate::app::style::{self, BOLD, Colors, REGULAR};
use crate::app::update::{PASSWORD, USERNAME};
use crate::app::{Greeter, Message, icons};
use crate::domain::focus::Focus;
use crate::domain::form::Reveal;
use crate::domain::layout::Layout;

const LOGIN: &str = "Login";
const LINE: f32 = 1.3;

#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub layout: Layout,
    pub colors: Colors,
}

struct Field<'a> {
    id: &'static str,
    value: &'a str,
    secure: bool,
    typed: fn(String) -> Message,
    submitted: Message,
    icon: Handle,
    icon_press: Option<Message>,
}

pub fn clock<'a>(content: String, size: f32, look: Look) -> Element<'a, Message> {
    text(content)
        .size(size)
        .font(BOLD)
        .color(look.colors.clock)
        .into()
}

pub fn warning<'a>(content: Option<String>, look: Look) -> Element<'a, Message> {
    let italic = Font {
        style: iced::font::Style::Italic,
        ..REGULAR
    };
    text(content.unwrap_or_default())
        .size(look.layout.small)
        .font(italic)
        .color(look.colors.warning)
        .into()
}

pub fn username(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let field = Field {
        id: USERNAME,
        value: greeter.form.username(),
        secure: false,
        typed: Message::TypedUsername,
        submitted: Message::Submitted(Focus::Username),
        icon: icons::user(),
        icon_press: None,
    };
    input(field, look)
}

pub fn password(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let reveal = greeter.form.reveal();
    let field = Field {
        id: PASSWORD,
        value: greeter.form.password(),
        secure: reveal == Reveal::Hidden,
        typed: Message::TypedPassword,
        submitted: Message::Submitted(Focus::Password),
        icon: icons::password(reveal),
        icon_press: Some(Message::Reveal),
    };
    input(field, look)
}

fn input(field: Field<'_>, look: Look) -> Element<'_, Message> {
    let layout = look.layout;
    let vertical = ((layout.field_height - layout.text * LINE) / 2.0).max(0.0);
    let padding = Padding {
        top: vertical,
        bottom: vertical,
        left: layout.field_height,
        right: layout.field_height,
    };
    let entry = text_input("", field.value)
        .id(field.id)
        .secure(field.secure)
        .on_input(field.typed)
        .on_submit(field.submitted)
        .align_x(Horizontal::Center)
        .font(BOLD)
        .size(layout.text)
        .padding(padding)
        .width(Length::Fill)
        .style(style::field(look.colors));
    let icon = svg(field.icon)
        .width(layout.field_icon)
        .height(layout.field_icon)
        .style(style::tint(look.colors.icon));
    let icon = container(icon)
        .center_x(layout.field_height)
        .center_y(layout.field_height);
    let icon: Element<'_, Message> = match field.icon_press {
        Some(message) => mouse_area(icon).on_press(message).into(),
        None => icon.into(),
    };
    stack![entry, icon]
        .width(layout.field_width)
        .height(layout.field_height)
        .into()
}

pub fn login(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let layout = look.layout;
    let ready = greeter.form.ready();
    let focused = greeter.focus == Focus::Login;
    let label = container(text(LOGIN).size(layout.text).font(BOLD))
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    button(label)
        .width(layout.field_width)
        .height(layout.field_height)
        .padding(0)
        .style(style::login(look.colors, focused, ready))
        .on_press_maybe(ready.then_some(Message::Login))
        .into()
}

pub fn underlined<'a>(content: String, focused: bool, look: Look) -> Element<'a, Message> {
    let spans: [Span<'a, (), Font>; 1] = [span(content).underline(focused)];
    rich_text(spans)
        .size(look.layout.small)
        .font(REGULAR)
        .color(look.colors.clock)
        .into()
}

pub fn session(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let Some(sessions) = &greeter.sessions else {
        return text("").into();
    };
    let label = underlined(sessions.label(), greeter.focus == Focus::Session, look);
    mouse_area(label).on_press(Message::NextSession).into()
}
