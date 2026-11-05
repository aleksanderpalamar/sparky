use iced::alignment::Vertical;
use iced::widget::{container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, STATUS_HEIGHT, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::{Shown, Viewer};

const NARROW: f32 = 760.0;

pub fn status(viewer: &Viewer) -> Element<'_, Message> {
    let skin = viewer.skin;
    let hints = match viewer.size.width < NARROW {
        true => "short-hints",
        false => "hints",
    };
    let line = row![
        text(summary(viewer))
            .size(TEXT_SIZE)
            .color(skin.foreground)
            .width(Length::Fill),
        text(crate::i18n::text(hints))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    container(line)
        .height(STATUS_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .style(move |_| style::panel(skin))
        .into()
}

pub fn summary(viewer: &Viewer) -> String {
    let Some(album) = &viewer.album else {
        return String::new();
    };
    let name = album
        .current()
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (index, total) = album.position();
    let mut parts = vec![name, format!("{index}/{total}")];
    if let Shown::Ready(ready) = &viewer.shown {
        parts.push(format!("{} × {}", ready.width, ready.height));
        parts.push(format!("{}%", ready.viewport.percent(viewer.area())));
    }
    parts.join(" · ")
}
