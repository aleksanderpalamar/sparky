use std::rc::Rc;

use iced::Task;
use iced::widget::operation;

use super::{APPLICATION_ID, Chooser, NAME_INPUT, update};
use crate::adapters::portal::answer::{StdoutAnswer, mime_of};
use crate::app::boot;
use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::choice::Request;

const WIDTH: u32 = 875;
const HEIGHT: u32 = 600;
const MIN_SIZE: (u32, u32) = (480, 360);

pub fn run(request: Request) -> Result<(), iced_layershell::Error> {
    crate::app::serve(
        move || start(request.clone()),
        APPLICATION_ID,
        libsparky::desk::window(APPLICATION_ID, (WIDTH, HEIGHT), MIN_SIZE),
        update,
    )
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
