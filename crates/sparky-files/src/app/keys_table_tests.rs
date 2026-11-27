use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::keys::{Command, browse};
use crate::domain::grid::ViewMode;

const NENHUM: Modifiers = Modifiers::empty();
const CTRL: Modifiers = Modifiers::CTRL;
const ALT: Modifiers = Modifiers::ALT;
const SHIFT: Modifiers = Modifiers::SHIFT;

fn ctrl_shift() -> Modifiers {
    Modifiers::CTRL | Modifiers::SHIFT
}

fn letras() -> Vec<(&'static str, Command)> {
    vec![
        ("j", Command::Down),
        ("k", Command::Up),
        ("l", Command::Open),
        ("h", Command::Parent),
        ("g", Command::First),
        ("G", Command::Last),
        ("~", Command::Home),
        (".", Command::ToggleHidden),
        ("/", Command::Filter),
        ("y", Command::Copy),
        ("x", Command::Cut),
        ("p", Command::Paste),
        ("r", Command::Rename),
        ("a", Command::Create),
        ("d", Command::Trash),
        ("e", Command::Extract),
        ("s", Command::SortNext),
        ("S", Command::SortReverse),
        ("q", Command::Quit),
        ("1", Command::Place(0)),
        ("2", Command::Place(1)),
        ("3", Command::Place(2)),
        ("4", Command::Place(3)),
        ("5", Command::Place(4)),
        ("6", Command::Place(5)),
        ("7", Command::Place(6)),
        ("8", Command::Place(7)),
        ("9", Command::Place(8)),
    ]
}

fn com_ctrl() -> Vec<(&'static str, Command)> {
    vec![
        ("a", Command::SelectAll),
        ("f", Command::Search),
        ("t", Command::NewTab),
        ("w", Command::CloseTab),
        ("c", Command::Copy),
        ("x", Command::Cut),
        ("v", Command::Paste),
        ("h", Command::ToggleHidden),
        ("r", Command::Refresh),
        ("d", Command::Bookmark),
        ("u", Command::PageUp),
        ("q", Command::Quit),
        ("z", Command::Undo),
        ("1", Command::View(ViewMode::Grid)),
        ("2", Command::View(ViewMode::List)),
    ]
}

fn com_alt() -> Vec<(&'static str, Command)> {
    let mut atalhos = vec![
        ("u", Command::UnmountCurrent),
        ("U", Command::UnmountCurrent),
    ];
    let digitos = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
    atalhos.extend(
        digitos
            .into_iter()
            .enumerate()
            .map(|(indice, digito)| (digito, Command::Volume(indice))),
    );
    atalhos
}

fn nomeadas() -> Vec<(Named, Modifiers, Command)> {
    vec![
        (Named::ArrowDown, NENHUM, Command::Down),
        (Named::ArrowUp, NENHUM, Command::Up),
        (Named::ArrowLeft, NENHUM, Command::Left),
        (Named::ArrowRight, NENHUM, Command::Right),
        (Named::PageDown, NENHUM, Command::PageDown),
        (Named::PageUp, NENHUM, Command::PageUp),
        (Named::Home, NENHUM, Command::First),
        (Named::End, NENHUM, Command::Last),
        (Named::Enter, NENHUM, Command::Open),
        (Named::Backspace, NENHUM, Command::Parent),
        (Named::Space, NENHUM, Command::Mark),
        (Named::F2, NENHUM, Command::Rename),
        (Named::F5, NENHUM, Command::Refresh),
        (Named::Delete, NENHUM, Command::Trash),
        (Named::Delete, SHIFT, Command::Delete),
        (Named::ArrowUp, ALT, Command::Parent),
        (Named::Home, ALT, Command::Home),
        (Named::Enter, ALT, Command::Properties),
        (Named::Tab, CTRL, Command::NextTab),
        (Named::Tab, ctrl_shift(), Command::PreviousTab),
        (Named::PageDown, CTRL, Command::NextTab),
        (Named::PageUp, CTRL, Command::PreviousTab),
    ]
}

fn tecla(texto: &str) -> Key {
    Key::Character(texto.into())
}

fn procurar(tabela: &[(&str, Command)], texto: &str) -> Option<Command> {
    tabela
        .iter()
        .find(|(atalho, _)| *atalho == texto)
        .map(|(_, comando)| *comando)
}

fn imprimiveis() -> Vec<String> {
    (0x21u8..=0x7e)
        .map(|byte| char::from(byte).to_string())
        .collect()
}

#[test]
fn cada_atalho_de_letra_mantem_seu_comando() {
    for (texto, esperado) in letras() {
        assert_eq!(browse(&tecla(texto), NENHUM), Some(esperado), "{texto}");
    }
}

#[test]
fn cada_atalho_com_ctrl_mantem_seu_comando() {
    for (texto, esperado) in com_ctrl() {
        assert_eq!(browse(&tecla(texto), CTRL), Some(esperado), "Ctrl+{texto}");
    }
    assert_eq!(browse(&tecla("n"), ctrl_shift()), Some(Command::NewFolder));
}

#[test]
fn cada_atalho_com_alt_mantem_seu_comando() {
    for (texto, esperado) in com_alt() {
        assert_eq!(browse(&tecla(texto), ALT), Some(esperado), "Alt+{texto}");
    }
}

#[test]
fn cada_tecla_nomeada_mantem_seu_comando() {
    for (nome, modificadores, esperado) in nomeadas() {
        assert_eq!(
            browse(&Key::Named(nome), modificadores),
            Some(esperado),
            "{modificadores:?}+{nome:?}"
        );
    }
}

#[test]
fn caracteres_fora_da_tabela_nao_tem_atalho() {
    let (letras, ctrl, alt) = (letras(), com_ctrl(), com_alt());
    for texto in imprimiveis() {
        let minuscula = texto.to_lowercase();
        assert_eq!(
            browse(&tecla(&texto), NENHUM),
            procurar(&letras, &texto),
            "{texto}"
        );
        assert_eq!(
            browse(&tecla(&texto), CTRL),
            procurar(&ctrl, &minuscula),
            "Ctrl+{texto}"
        );
        assert_eq!(
            browse(&tecla(&texto), ALT),
            procurar(&alt, &texto),
            "Alt+{texto}"
        );
    }
}

#[test]
fn teclas_nomeadas_sem_funcao_continuam_livres() {
    for nome in [
        Named::Escape,
        Named::Tab,
        Named::Insert,
        Named::F1,
        Named::F3,
        Named::F4,
        Named::F6,
        Named::F10,
        Named::F11,
        Named::F12,
    ] {
        assert_eq!(browse(&Key::Named(nome), NENHUM), None, "{nome:?}");
    }
}
