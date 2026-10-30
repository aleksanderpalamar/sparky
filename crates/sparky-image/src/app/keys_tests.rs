use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::keys::{Command, command};
use crate::domain::album::{Edge, Step};
use crate::domain::turn::Spin;
use crate::domain::viewport::Direction;

fn letra(texto: &str) -> Option<Command> {
    command(&Key::Character(texto.into()), Modifiers::empty())
}

fn nomeada(tecla: Named) -> Option<Command> {
    command(&Key::Named(tecla), Modifiers::empty())
}

#[test]
fn setas_e_hjkl_movem_a_visao() {
    assert_eq!(
        nomeada(Named::ArrowLeft),
        Some(Command::Arrow(Direction::Left))
    );
    assert_eq!(
        nomeada(Named::ArrowDown),
        Some(Command::Arrow(Direction::Down))
    );
    assert_eq!(letra("l"), Some(Command::Arrow(Direction::Right)));
    assert_eq!(letra("k"), Some(Command::Arrow(Direction::Up)));
}

#[test]
fn espaco_backspace_e_paginas_trocam_de_imagem() {
    assert_eq!(nomeada(Named::Space), Some(Command::Step(Step::Next)));
    assert_eq!(nomeada(Named::PageDown), Some(Command::Step(Step::Next)));
    assert_eq!(
        nomeada(Named::Backspace),
        Some(Command::Step(Step::Previous))
    );
    assert_eq!(nomeada(Named::PageUp), Some(Command::Step(Step::Previous)));
    assert_eq!(nomeada(Named::Home), Some(Command::Jump(Edge::First)));
    assert_eq!(nomeada(Named::End), Some(Command::Jump(Edge::Last)));
}

#[test]
fn zoom_ajuste_tamanho_real_rotacao_e_saida() {
    assert_eq!(letra("+"), Some(Command::ZoomIn));
    assert_eq!(letra("="), Some(Command::ZoomIn));
    assert_eq!(letra("-"), Some(Command::ZoomOut));
    assert_eq!(letra("0"), Some(Command::Fit));
    assert_eq!(letra("1"), Some(Command::ActualSize));
    assert_eq!(letra("r"), Some(Command::Rotate(Spin::Clockwise)));
    assert_eq!(letra("R"), Some(Command::Rotate(Spin::CounterClockwise)));
    assert_eq!(
        command(&Key::Character("r".into()), Modifiers::SHIFT),
        Some(Command::Rotate(Spin::CounterClockwise))
    );
    assert_eq!(letra("q"), Some(Command::Quit));
    assert_eq!(letra("x"), None);
}
