use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Down,
    Up,
    Less,
    More,
    Activate,
    Remove,
    NextSection,
    PreviousSection,
    Section(usize),
    Refresh,
    Quit,
}

pub fn browse(key: &Key, modifiers: Modifiers) -> Option<Command> {
    match key.as_ref() {
        Key::Named(Named::Tab) if modifiers.shift() => Some(Command::PreviousSection),
        Key::Named(named) => named_command(named),
        Key::Character(text) if modifiers.control() => control_command(text),
        Key::Character(text) => letter_command(text),
        Key::Unidentified => None,
    }
}

fn named_command(named: Named) -> Option<Command> {
    match named {
        Named::ArrowDown => Some(Command::Down),
        Named::ArrowUp => Some(Command::Up),
        Named::ArrowLeft => Some(Command::Less),
        Named::ArrowRight => Some(Command::More),
        Named::Enter | Named::Space => Some(Command::Activate),
        Named::Delete => Some(Command::Remove),
        Named::Tab => Some(Command::NextSection),
        Named::F5 => Some(Command::Refresh),
        _ => None,
    }
}

fn control_command(text: &str) -> Option<Command> {
    match text.to_lowercase().as_str() {
        "q" => Some(Command::Quit),
        "r" => Some(Command::Refresh),
        _ => None,
    }
}

fn letter_command(text: &str) -> Option<Command> {
    match text {
        "j" => Some(Command::Down),
        "k" => Some(Command::Up),
        "h" | "-" => Some(Command::Less),
        "l" | "+" | "=" => Some(Command::More),
        "d" => Some(Command::Remove),
        "q" => Some(Command::Quit),
        digit => {
            let number = digit.parse::<usize>().ok()?;
            (1..=9).contains(&number).then(|| Command::Section(number - 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, browse};
    use iced::keyboard::key::Named;
    use iced::keyboard::{Key, Modifiers};

    fn letra(text: &str) -> Option<Command> {
        browse(&Key::Character(text.into()), Modifiers::empty())
    }

    #[test]
    fn jk_movem_e_hl_ajustam() {
        assert_eq!(letra("j"), Some(Command::Down));
        assert_eq!(letra("k"), Some(Command::Up));
        assert_eq!(letra("h"), Some(Command::Less));
        assert_eq!(letra("l"), Some(Command::More));
        assert_eq!(letra("+"), Some(Command::More));
        assert_eq!(browse(&Key::Named(Named::ArrowLeft), Modifiers::empty()), Some(Command::Less));
    }

    #[test]
    fn tab_e_shift_tab_trocam_de_secao() {
        assert_eq!(browse(&Key::Named(Named::Tab), Modifiers::empty()), Some(Command::NextSection));
        assert_eq!(browse(&Key::Named(Named::Tab), Modifiers::SHIFT), Some(Command::PreviousSection));
        assert_eq!(letra("3"), Some(Command::Section(2)));
        assert_eq!(letra("0"), None);
    }

    #[test]
    fn enter_e_espaco_ativam_e_d_remove() {
        assert_eq!(browse(&Key::Named(Named::Enter), Modifiers::empty()), Some(Command::Activate));
        assert_eq!(browse(&Key::Named(Named::Space), Modifiers::empty()), Some(Command::Activate));
        assert_eq!(letra("d"), Some(Command::Remove));
    }

    #[test]
    fn q_fecha_e_f5_rele() {
        assert_eq!(letra("q"), Some(Command::Quit));
        assert_eq!(browse(&Key::Character("Q".into()), Modifiers::CTRL), Some(Command::Quit));
        assert_eq!(browse(&Key::Named(Named::F5), Modifiers::empty()), Some(Command::Refresh));
        assert_eq!(letra("z"), None);
    }
}
