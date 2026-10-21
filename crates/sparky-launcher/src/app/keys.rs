use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

pub fn movement(key: &Key, modifiers: Modifiers) -> Option<isize> {
    match key.as_ref() {
        Key::Named(Named::ArrowDown) => Some(1),
        Key::Named(Named::ArrowUp) => Some(-1),
        Key::Named(Named::Tab) if modifiers.shift() => Some(-1),
        Key::Named(Named::Tab) => Some(1),
        Key::Character("n" | "j") if modifiers.control() => Some(1),
        Key::Character("p" | "k") if modifiers.control() => Some(-1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::movement;
    use iced::keyboard::key::Named;
    use iced::keyboard::{Key, Modifiers};

    #[test]
    fn setas_e_tab_andam_pela_lista() {
        assert_eq!(movement(&Key::Named(Named::ArrowDown), Modifiers::empty()), Some(1));
        assert_eq!(movement(&Key::Named(Named::ArrowUp), Modifiers::empty()), Some(-1));
        assert_eq!(movement(&Key::Named(Named::Tab), Modifiers::empty()), Some(1));
        assert_eq!(movement(&Key::Named(Named::Tab), Modifiers::SHIFT), Some(-1));
    }

    #[test]
    fn ctrl_com_n_p_j_k_tambem_andam() {
        assert_eq!(movement(&Key::Character("j".into()), Modifiers::CTRL), Some(1));
        assert_eq!(movement(&Key::Character("p".into()), Modifiers::CTRL), Some(-1));
        assert_eq!(movement(&Key::Character("j".into()), Modifiers::empty()), None);
    }
}
