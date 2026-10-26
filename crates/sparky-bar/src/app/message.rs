use iced_layershell::to_layer_message;

use crate::app::tray_state::TrayClient;
use crate::domain::tray::TrayChange;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    WorkspacesChanged,
    Tick,
    FocusWorkspace(u32),
    ToggleClock,
    Tray(TrayChange),
    Bluetooth(Option<crate::domain::tray::TrayItem>),
    TrayHover(iced::window::Id, crate::domain::reveal::Expansion),
    TrayFrame(std::time::Instant),
    TrayReady(TrayClient),
    TrayActivate(String),
    TrayMenu(String),
    TrayChoose(i32),
    CloseMenu,
    Ignore,
}
