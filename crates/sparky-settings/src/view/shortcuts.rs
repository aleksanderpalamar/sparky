use iced::alignment::Vertical;
use iced::widget::{Column, container, mouse_area, row, text};
use iced::{Element, Font, Length};
use sparky_ui::style::{self, Highlight, PADDING, SPACING, TEXT_SIZE};
use sparky_ui::wheel;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::describe::describe;
use crate::view::style::{KEYS_WIDTH, LINE_HEIGHT};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let binds = settings.binds();
    let range = settings.list.range(binds.len());
    let rows: Vec<Element<'_, Message>> = range
        .filter_map(|index| binds.get(index).map(|bind| (index, bind)))
        .map(|(index, bind)| {
            let line = row![
                text(bind.keys.clone()).font(Font::MONOSPACE).size(TEXT_SIZE).color(skin.accent).width(KEYS_WIDTH),
                text(describe(&bind.action)).size(TEXT_SIZE).width(Length::Fill),
            ]
            .spacing(SPACING)
            .align_y(Vertical::Center);
            let highlight = Highlight::when(settings.focus == index);
            container(line)
                .height(LINE_HEIGHT)
                .padding([0.0, PADDING])
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .style(move |_| style::row(skin, highlight))
                .into()
        })
        .collect();
    mouse_area(Column::with_children(rows).width(Length::Fill).height(Length::Fill))
        .on_scroll(|delta| Message::Scrolled(wheel::rows(delta, LINE_HEIGHT)))
        .into()
}
