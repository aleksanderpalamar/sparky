use iced::keyboard::{Key, Modifiers, key::Named};

use super::keys::{Cursor, Pressed, pressed};
use crate::domain::tabs::TabAction;

const MODS: Modifiers = Modifiers::CTRL.union(Modifiers::SHIFT);

#[test]
fn atalhos_de_abas_sao_consumidos_antes_de_chegar_ao_programa_no_pty() {
    let cases = [
        (Key::Character("t".into()), TabAction::New),
        (Key::Character("Q".into()), TabAction::Close),
        (Key::Named(Named::ArrowRight), TabAction::Cycle(1)),
        (Key::Named(Named::ArrowLeft), TabAction::Cycle(-1)),
    ];
    for (key, action) in cases {
        assert_eq!(
            pressed(&key, MODS, None, None, Cursor::Application),
            Pressed::Tab(action)
        );
    }
}

#[test]
fn sem_shift_as_teclas_continuam_chegando_ao_programa() {
    assert_eq!(
        pressed(
            &Key::Character("t".into()),
            Modifiers::CTRL,
            None,
            None,
            Cursor::Normal
        ),
        Pressed::Send(vec![0x14])
    );
    assert_eq!(
        pressed(
            &Key::Character("c".into()),
            Modifiers::CTRL,
            None,
            None,
            Cursor::Normal
        ),
        Pressed::Send(vec![0x03])
    );
}

#[test]
fn alt_e_super_nao_acionam_os_atalhos_de_abas_por_engano() {
    for extra in [Modifiers::ALT, Modifiers::LOGO] {
        assert!(matches!(
            pressed(
                &Key::Character("t".into()),
                MODS | extra,
                None,
                None,
                Cursor::Normal
            ),
            Pressed::Send(_)
        ));
    }
}
