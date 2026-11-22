use std::collections::HashMap;

use iced_layershell::to_layer_message;
use sparky_launcher::app::pictures::Shown;

use crate::domain::app::App;
use crate::domain::presence::Seen;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Catalog(Vec<App>),
    Icons(HashMap<String, Shown>),
    Windows(Vec<Seen>),
    Palette(sparky_theme::Palette),
    Preferences(sparky_config::Dock),
    Surface(iced::window::Id, super::contact::Contact),
    Hover(bool),
    Pointer(iced::Point),
    Press(String),
    Release,
    Frame(std::time::Instant),
    Menu(String),
    Unpin(String),
    Start(String),
    OpenPicker,
    Opened(iced::window::Id),
    Search(String),
    Toggle(String),
    Confirm,
    Cancel,
}
