use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::keys::{Cursor, Pressed, Scroll, pressed};

const NADA: Modifiers = Modifiers::empty();

fn nomeada(nome: Named, mods: Modifiers) -> Pressed {
    pressed(&Key::Named(nome), mods, None, Cursor::Normal)
}

fn letra(letra: &str, mods: Modifiers, texto: Option<&str>) -> Pressed {
    pressed(&Key::Character(letra.into()), mods, texto, Cursor::Normal)
}

fn envia(bytes: &[u8]) -> Pressed {
    Pressed::Send(bytes.to_vec())
}

#[test]
fn copiar_e_colar_como_no_kitty() {
    assert_eq!(
        letra("C", Modifiers::CTRL | Modifiers::SHIFT, None),
        Pressed::Copy
    );
    assert_eq!(
        letra("v", Modifiers::CTRL | Modifiers::SHIFT, None),
        Pressed::Paste
    );
    assert_eq!(nomeada(Named::Insert, Modifiers::CTRL), Pressed::Copy);
    assert_eq!(nomeada(Named::Insert, Modifiers::SHIFT), Pressed::Paste);
}

#[test]
fn shift_com_paginas_rola_o_historico() {
    assert_eq!(
        nomeada(Named::PageUp, Modifiers::SHIFT),
        Pressed::Scroll(Scroll::PageUp)
    );
    assert_eq!(
        nomeada(Named::PageDown, Modifiers::SHIFT),
        Pressed::Scroll(Scroll::PageDown)
    );
    assert_eq!(
        nomeada(Named::Home, Modifiers::SHIFT),
        Pressed::Scroll(Scroll::Top)
    );
    assert_eq!(
        nomeada(Named::End, Modifiers::SHIFT),
        Pressed::Scroll(Scroll::Bottom)
    );
}

#[test]
fn enter_e_as_variacoes_do_kitty_do_usuario() {
    assert_eq!(nomeada(Named::Enter, NADA), envia(b"\r"));
    assert_eq!(
        nomeada(Named::Enter, Modifiers::SHIFT),
        envia(b"\x1b[13;2u")
    );
    assert_eq!(
        nomeada(Named::Enter, Modifiers::ALT | Modifiers::SHIFT),
        envia(b"\x1b[13;4u")
    );
    assert_eq!(nomeada(Named::Enter, Modifiers::ALT), envia(b"\x1b\r"));
}

#[test]
fn apagar_tab_e_esc() {
    assert_eq!(nomeada(Named::Backspace, NADA), envia(b"\x7f"));
    assert_eq!(nomeada(Named::Backspace, Modifiers::CTRL), envia(b"\x08"));
    assert_eq!(
        nomeada(Named::Backspace, Modifiers::ALT),
        envia(b"\x1b\x7f")
    );
    assert_eq!(nomeada(Named::Tab, NADA), envia(b"\t"));
    assert_eq!(nomeada(Named::Tab, Modifiers::SHIFT), envia(b"\x1b[Z"));
    assert_eq!(nomeada(Named::Escape, NADA), envia(b"\x1b"));
}

#[test]
fn setas_seguem_o_modo_do_cursor_e_os_modificadores() {
    assert_eq!(nomeada(Named::ArrowUp, NADA), envia(b"\x1b[A"));
    assert_eq!(
        pressed(&Key::Named(Named::ArrowUp), NADA, None, Cursor::Application),
        envia(b"\x1bOA")
    );
    assert_eq!(
        nomeada(Named::ArrowLeft, Modifiers::CTRL),
        envia(b"\x1b[1;5D")
    );
    assert_eq!(
        nomeada(Named::ArrowRight, Modifiers::SHIFT),
        envia(b"\x1b[1;2C")
    );
    assert_eq!(nomeada(Named::Home, NADA), envia(b"\x1b[H"));
    assert_eq!(
        pressed(&Key::Named(Named::End), NADA, None, Cursor::Application),
        envia(b"\x1bOF")
    );
}

#[test]
fn teclas_de_edicao_e_de_funcao() {
    assert_eq!(nomeada(Named::Delete, NADA), envia(b"\x1b[3~"));
    assert_eq!(nomeada(Named::Delete, Modifiers::CTRL), envia(b"\x1b[3;5~"));
    assert_eq!(nomeada(Named::PageDown, NADA), envia(b"\x1b[6~"));
    assert_eq!(nomeada(Named::F1, NADA), envia(b"\x1bOP"));
    assert_eq!(nomeada(Named::F1, Modifiers::CTRL), envia(b"\x1b[1;5P"));
    assert_eq!(nomeada(Named::F5, NADA), envia(b"\x1b[15~"));
    assert_eq!(nomeada(Named::F5, Modifiers::SHIFT), envia(b"\x1b[15;2~"));
    assert_eq!(nomeada(Named::F12, NADA), envia(b"\x1b[24~"));
}

#[test]
fn ctrl_com_letra_vira_caractere_de_controle() {
    assert_eq!(letra("c", Modifiers::CTRL, Some("c")), envia(&[0x03]));
    assert_eq!(letra("a", Modifiers::CTRL, None), envia(&[0x01]));
    assert_eq!(letra("[", Modifiers::CTRL, None), envia(&[0x1b]));
    assert_eq!(letra("/", Modifiers::CTRL, None), envia(&[0x1f]));
    assert_eq!(nomeada(Named::Space, Modifiers::CTRL), envia(&[0x00]));
}

#[test]
fn texto_digitado_e_alt_como_prefixo_esc() {
    assert_eq!(letra("a", NADA, Some("a")), envia(b"a"));
    assert_eq!(letra("a", Modifiers::SHIFT, Some("A")), envia(b"A"));
    assert_eq!(letra("c", NADA, Some("ç")), envia("ç".as_bytes()));
    assert_eq!(letra("x", Modifiers::ALT, Some("x")), envia(b"\x1bx"));
    assert_eq!(nomeada(Named::Space, NADA), envia(b" "));
}

#[test]
fn modificadores_sozinhos_nao_enviam_nada() {
    assert_eq!(nomeada(Named::Shift, Modifiers::SHIFT), Pressed::Ignored);
    assert_eq!(nomeada(Named::Control, Modifiers::CTRL), Pressed::Ignored);
}
