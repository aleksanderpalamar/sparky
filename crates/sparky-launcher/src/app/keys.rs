use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::domain::grid::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Move(Move),
    Shelf(isize),
    Menu,
}

pub fn command(key: &Key, modifiers: Modifiers) -> Option<Command> {
    let backwards = modifiers.shift();
    match key.as_ref() {
        Key::Named(Named::Tab) if modifiers.control() => Some(Command::Shelf(sign(backwards))),
        Key::Named(Named::PageDown) if modifiers.control() => Some(Command::Shelf(1)),
        Key::Named(Named::PageUp) if modifiers.control() => Some(Command::Shelf(-1)),
        Key::Named(Named::Tab) if backwards => Some(Command::Move(Move::Previous)),
        Key::Named(Named::Tab | Named::ArrowRight) => Some(Command::Move(Move::Next)),
        Key::Named(Named::ArrowLeft) => Some(Command::Move(Move::Previous)),
        Key::Named(Named::ArrowDown) => Some(Command::Move(Move::Down)),
        Key::Named(Named::ArrowUp) => Some(Command::Move(Move::Up)),
        Key::Named(Named::ContextMenu) => Some(Command::Menu),
        Key::Named(Named::F10) if backwards => Some(Command::Menu),
        Key::Character("n" | "j") if modifiers.control() => Some(Command::Move(Move::Down)),
        Key::Character("p" | "k") if modifiers.control() => Some(Command::Move(Move::Up)),
        _ => None,
    }
}

pub fn confirms(key: &Key) -> bool {
    matches!(key.as_ref(), Key::Named(Named::Enter))
}

fn sign(backwards: bool) -> isize {
    match backwards {
        true => -1,
        false => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, command, confirms};
    use crate::domain::grid::Move;
    use iced::keyboard::key::Named;
    use iced::keyboard::{Key, Modifiers};

    fn tecla(named: Named, modifiers: Modifiers) -> Option<Command> {
        command(&Key::Named(named), modifiers)
    }

    #[test]
    fn setas_e_tab_andam_pela_grade() {
        let nada = Modifiers::empty();
        assert_eq!(
            tecla(Named::ArrowDown, nada),
            Some(Command::Move(Move::Down))
        );
        assert_eq!(tecla(Named::ArrowUp, nada), Some(Command::Move(Move::Up)));
        assert_eq!(
            tecla(Named::ArrowRight, nada),
            Some(Command::Move(Move::Next))
        );
        assert_eq!(
            tecla(Named::ArrowLeft, nada),
            Some(Command::Move(Move::Previous))
        );
        assert_eq!(tecla(Named::Tab, nada), Some(Command::Move(Move::Next)));
        assert_eq!(
            tecla(Named::Tab, Modifiers::SHIFT),
            Some(Command::Move(Move::Previous))
        );
    }

    #[test]
    fn ctrl_tab_e_ctrl_page_trocam_de_prateleira() {
        assert_eq!(tecla(Named::Tab, Modifiers::CTRL), Some(Command::Shelf(1)));
        assert_eq!(
            tecla(Named::Tab, Modifiers::CTRL | Modifiers::SHIFT),
            Some(Command::Shelf(-1))
        );
        assert_eq!(
            tecla(Named::PageDown, Modifiers::CTRL),
            Some(Command::Shelf(1))
        );
        assert_eq!(
            tecla(Named::PageUp, Modifiers::CTRL),
            Some(Command::Shelf(-1))
        );
    }

    #[test]
    fn tecla_de_menu_e_shift_f10_abrem_o_menu() {
        assert_eq!(
            tecla(Named::ContextMenu, Modifiers::empty()),
            Some(Command::Menu)
        );
        assert_eq!(tecla(Named::F10, Modifiers::SHIFT), Some(Command::Menu));
        assert_eq!(tecla(Named::F10, Modifiers::empty()), None);
    }

    #[test]
    fn enter_confirma_a_opcao_escolhida() {
        assert!(confirms(&Key::Named(Named::Enter)));
        assert!(!confirms(&Key::Named(Named::Space)));
        assert!(!confirms(&Key::Character("a".into())));
    }

    #[test]
    fn ctrl_com_n_p_j_k_tambem_andam() {
        assert_eq!(
            command(&Key::Character("j".into()), Modifiers::CTRL),
            Some(Command::Move(Move::Down))
        );
        assert_eq!(
            command(&Key::Character("p".into()), Modifiers::CTRL),
            Some(Command::Move(Move::Up))
        );
        assert_eq!(
            command(&Key::Character("j".into()), Modifiers::empty()),
            None
        );
    }
}
