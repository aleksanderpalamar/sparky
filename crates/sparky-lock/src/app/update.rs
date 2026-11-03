use std::sync::Arc;

use chrono::Local;
use iced::Task;
use iced::widget::operation;
use sparky_login_core::protocol::Refusal;

use crate::app::{Locker, Message, blocking};

pub const PASSWORD: &str = "senha";
pub const USER: &str = "usuario";
const STOPPED: &str = "a conferência da senha parou no meio";

pub fn update(locker: &mut Locker, message: Message) -> Task<Message> {
    match message {
        Message::Tick => locker.now = Local::now(),
        Message::Backdrop(Ok(backdrop)) => locker.backdrop = Some(backdrop),
        Message::Backdrop(Err(error)) => eprintln!("sparky-lock: papel de parede: {error}"),
        Message::Typed(text) => locker.unlock.type_password(text),
        Message::Reveal => locker.unlock.toggle_reveal(),
        Message::Escape => locker.unlock.clear(),
        Message::Submit => return submit(locker),
        Message::Answer(Ok(())) => return Task::done(Message::UnLock),
        Message::Answer(Err(refusal)) => {
            locker.unlock.refuse(refusal);
            return focus_password();
        }
        _ => {}
    }
    Task::none()
}

pub fn focus_password() -> Task<Message> {
    operation::focus(PASSWORD)
}

fn submit(locker: &mut Locker) -> Task<Message> {
    let Some(password) = locker.unlock.submit() else {
        return Task::none();
    };
    let authenticator = Arc::clone(&locker.authenticator);
    let lost = Err(Refusal::Unavailable(STOPPED.to_owned()));
    Task::perform(
        blocking(move || authenticator.unlock(password), lost),
        Message::Answer,
    )
}
