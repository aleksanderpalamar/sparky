use iced::widget::Space;
use iced::{Element, Task, Theme, window};
use iced_layershell::actions::IcedXdgWindowSettings;
use libsparky::desk::Desk;
use libsparky::popup::{self, Step};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::view::{self, context};

pub type Handler = fn(&mut Files, Message) -> Task<Message>;

pub struct Shell {
    pub files: Files,
    desk: Desk,
    menu: Option<window::Id>,
    step: Handler,
}

pub fn open(
    files: Files,
    task: Task<Message>,
    window: IcedXdgWindowSettings,
    step: Handler,
) -> (Shell, Task<Message>) {
    let main = window::Id::unique();
    let opened = Task::done(Message::NewBaseWindow {
        settings: window,
        id: main,
    });
    let shell = Shell {
        files,
        desk: Desk::new(main),
        menu: None,
        step,
    };
    (shell, Task::batch([opened, task]))
}

pub fn update(shell: &mut Shell, message: Message) -> Task<Message> {
    let task = match message {
        Message::Moved(id, point) => within(shell, id, Message::Pointer(point)),
        Message::Sized(id, size) => within(shell, id, Message::Resized(size)),
        Message::Closed(id) => return closed(shell, id),
        other => (shell.step)(&mut shell.files, other),
    };
    Task::batch([task, sync(shell)])
}

pub fn view(shell: &Shell, id: window::Id) -> Element<'_, Message> {
    if shell.desk.is_main(id) {
        return view::view(&shell.files);
    }
    match shell.menu == Some(id) {
        true => context::popup(&shell.files),
        false => Space::new().into(),
    }
}

pub fn title(shell: &Shell, id: window::Id) -> Option<String> {
    shell.desk.is_main(id).then(|| view::title(&shell.files))
}

pub fn theme(shell: &Shell, _id: window::Id) -> Option<Theme> {
    Some(shell.files.skin.theme())
}

fn within(shell: &mut Shell, id: window::Id, message: Message) -> Task<Message> {
    match shell.desk.is_main(id) {
        true => (shell.step)(&mut shell.files, message),
        false => Task::none(),
    }
}

fn closed(shell: &mut Shell, id: window::Id) -> Task<Message> {
    if shell.desk.ends_with(id) {
        return iced::exit();
    }
    if shell.menu == Some(id) {
        shell.menu = None;
        shell.files.context = None;
    }
    Task::none()
}

fn sync(shell: &mut Shell) -> Task<Message> {
    match popup::step(shell.files.context.is_some(), shell.menu) {
        Step::Open => show(shell),
        Step::Close(id) => {
            shell.menu = None;
            window::close(id)
        }
        Step::Keep => Task::none(),
    }
}

fn show(shell: &mut Shell) -> Task<Message> {
    let (Some(menu), Some(asked)) = (context::menu(&shell.files), shell.files.context.as_ref())
    else {
        return Task::none();
    };
    let point = asked.point;
    let size = menu.size(shell.files.size.height);
    let id = window::Id::unique();
    shell.menu = Some(id);
    Task::done(Message::NewPopUp {
        settings: popup::at(shell.desk.main(), point, size),
        id,
    })
}

#[cfg(test)]
#[path = "shell_tests.rs"]
mod tests;
