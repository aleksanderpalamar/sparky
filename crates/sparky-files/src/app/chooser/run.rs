use std::rc::Rc;

use iced::widget::operation;
use iced::{Size, Task};

use super::{APPLICATION_ID, Chooser, NAME_INPUT, update};
use crate::adapters::portal::answer::{StdoutAnswer, mime_of};
use crate::app::boot;
use crate::app::events;
use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::choice::Request;
use crate::view;

const WIDTH: f32 = 875.0;
const HEIGHT: f32 = 600.0;

pub fn run(request: Request) -> iced::Result {
    iced::application(move || start(request.clone()), update, view::view)
        .title(view::title)
        .subscription(events::subscription)
        .default_font(sparky_ui::style::FONT)
        .theme(|files: &Files| files.skin.theme())
        .style(sparky_ui::surface::transparent)
        .window(sparky_ui::window::settings(
            APPLICATION_ID,
            Size::new(WIDTH, HEIGHT),
        ))
        .run()
}

fn start(request: Request) -> (Files, Task<Message>) {
    let mut files = boot::assemble(request.folder.clone());
    let chooser = Chooser::new(request, Rc::new(StdoutAnswer), mime_of);
    let naming = chooser.naming();
    let length = stem_length(&chooser.name);
    files.chooser = Some(chooser);
    files.refresh();
    let (files, task) = boot::ready(files);
    let focus = match naming {
        true => operation::focus(NAME_INPUT).chain(operation::select_range(NAME_INPUT, 0, length)),
        false => Task::none(),
    };
    (files, Task::batch([task, focus]))
}

pub fn stem_length(name: &str) -> usize {
    match name.rfind('.') {
        Some(dot) if dot > 0 => name[..dot].chars().count(),
        _ => name.chars().count(),
    }
}
