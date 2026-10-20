use iced::Task;

use crate::app::message::Message;
use crate::app::state::Bar;

pub fn update(bar: &mut Bar, message: Message) -> Task<Message> {
    match message {
        Message::WorkspacesChanged => bar.refresh_workspaces(),
        Message::Tick => bar.tick(chrono::Local::now().naive_local()),
        Message::FocusWorkspace(id) => report(bar.focus(id)),
        Message::ToggleClock => bar.toggle_clock(),
        _ => {}
    }
    Task::none()
}

fn report(result: Result<(), sparky_ipc::IpcError>) {
    if let Err(error) = result {
        eprintln!("sparky-bar: o sparky recusou trocar de área: {error}");
    }
}
