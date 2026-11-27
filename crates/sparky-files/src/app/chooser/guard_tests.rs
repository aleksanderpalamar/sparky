use std::path::PathBuf;

use iced::keyboard::Modifiers;

use super::guard::{allowed, blocked};
use crate::app::keys::Command;
use crate::app::message::{Message, TrashEvent};
use crate::domain::context::Action;
use crate::domain::drop::DropTarget;
use crate::domain::grid::ViewMode;
use crate::domain::order::SortKey;
use crate::domain::sidebar_location::SidebarLocation;

fn liberados() -> Vec<Command> {
    vec![
        Command::Down,
        Command::Up,
        Command::Left,
        Command::Right,
        Command::PageDown,
        Command::PageUp,
        Command::First,
        Command::Last,
        Command::Parent,
        Command::Home,
        Command::Place(0),
        Command::ToggleHidden,
        Command::Filter,
        Command::Mark,
        Command::SelectAll,
        Command::SortNext,
        Command::SortReverse,
        Command::Refresh,
        Command::View(ViewMode::Grid),
        Command::View(ViewMode::List),
        Command::NewFolder,
        Command::Volume(0),
    ]
}

fn proibidos() -> Vec<Command> {
    vec![
        Command::Open,
        Command::Copy,
        Command::Cut,
        Command::Paste,
        Command::Rename,
        Command::Create,
        Command::Properties,
        Command::Trash,
        Command::Delete,
        Command::Extract,
        Command::Search,
        Command::NewTab,
        Command::CloseTab,
        Command::NextTab,
        Command::PreviousTab,
        Command::Undo,
        Command::Quit,
        Command::UnmountCurrent,
        Command::Bookmark,
    ]
}

fn bloqueadas() -> Vec<Message> {
    let caminho = String::from("/media/pendrive");
    vec![
        Message::ContextEntry(0),
        Message::ContextBackground,
        Message::ContextBookmark(0),
        Message::ContextAction(Action::Trash),
        Message::DeleteConfirm,
        Message::Trash(TrashEvent::Empty),
        Message::Hover(DropTarget::Trash),
        Message::OpenWithChoice(String::from("app")),
        Message::OpenWithConfirm,
        Message::Tab(0),
        Message::CloseTab(0),
        Message::FavoriteSelected,
        Message::RemoveFavorite(0),
        Message::Sidebar(SidebarLocation::Recent),
        Message::SidebarItem(0),
        Message::UnmountVolume(caminho.clone()),
        Message::EjectVolume(caminho),
        Message::ToastUndo,
    ]
}

fn repassadas() -> Vec<Message> {
    vec![
        Message::Clicked(0),
        Message::Scrolled(1),
        Message::Typed(String::from("nome")),
        Message::Submit,
        Message::Place(0),
        Message::View(ViewMode::List),
        Message::Sort(SortKey::Name),
        Message::OpenVolume(String::from("/media/pendrive")),
        Message::Modifiers(Modifiers::empty()),
        Message::ArchiveCompleted(PathBuf::from("/tmp/a.zip"), Ok(())),
    ]
}

#[test]
fn seletor_libera_somente_navegacao_selecao_e_nova_pasta() {
    for comando in liberados() {
        assert!(allowed(comando), "{comando:?}");
    }
    for comando in proibidos() {
        assert!(!allowed(comando), "{comando:?}");
    }
}

#[test]
fn seletor_bloqueia_menus_lixeira_abas_e_dispositivos() {
    for mensagem in bloqueadas() {
        assert!(blocked(&mensagem), "{mensagem:?}");
    }
    for mensagem in repassadas() {
        assert!(!blocked(&mensagem), "{mensagem:?}");
    }
}
