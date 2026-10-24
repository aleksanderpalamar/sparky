use iced::mouse::{Button, Event};

use super::message::Message;
use super::pointer::{Pointer, update};
use crate::domain::grid::{Half, Spot};
use crate::domain::tabs::TabId;

#[test]
fn soltar_a_selecao_publica_a_selecao_primaria() {
    let id = TabId(0);
    let mut pointer = Pointer::Selecting(id);
    assert!(matches!(
        update(&mut pointer, id, &Event::ButtonReleased(Button::Left), None, false, 0.0),
        Some(Message::Selected(found)) if found == id
    ));
    assert_eq!(pointer, Pointer::Idle);
}

#[test]
fn clicar_na_rodinha_cola_a_selecao_primaria_somente_dentro_do_terminal() {
    let mut pointer = Pointer::Idle;
    let id = TabId(0);
    let event = Event::ButtonPressed(Button::Middle);
    assert!(matches!(
        update(&mut pointer, id, &event, None, true, 0.0),
        Some(Message::PastePrimary(found)) if found == id
    ));
    assert!(update(&mut pointer, id, &event, None, false, 0.0).is_none());
}

#[test]
fn trocar_a_aba_nao_continua_o_arrasto_da_anterior() {
    let mut pointer = Pointer::Selecting(TabId(0));
    let event = Event::CursorMoved {
        position: iced::Point::ORIGIN,
    };
    let spot = Spot {
        line: 0,
        column: 0,
        half: Half::Left,
    };
    assert!(update(&mut pointer, TabId(1), &event, Some(spot), true, 0.0).is_none());
    assert_eq!(pointer, Pointer::Idle);
}
