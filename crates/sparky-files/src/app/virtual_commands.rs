use iced::Task;

use crate::app::keys::Command;
use crate::app::message::Message;
use crate::app::state::Files;
use crate::app::volumes;

pub(super) fn virtual_command(files: &mut Files, command: Command) -> Task<Message> {
    let last = files.sidebar_items.len().saturating_sub(1);
    match command {
        Command::Down => files.sidebar_cursor = (files.sidebar_cursor + 1).min(last),
        Command::Up => files.sidebar_cursor = files.sidebar_cursor.saturating_sub(1),
        Command::First => files.sidebar_cursor = 0,
        Command::Last => files.sidebar_cursor = last,
        Command::Open => return files.open_sidebar_item(files.sidebar_cursor),
        Command::Parent if files.network_uri.is_some() => return files.network_parent(),
        Command::Parent => files.parent(),
        Command::Home => files.home(),
        Command::Place(index) => files.place(index),
        Command::Volume(index) => {
            let device = files.volumes.get(index).map(|volume| volume.path.clone());
            return device.map_or_else(Task::none, |device| volumes::open(files, &device));
        }
        Command::UnmountCurrent => return volumes::unmount_current(files),
        Command::Refresh => {
            if let Some(uri) = files.network_uri.clone() {
                return files.load_network(uri);
            }
            files.refresh();
        }
        Command::Quit => return iced::exit(),
        Command::Undo => return files.undo(),
        _ => {}
    }
    Task::none()
}
