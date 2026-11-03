use iced::Element;
use iced::widget::{button, container, mouse_area, row, svg};

use crate::app::widgets::{Look, underlined};
use crate::app::{Message, icons, style};
use crate::domain::focus::Focus;
use crate::domain::power::{self, Power};

pub fn icons<'a>(look: Look) -> Element<'a, Message> {
    row(Power::ALL.map(|action| icon(action, look)))
        .spacing(look.layout.power_spacing)
        .into()
}

pub fn labels<'a>(focus: Focus, look: Look) -> Element<'a, Message> {
    row(Power::ALL.map(|action| label(action, focus == Focus::Power(action), look)))
        .spacing(look.layout.power_spacing)
        .into()
}

fn icon<'a>(action: Power, look: Look) -> Element<'a, Message> {
    let layout = look.layout;
    let picture = svg(icons::power(action))
        .width(layout.power_icon)
        .height(layout.power_icon)
        .style(style::tint(look.colors.clock));
    button(container(picture).center_x(layout.power_width))
        .padding(0)
        .style(style::bare)
        .on_press(Message::Power(action))
        .into()
}

fn label<'a>(action: Power, focused: bool, look: Look) -> Element<'a, Message> {
    let text = container(underlined(power::label(action).to_owned(), focused, look))
        .center_x(look.layout.power_width);
    mouse_area(text).on_press(Message::Power(action)).into()
}
