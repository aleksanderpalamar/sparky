use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::keys::{Command, browse};
use crate::domain::context::{Action, Row, Target, entries};

fn comando(action: Action) -> Option<Command> {
    match action {
        Action::Open => Some(Command::Open),
        Action::Cut => Some(Command::Cut),
        Action::Copy => Some(Command::Copy),
        Action::Rename => Some(Command::Rename),
        Action::Extract => Some(Command::Extract),
        Action::Trash => Some(Command::Trash),
        Action::Delete => Some(Command::Delete),
        Action::Properties => Some(Command::Properties),
        Action::NewFolder => Some(Command::NewFolder),
        Action::Paste => Some(Command::Paste),
        Action::SelectAll => Some(Command::SelectAll),
        _ => None,
    }
}

fn modificador(nome: &str) -> Modifiers {
    match nome {
        "Ctrl" => Modifiers::CTRL,
        "Shift" => Modifiers::SHIFT,
        "Alt" => Modifiers::ALT,
        _ => Modifiers::empty(),
    }
}

fn tecla(nome: &str) -> Key {
    match nome {
        "Enter" => Key::Named(Named::Enter),
        "Delete" => Key::Named(Named::Delete),
        "F2" => Key::Named(Named::F2),
        letra => Key::Character(letra.to_lowercase().into()),
    }
}

fn pressionar(atalho: &str) -> Option<Command> {
    let partes: Vec<_> = atalho.split('+').collect();
    let (nome, modificadores) = partes.split_last()?;
    let modificadores = modificadores
        .iter()
        .fold(Modifiers::empty(), |soma, parte| soma | modificador(parte));
    browse(&tecla(nome), modificadores)
}

fn atalhos_exibidos() -> Vec<(Action, &'static str)> {
    [
        Target::File,
        Target::Archive,
        Target::Folder,
        Target::Background,
        Target::Trashed,
        Target::TrashBackground,
        Target::Bookmark(0),
    ]
    .into_iter()
    .flat_map(|target| entries(target, true, true))
    .filter_map(|row| match row {
        Row::Item {
            action, shortcut, ..
        } if !shortcut.is_empty() => Some((action, shortcut)),
        _ => None,
    })
    .collect()
}

#[test]
fn todo_atalho_exibido_no_menu_executa_a_mesma_acao() {
    let atalhos = atalhos_exibidos();
    assert!(!atalhos.is_empty());
    for (action, atalho) in atalhos {
        let esperado = comando(action);
        assert!(esperado.is_some(), "{action:?} exibe {atalho} sem comando");
        assert_eq!(pressionar(atalho), esperado, "{action:?}: {atalho}");
    }
}
