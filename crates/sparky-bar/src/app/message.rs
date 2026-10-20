use iced_layershell::to_layer_message;

use crate::app::tray_state::TrayClient;
use crate::domain::tray::TrayChange;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    WorkspacesChanged,
    Tick,
    FocusWorkspace(u32),
    ToggleClock,
    Tray(TrayChange),
    TrayReady(TrayClient),
    TrayActivate(String),
    TrayMenu(String),
    TrayChoose(i32),
    CloseMenu,
    Ignore,
}
