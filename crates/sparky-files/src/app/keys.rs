use crate::domain::grid::ViewMode;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Down,
    Up,
    Left,
    Right,
    PageDown,
    PageUp,
    First,
    Last,
    Open,
    Parent,
    Home,
    Place(usize),
    ToggleHidden,
    Filter,
    Mark,
    Copy,
    Cut,
    Paste,
    Rename,
    Create,
    NewFolder,
    SelectAll,
    Properties,
    Trash,
    Refresh,
    Quit,
    View(ViewMode),
    Volume(usize),
    UnmountCurrent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
}

pub fn browse(key: &Key, modifiers: Modifiers) -> Option<Command> {
    match key.as_ref() {
        Key::Named(named) => named_command(named, modifiers),
        Key::Character(text) if modifiers.alt() => volume_command(text),
        Key::Character(text) if modifiers.control() => control_command(text, modifiers),
        Key::Character(text) => letter_command(text),
        Key::Unidentified => None,
    }
}

pub fn filtering(key: &Key) -> Option<Command> {
    match key.as_ref() {
        Key::Named(Named::ArrowDown) => Some(Command::Down),
        Key::Named(Named::ArrowUp) => Some(Command::Up),
        Key::Named(Named::PageDown) => Some(Command::PageDown),
        Key::Named(Named::PageUp) => Some(Command::PageUp),
        _ => None,
    }
}

pub fn confirm(key: &Key) -> Option<Answer> {
    match key.as_ref() {
        Key::Named(Named::Enter) => Some(Answer::Yes),
        Key::Character("s" | "S" | "y" | "Y") => Some(Answer::Yes),
        Key::Character("n" | "N") => Some(Answer::No),
        _ => None,
    }
}

fn named_command(named: Named, modifiers: Modifiers) -> Option<Command> {
    match named {
        Named::ArrowUp if modifiers.alt() => Some(Command::Parent),
        Named::Home if modifiers.alt() => Some(Command::Home),
        Named::Enter if modifiers.alt() => Some(Command::Properties),
        Named::ArrowDown => Some(Command::Down),
        Named::ArrowUp => Some(Command::Up),
        Named::PageDown => Some(Command::PageDown),
        Named::PageUp => Some(Command::PageUp),
        Named::Home => Some(Command::First),
        Named::End => Some(Command::Last),
        Named::Enter => Some(Command::Open),
        Named::ArrowRight => Some(Command::Right),
        Named::ArrowLeft => Some(Command::Left),
        Named::Backspace => Some(Command::Parent),
        Named::Space => Some(Command::Mark),
        Named::F2 => Some(Command::Rename),
        Named::F5 => Some(Command::Refresh),
        Named::Delete => Some(Command::Trash),
        _ => None,
    }
}

fn control_command(text: &str, modifiers: Modifiers) -> Option<Command> {
    match text.to_lowercase().as_str() {
        "n" if modifiers.shift() => Some(Command::NewFolder),
        "a" => Some(Command::SelectAll),
        "c" => Some(Command::Copy),
        "x" => Some(Command::Cut),
        "v" => Some(Command::Paste),
        "h" => Some(Command::ToggleHidden),
        "r" => Some(Command::Refresh),
        "d" => Some(Command::PageDown),
        "u" => Some(Command::PageUp),
        "q" => Some(Command::Quit),
        "1" => Some(Command::View(ViewMode::Grid)),
        "2" => Some(Command::View(ViewMode::List)),
        _ => None,
    }
}

fn volume_command(text: &str) -> Option<Command> {
    if text.eq_ignore_ascii_case("u") {
        return Some(Command::UnmountCurrent);
    }
    let Command::Place(index) = place(text)? else {
        return None;
    };
    Some(Command::Volume(index))
}

fn letter_command(text: &str) -> Option<Command> {
    match text {
        "j" => Some(Command::Down),
        "k" => Some(Command::Up),
        "l" => Some(Command::Open),
        "h" => Some(Command::Parent),
        "g" => Some(Command::First),
        "G" => Some(Command::Last),
        "~" => Some(Command::Home),
        "." => Some(Command::ToggleHidden),
        "/" => Some(Command::Filter),
        "y" => Some(Command::Copy),
        "x" => Some(Command::Cut),
        "p" => Some(Command::Paste),
        "r" => Some(Command::Rename),
        "a" => Some(Command::Create),
        "d" => Some(Command::Trash),
        "q" => Some(Command::Quit),
        digit => place(digit),
    }
}

fn place(text: &str) -> Option<Command> {
    let digit = text.parse::<usize>().ok()?;
    (1..=9).contains(&digit).then(|| Command::Place(digit - 1))
}
