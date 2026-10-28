use iced::widget::{container, row};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::ui::clock::clock;
use crate::ui::menu::menu;
use crate::ui::tray::tray;
use crate::ui::workspaces::workspaces;

pub fn view(bar: &Bar, window: iced::window::Id) -> Element<'_, Message> {
    if let Some(open) = bar.panels.open.filter(|open| open.window == window) {
        return crate::ui::panels::view(bar.skin(), &bar.panels, open.kind);
    }
    let open = bar.tray().menu().filter(|menu| menu.window == window);
    match open.and_then(|menu| bar.tray().tray().item(&menu.address)) {
        Some(item) => menu(bar.skin(), &item.menu),
        None => strip(bar, window),
    }
}

fn strip(bar: &Bar, window: iced::window::Id) -> Element<'_, Message> {
    let skin = bar.skin();
    let left = container(workspaces(skin, bar.slots(), Message::FocusWorkspace))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start);
    let center = clock(skin, bar.clock(), Message::ToggleClock);
    let right = container(tray(
        skin,
        bar.tray().tray().items(),
        window,
        bar.tray().drawers.progress(window),
    ))
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
