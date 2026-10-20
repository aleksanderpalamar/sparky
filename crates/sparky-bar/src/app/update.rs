use iced::Task;

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::app::tray_actions;

pub fn update(bar: &mut Bar, message: Message) -> Task<Message> {
    match message {
        Message::WorkspacesChanged => bar.refresh_workspaces(),
        Message::Tick => bar.tick(chrono::Local::now().naive_local()),
        Message::FocusWorkspace(id) => report(bar.focus(id)),
        Message::ToggleClock => bar.toggle_clock(),
        Message::Tray(change) => bar.tray_mut().apply(change),
        Message::TrayReady(client) => bar.tray_mut().connect(client),
        Message::TrayActivate(address) => return tray_actions::activate(bar, address),
        Message::TrayMenu(address) => return tray_actions::open_menu(bar, address),
        Message::TrayChoose(entry) => return tray_actions::choose(bar, entry),
        Message::CloseMenu => return tray_actions::close_menu(bar),
        _ => {}
    }
    Task::none()
}

fn report(result: Result<(), sparky_ipc::IpcError>) {
    if let Err(error) = result {
        eprintln!("sparky-bar: o sparky recusou trocar de área: {error}");
    }
}
