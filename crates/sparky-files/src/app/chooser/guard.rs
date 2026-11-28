use crate::app::keys::Command;
use crate::app::message::Message;

pub fn allowed(command: Command) -> bool {
    matches!(
        command,
        Command::Down
            | Command::Up
            | Command::Left
            | Command::Right
            | Command::PageDown
            | Command::PageUp
            | Command::First
            | Command::Last
            | Command::Parent
            | Command::Home
            | Command::Back
            | Command::Forward
            | Command::EditLocation
            | Command::Place(_)
            | Command::ToggleHidden
            | Command::Filter
            | Command::Mark
            | Command::SelectAll
            | Command::SortNext
            | Command::SortReverse
            | Command::Refresh
            | Command::View(_)
            | Command::NewFolder
            | Command::Volume(_)
    )
}

pub fn blocked(message: &Message) -> bool {
    matches!(
        message,
        Message::ContextEntry(_)
            | Message::ContextBackground
            | Message::ContextBookmark(_)
            | Message::ContextAction(_)
            | Message::DeleteConfirm
            | Message::Trash(_)
            | Message::Hover(_)
            | Message::OpenWithChoice(_)
            | Message::OpenWithConfirm
            | Message::Tab(_)
            | Message::CloseTab(_)
            | Message::FavoriteSelected
            | Message::RemoveFavorite(_)
            | Message::Sidebar(_)
            | Message::SidebarItem(_)
            | Message::UnmountVolume(_)
            | Message::EjectVolume(_)
            | Message::ToastUndo
    )
}
