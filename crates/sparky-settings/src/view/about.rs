use iced::alignment::Vertical;
use iced::widget::{Column, container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{PADDING, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::view::style::KEYS_WIDTH;

const UNKNOWN: &str = "não informado";
const ROW_PADDING: f32 = 6.0;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let about = &settings.context.about;
    let known = |value: &Option<String>| value.clone().unwrap_or_else(|| UNKNOWN.to_owned());
    let facts = [
        ("Sparky", env!("CARGO_PKG_VERSION").to_owned()),
        ("Sistema", known(&about.distribution)),
        ("Kernel", known(&about.kernel)),
        ("Computador", known(&about.host)),
        ("Configuração", sparky_xdg::tilde(&settings.services.store.location(), &settings.context.home)),
    ];
    let rows = facts.into_iter().map(|(label, value)| {
        let line = row![
            text(label).size(TEXT_SIZE).color(skin.accent).width(KEYS_WIDTH),
            text(value).size(TEXT_SIZE).width(Length::Fill),
        ]
        .spacing(SPACING)
        .align_y(Vertical::Center);
        container(line)
            .padding([ROW_PADDING, PADDING])
            .align_y(Vertical::Center)
            .into()
    });
    Column::with_children(rows).spacing(2).into()
}
