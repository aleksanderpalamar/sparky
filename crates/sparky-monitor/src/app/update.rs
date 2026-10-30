use iced::Task;
use sparky_theme::Skin;

use super::message::Message;
use super::state::Monitor;

pub fn update(monitor: &mut Monitor, message: Message) -> Task<Message> {
    if !matches!(message, Message::Tick | Message::Sampled(_)) {
        monitor.notice = None;
    }
    match message {
        Message::Tick => return monitor.tick(),
        Message::Quit => return iced::exit(),
        Message::Palette(palette) => monitor.skin = Skin::new(palette),
        Message::Resized(size) => monitor.size = size,
        Message::Sampled(snapshot) => monitor.sampled(*snapshot),
        Message::Catalog(catalog) => monitor.catalog = catalog,
        Message::Open(page) => monitor.open(page),
        Message::SortBy(column) => monitor.sort_by(column),
        Message::Select(selection) => monitor.select(selection),
        Message::Move(movement) => monitor.step(movement),
        Message::Scrolled(lines) => monitor.scrolled(lines),
        Message::SwitchFocus => monitor.switch_focus(),
        Message::Ask(stop) => monitor.ask(stop),
        Message::Confirm => monitor.confirm(),
        Message::Escape => monitor.escape(),
    }
    Task::none()
}
