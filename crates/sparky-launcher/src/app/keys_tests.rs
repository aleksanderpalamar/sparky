use iced::Event;
use iced::event::Status;
use iced::keyboard::key::{NativeCode, Physical};
use iced::keyboard::{self, Key, Location, Modifiers};

use crate::app::keys::typed;

fn letra(text: &str, modifiers: Modifiers) -> Event {
    let key = Key::Character(text.into());
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: Some(text.into()),
        repeat: false,
    })
}

#[test]
fn digitar_com_a_busca_sem_foco_vira_texto_da_busca() {
    assert_eq!(
        typed(&letra("f", Modifiers::empty()), Status::Ignored),
        Some("f".to_owned())
    );
    assert_eq!(
        typed(&letra("F", Modifiers::SHIFT), Status::Ignored),
        Some("F".to_owned())
    );
}

#[test]
fn com_a_busca_em_foco_a_letra_ja_vai_para_ela() {
    assert_eq!(
        typed(&letra("f", Modifiers::empty()), Status::Captured),
        None
    );
}

#[test]
fn atalhos_e_caracteres_de_controle_nao_viram_texto() {
    assert_eq!(typed(&letra("j", Modifiers::CTRL), Status::Ignored), None);
    assert_eq!(
        typed(&letra("\t", Modifiers::empty()), Status::Ignored),
        None
    );
    assert_eq!(
        typed(&letra("\r", Modifiers::empty()), Status::Ignored),
        None
    );
}
