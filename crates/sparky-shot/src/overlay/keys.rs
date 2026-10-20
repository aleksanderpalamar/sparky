use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

const STEP: i32 = 10;
const FINE: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Nudge(i32, i32),
    Grow(i32, i32),
    Whole,
    Confirm,
    Cancel,
}

pub fn command(key: &Key, modifiers: Modifiers) -> Option<Command> {
    let step = match modifiers.control() {
        true => FINE,
        false => STEP,
    };
    match key {
        Key::Named(Named::Escape) => Some(Command::Cancel),
        Key::Named(Named::Enter) => Some(Command::Confirm),
        Key::Named(named) => arrow(*named, step).map(|(dx, dy)| match modifiers.shift() {
            true => Command::Grow(dx, dy),
            false => Command::Nudge(dx, dy),
        }),
        Key::Character(text) if text.eq_ignore_ascii_case("a") => Some(Command::Whole),
        _ => None,
    }
}

fn arrow(named: Named, step: i32) -> Option<(i32, i32)> {
    match named {
        Named::ArrowLeft => Some((-step, 0)),
        Named::ArrowRight => Some((step, 0)),
        Named::ArrowUp => Some((0, -step)),
        Named::ArrowDown => Some((0, step)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, command};
    use iced::keyboard::key::Named;
    use iced::keyboard::{Key, Modifiers};

    fn nomeada(named: Named) -> Key {
        Key::Named(named)
    }

    #[test]
    fn enter_captura_e_esc_cancela() {
        assert_eq!(command(&nomeada(Named::Enter), Modifiers::empty()), Some(Command::Confirm));
        assert_eq!(command(&nomeada(Named::Escape), Modifiers::empty()), Some(Command::Cancel));
    }

    #[test]
    fn setas_movem_dez_pixels_e_com_ctrl_um() {
        assert_eq!(command(&nomeada(Named::ArrowLeft), Modifiers::empty()), Some(Command::Nudge(-10, 0)));
        assert_eq!(command(&nomeada(Named::ArrowDown), Modifiers::CTRL), Some(Command::Nudge(0, 1)));
    }

    #[test]
    fn shift_com_setas_redimensiona() {
        assert_eq!(command(&nomeada(Named::ArrowRight), Modifiers::SHIFT), Some(Command::Grow(10, 0)));
        assert_eq!(command(&nomeada(Named::ArrowUp), Modifiers::SHIFT | Modifiers::CTRL), Some(Command::Grow(0, -1)));
    }

    #[test]
    fn a_seleciona_a_tela_toda_e_o_resto_e_ignorado() {
        assert_eq!(command(&Key::Character("A".into()), Modifiers::SHIFT), Some(Command::Whole));
        assert_eq!(command(&Key::Character("q".into()), Modifiers::empty()), None);
        assert_eq!(command(&nomeada(Named::Tab), Modifiers::empty()), None);
    }
}
