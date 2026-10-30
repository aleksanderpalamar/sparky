use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::keys::message;
use super::message::{Message, Move};
use super::page::Page;
use crate::domain::table::Column;
use crate::ports::Stop;

fn letra(texto: &str) -> Option<Message> {
    message(&Key::Character(texto.into()), Modifiers::empty())
}

fn descreve(message: Option<Message>) -> String {
    format!("{message:?}")
}

#[test]
fn numeros_abrem_paginas_e_letras_ordenam() {
    assert_eq!(
        descreve(letra("1")),
        descreve(Some(Message::Open(Page::Dashboard)))
    );
    assert_eq!(
        descreve(letra("6")),
        descreve(Some(Message::Open(Page::Disk)))
    );
    assert_eq!(
        descreve(letra("8")),
        descreve(Some(Message::Open(Page::Gpu)))
    );
    assert!(letra("9").is_none() && letra("0").is_none());
    assert_eq!(
        descreve(letra("c")),
        descreve(Some(Message::SortBy(Column::Cpu)))
    );
    assert_eq!(
        descreve(letra("v")),
        descreve(Some(Message::SortBy(Column::Vram)))
    );
    assert_eq!(
        descreve(letra("j")),
        descreve(Some(Message::Move(Move::Down)))
    );
}

#[test]
fn delete_encerra_e_shift_delete_forca() {
    let delete = Key::Named(Named::Delete);
    assert_eq!(
        descreve(message(&delete, Modifiers::empty())),
        descreve(Some(Message::Ask(Stop::Terminate)))
    );
    assert_eq!(
        descreve(message(&delete, Modifiers::SHIFT)),
        descreve(Some(Message::Ask(Stop::Kill)))
    );
    assert_eq!(
        descreve(message(&Key::Named(Named::Tab), Modifiers::empty())),
        descreve(Some(Message::SwitchFocus))
    );
}
