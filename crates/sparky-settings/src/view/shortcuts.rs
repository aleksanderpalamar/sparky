use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Alignment, Element, Length};
use sparky_ui::wheel;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::describe::describe;
use crate::view::kit::rows::{caption, cell};
use crate::view::kit::tokens::{size, space};
use crate::view::style::{KEYS_WIDTH, LINE_HEIGHT};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let binds = settings.binds();
    let rows: Vec<Element<'_, Message>> = settings
        .list
        .range(binds.len())
        .filter_map(|index| binds.get(index).map(|bind| (index, bind)))
        .map(|(index, bind)| {
            let line = row![
                text(bind.keys.clone())
                    .size(size::BODY)
                    .color(skin.accent)
                    .width(KEYS_WIDTH),
                text(describe(&bind.action))
                    .size(size::BODY)
                    .width(Length::Fill),
            ]
            .spacing(space::XXL)
            .align_y(Alignment::Center);
            let padded = container(line)
                .height(space::ROW_HEIGHT)
                .padding([0.0, space::LG])
                .align_y(Alignment::Center)
                .into();
            cell(skin, settings.focus == index, padded)
        })
        .collect();
    let list = mouse_area(
        Column::with_children(rows)
            .spacing(space::XXS)
            .width(Length::Fill),
    )
    .on_scroll(|delta| Message::Scrolled(wheel::rows(delta, LINE_HEIGHT)));
    column![caption(skin, &crate::i18n::text("shortcuts-active")), list]
        .spacing(space::MD)
        .into()
}
