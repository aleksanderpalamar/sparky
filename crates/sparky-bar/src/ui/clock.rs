use iced::widget::{container, mouse_area, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use crate::ui::style::{FONT, PADDING_X, TEXT_SIZE};

pub fn clock<'a, M: Clone + 'a>(skin: &Skin, label: String, on_toggle: M) -> Element<'a, M> {
    let label = text(label).font(FONT).size(TEXT_SIZE).color(skin.foreground);
    let body = container(label)
        .height(Length::Fill)
        .padding([0.0, PADDING_X])
        .align_y(Alignment::Center);
    mouse_area(body).on_right_press(on_toggle).into()
}
