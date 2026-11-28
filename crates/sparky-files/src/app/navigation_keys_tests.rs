use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::keys::{Command, browse, locating};

fn letra(text: &str) -> Option<Command> {
    browse(&Key::Character(text.into()), Modifiers::empty())
}

#[test]
fn alt_e_setas_navegam_no_historico_sem_mudar_backspace_e_espaco() {
    let named = |key, modifiers| browse(&Key::Named(key), modifiers);
    assert_eq!(named(Named::ArrowLeft, Modifiers::ALT), Some(Command::Back));
    assert_eq!(
        named(Named::ArrowRight, Modifiers::ALT),
        Some(Command::Forward)
    );
    assert_eq!(
        named(Named::ArrowLeft, Modifiers::empty()),
        Some(Command::Left)
    );
    assert_eq!(
        named(Named::Backspace, Modifiers::empty()),
        Some(Command::Parent)
    );
    assert_eq!(named(Named::Space, Modifiers::empty()), Some(Command::Mark));
}

#[test]
fn ctrl_l_edita_o_local_e_tab_completa_so_no_campo_do_local() {
    assert_eq!(
        browse(&Key::Character("l".into()), Modifiers::CTRL),
        Some(Command::EditLocation)
    );
    assert_eq!(letra("l"), Some(Command::Open));
    let tab = |modifiers| locating(&Key::Named(Named::Tab), modifiers);
    assert_eq!(tab(Modifiers::empty()), Some(Command::Complete(true)));
    assert_eq!(tab(Modifiers::SHIFT), Some(Command::Complete(false)));
}
