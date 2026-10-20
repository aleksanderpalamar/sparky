use iced::widget::{container, row};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::ui::clock::clock;
use crate::ui::workspaces::workspaces;

pub fn view(bar: &Bar, _window: iced::window::Id) -> Element<'_, Message> {
    let skin = bar.skin();
    let left = container(workspaces(skin, bar.slots(), Message::FocusWorkspace))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start);
    let center = clock(skin, bar.clock(), Message::ToggleClock);
    let right = container(row![])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::End);

    container(row![left, center, right].height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(background(*skin))
        .into()
}

fn background(skin: sparky_theme::Skin) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(skin.background.into()),
        text_color: Some(skin.foreground),
        ..container::Style::default()
    }
}
