use iced::Task;
use sparky_theme::Skin;

use super::keys::Command;
use super::message::Message;
use super::state::Viewer;
use crate::domain::viewport::Viewport;

pub fn update(viewer: &mut Viewer, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => viewer.skin = Skin::new(palette),
        Message::Resized(size) => viewer.size = size,
        Message::Dragged(x, y) => viewer.adjust(|viewport, area| viewport.drag(x, y, area)),
        Message::Loaded(path, result) => viewer.loaded(&path, result),
        Message::Command(command) => return run(viewer, command),
    }
    Task::none()
}

fn run(viewer: &mut Viewer, command: Command) -> Task<Message> {
    match command {
        Command::Quit => return iced::exit(),
        Command::Arrow(direction) => return viewer.arrow(direction),
        Command::Step(step) => return viewer.step(step),
        Command::Jump(edge) => return viewer.jump(edge),
        Command::ZoomIn => viewer.adjust(Viewport::zoom_in),
        Command::ZoomOut => viewer.adjust(Viewport::zoom_out),
        Command::Fit => viewer.adjust(|viewport, _| viewport.fit()),
        Command::ActualSize => viewer.adjust(|viewport, _| viewport.actual_size()),
        Command::Rotate(spin) => viewer.adjust(|viewport, _| viewport.rotate(spin)),
    }
    Task::none()
}
