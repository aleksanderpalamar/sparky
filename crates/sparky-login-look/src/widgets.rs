use iced::alignment::Horizontal;
use iced::widget::svg::Handle;
use iced::widget::text::Span;
use iced::widget::{button, container, mouse_area, rich_text, span, stack, svg, text, text_input};
use iced::{Element, Font, Length, Padding};

use crate::layout::Layout;
use crate::style::{self, BOLD, Colors, REGULAR};

const LINE: f32 = 1.3;

#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub layout: Layout,
    pub colors: Colors,
}

pub struct Field<'a, M> {
    pub id: &'static str,
    pub value: &'a str,
    pub secure: bool,
    pub typed: Option<fn(String) -> M>,
    pub submitted: Option<M>,
    pub icon: Handle,
    pub icon_press: Option<M>,
}

pub fn clock<'a, M: 'a>(content: String, size: f32, look: Look) -> Element<'a, M> {
    text(content)
        .size(size)
        .font(BOLD)
        .color(look.colors.clock)
        .into()
}

pub fn warning<'a, M: 'a>(content: Option<String>, look: Look) -> Element<'a, M> {
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

pub fn input<'a, M: Clone + 'a>(field: Field<'a, M>, look: Look) -> Element<'a, M> {
    let layout = look.layout;
    let vertical = ((layout.field_height - layout.text * LINE) / 2.0).max(0.0);
    let padding = Padding {
        top: vertical,
        bottom: vertical,
        left: layout.field_height,
        right: layout.field_height,
    };
    let mut entry = text_input("", field.value)
        .id(field.id)
        .secure(field.secure)
        .align_x(Horizontal::Center)
        .font(BOLD)
        .size(layout.text)
        .padding(padding)
        .width(Length::Fill)
        .style(style::field(look.colors));
    if let Some(typed) = field.typed {
        entry = entry.on_input(typed);
    }
    if let Some(submitted) = field.submitted {
        entry = entry.on_submit(submitted);
    }
    let icon = svg(field.icon)
        .width(layout.field_icon)
        .height(layout.field_icon)
        .style(style::tint(look.colors.icon));
    let icon = container(icon)
        .center_x(layout.field_height)
        .center_y(layout.field_height);
    let icon: Element<'a, M> = match field.icon_press {
        Some(message) => mouse_area(icon).on_press(message).into(),
        None => icon.into(),
    };
    stack![entry, icon]
        .width(layout.field_width)
        .height(layout.field_height)
        .into()
}

pub fn action<'a, M: Clone + 'a>(
    label: &'a str,
    focused: bool,
    press: Option<M>,
    look: Look,
) -> Element<'a, M> {
    let layout = look.layout;
    let ready = press.is_some();
    let label = container(text(label).size(layout.text).font(BOLD))
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    button(label)
        .width(layout.field_width)
        .height(layout.field_height)
        .padding(0)
        .style(style::login(look.colors, focused, ready))
        .on_press_maybe(press)
        .into()
}

pub fn underlined<'a, M: 'a>(content: String, focused: bool, look: Look) -> Element<'a, M> {
    let spans: [Span<'a, (), Font>; 1] = [span(content).underline(focused)];
    rich_text(spans)
        .size(look.layout.small)
        .font(REGULAR)
        .color(look.colors.clock)
        .into()
}
