use iced::keyboard::{Key, Modifiers};
use sparky_theme::{Palette, TerminalPalette};

use crate::adapters::listener::{Listener, Signal};
use crate::domain::grid::Spot;
use crate::domain::input::Focus;
use crate::domain::tabs::{TabAction, TabId};

#[derive(Debug, Clone)]
pub enum Message {
    Connected(TabId, Listener),
    Pty(TabId, Signal),
    Resized(iced::Size),
    Key(Key, Modifiers, Option<String>, Option<char>),
    Colors(Palette, TerminalPalette),
    Pasted(TabId, Option<String>),
    Commit(String),
    Focus(Focus),
    Rescaled(f32),
    Press(TabId, Spot),
    Drag(TabId, Spot),
    Wheel(TabId, f32, Spot),
    Selected(TabId),
    PastePrimary(TabId),
    Tab(TabAction),
    SelectTab(TabId),
    CloseTab(TabId),
}
