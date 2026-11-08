use iced::{Task, window};
use iced_layershell::reexport::NewLayerShellSettings;

use crate::app::message::Message;

pub fn create(settings: NewLayerShellSettings, id: window::Id) -> Task<Message> {
    Task::done(Message::ForgetLastOutput).chain(Task::done(Message::NewLayerShell { settings, id }))
}
