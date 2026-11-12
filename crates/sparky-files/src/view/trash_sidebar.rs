use iced::alignment::Vertical;
use iced::widget::{Column, button as widget_button, mouse_area, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight};

use crate::app::message::{Message, TrashEvent};
use crate::app::state::Files;
use crate::domain::drop::DropTarget;
use crate::domain::icons::IconKind;
use crate::i18n;
use crate::view::icon::sidebar_icon;

pub fn buttons(files: &Files) -> Element<'_, Message> {
    let home = std::iter::once(row_for(
        files,
        i18n::text("trash"),
        TrashEvent::Open,
        files.location == files.services.trash.directory(),
    ));
    let others = files
        .other_trashes
        .iter()
        .enumerate()
        .map(|(index, trash)| {
            let label = trash.label().unwrap_or_default();
            row_for(
                files,
                i18n::with("trash-of", "disk", label),
                TrashEvent::OpenOther(index),
                files.location == trash.directory(),
            )
        });
    Column::with_children(home.chain(others)).spacing(3).into()
}

fn row_for(files: &Files, title: String, event: TrashEvent, active: bool) -> Element<'_, Message> {
    let skin = files.skin;
    let label = row![
        sidebar_icon(files, IconKind::Trash, 20.0),
        text(title).size(13.0)
    ]
    .spacing(6)
    .align_y(Vertical::Center);
    let home = matches!(event, TrashEvent::Open);
    let dropping = home && files.is_drop_target(DropTarget::Trash);
    let control = widget_button(label)
        .width(Length::Fill)
        .on_press(Message::Trash(event))
        .style(move |_, status| style::entry(skin, Highlight::when(active || dropping), status));
    match home {
        true => mouse_area(control)
            .on_enter(Message::Hover(DropTarget::Trash))
            .on_exit(Message::Unhover(DropTarget::Trash))
            .into(),
        false => control.into(),
    }
}
