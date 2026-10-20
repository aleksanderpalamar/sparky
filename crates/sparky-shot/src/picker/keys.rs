use iced::keyboard::Key;
use iced::keyboard::key::Named;

use crate::domain::mode::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Pick(Mode),
    NextDelay,
    Cancel,
}

pub fn command(key: &Key) -> Option<Command> {
    match key {
        Key::Named(Named::Escape) => Some(Command::Cancel),
        Key::Named(Named::Tab) => Some(Command::NextDelay),
        Key::Character(text) => letter(text).map(Command::Pick),
        _ => None,
    }
}

pub fn hint(mode: Mode) -> &'static str {
    match mode {
        Mode::Region => "R",
        Mode::Window => "J",
        Mode::Output => "M",
    }
}

fn letter(text: &str) -> Option<Mode> {
    [Mode::Region, Mode::Window, Mode::Output]
        .into_iter()
        .find(|mode| text.eq_ignore_ascii_case(hint(*mode)))
}

#[cfg(test)]
mod tests {
    use super::{Command, command, hint};
    use crate::domain::mode::Mode;
    use iced::keyboard::Key;
    use iced::keyboard::key::Named;

    fn letra(text: &str) -> Option<Command> {
        command(&Key::Character(text.into()))
    }

    #[test]
    fn cada_modo_tem_a_sua_letra_maiuscula_ou_minuscula() {
        assert_eq!(letra("r"), Some(Command::Pick(Mode::Region)));
        assert_eq!(letra("J"), Some(Command::Pick(Mode::Window)));
        assert_eq!(letra("m"), Some(Command::Pick(Mode::Output)));
    }

    #[test]
    fn tab_troca_o_atraso_e_esc_cancela() {
        assert_eq!(command(&Key::Named(Named::Tab)), Some(Command::NextDelay));
        assert_eq!(command(&Key::Named(Named::Escape)), Some(Command::Cancel));
    }

    #[test]
    fn outras_teclas_sao_ignoradas() {
        assert_eq!(letra("x"), None);
        assert_eq!(command(&Key::Named(Named::Enter)), None);
    }

    #[test]
    fn a_dica_mostrada_e_a_letra_aceita() {
        for mode in [Mode::Region, Mode::Window, Mode::Output] {
            assert_eq!(letra(hint(mode)), Some(Command::Pick(mode)));
        }
    }
}
