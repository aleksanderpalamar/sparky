use std::sync::Arc;

use chrono::Local;
use iced::Task;
use iced::widget::operation;

use crate::app::{Greeter, Message, Step, blocking};
use crate::domain::focus::Focus;
use crate::domain::form::Refusal;
use crate::domain::power::{Power, label};

pub const USERNAME: &str = "usuario";
pub const PASSWORD: &str = "senha";
const NOWHERE: &str = "nenhum";
const STOPPED: &str = "o pedido de login parou no meio";
const NO_SESSION: &str = "nenhuma sessão Wayland instalada";

pub fn update(greeter: &mut Greeter, message: Message) -> Task<Message> {
    match message {
        Message::Tick => greeter.now = Local::now(),
        Message::Backdrop(Ok(backdrop)) => greeter.backdrop = Some(backdrop),
        Message::Backdrop(Err(error)) => {
            eprintln!("sparky-login-greeter: papel de parede: {error}")
        }
        Message::TypedUsername(text) => {
            type_in(greeter, Focus::Username, |form| form.type_username(text))
        }
        Message::TypedPassword(text) => {
            type_in(greeter, Focus::Password, |form| form.type_password(text))
        }
        Message::Submitted(Focus::Username) => return move_to(greeter, Focus::Password),
        Message::Submitted(_) | Message::Login => return submit(greeter),
        Message::Reveal => greeter.form.toggle_reveal(),
        Message::Power(action) => return power(greeter, action),
        Message::NextSession => next_session(greeter),
        Message::Move(step) => return observe(step),
        Message::Observed(step, field) => return observed(greeter, step, field),
        Message::Activate => return activate(greeter),
        Message::Close => return iced::exit(),
        Message::Answer(Ok(())) => return iced::exit(),
        Message::Answer(Err(refusal)) => return refused(greeter, refusal),
        Message::Powered(result) => greeter.notice = result.err(),
        _ => {}
    }
    Task::none()
}

pub fn focused(username: bool, password: bool) -> Option<Focus> {
    match (username, password) {
        (true, _) => Some(Focus::Username),
        (_, true) => Some(Focus::Password),
        _ => None,
    }
}

fn observe(step: Step) -> Task<Message> {
    operation::is_focused(USERNAME).then(move |username| {
        operation::is_focused(PASSWORD)
            .map(move |password| Message::Observed(step, focused(username, password)))
    })
}

fn observed(greeter: &mut Greeter, step: Step, field: Option<Focus>) -> Task<Message> {
    let current = field.unwrap_or(greeter.focus);
    let target = match step {
        Step::Forward => current.next(),
        Step::Back => current.previous(),
    };
    move_to(greeter, target)
}

pub fn focus_on(focus: Focus) -> Task<Message> {
    match focus {
        Focus::Username => operation::focus(USERNAME),
        Focus::Password => operation::focus(PASSWORD),
        _ => operation::focus(NOWHERE),
    }
}

fn type_in(greeter: &mut Greeter, focus: Focus, edit: impl FnOnce(&mut crate::domain::form::Form)) {
    edit(&mut greeter.form);
    greeter.focus = focus;
    greeter.notice = None;
}

fn move_to(greeter: &mut Greeter, focus: Focus) -> Task<Message> {
    greeter.focus = focus;
    focus_on(focus)
}

fn activate(greeter: &mut Greeter) -> Task<Message> {
    match greeter.focus {
        Focus::Username => move_to(greeter, Focus::Password),
        Focus::Password | Focus::Login => submit(greeter),
        Focus::Power(action) => power(greeter, action),
        Focus::Session => {
            next_session(greeter);
            Task::none()
        }
    }
}

fn submit(greeter: &mut Greeter) -> Task<Message> {
    let Some(session) = greeter
        .sessions
        .as_ref()
        .map(|sessions| sessions.current().id.clone())
    else {
        greeter.notice = Some(NO_SESSION.to_owned());
        return Task::none();
    };
    let Some(credentials) = greeter.form.submit() else {
        return Task::none();
    };
    greeter.notice = None;
    let backend = Arc::clone(&greeter.backend);
    let lost = Err(Refusal::Unavailable(STOPPED.to_owned()));
    Task::perform(
        blocking(move || backend.login(credentials, &session), lost),
        Message::Answer,
    )
}

fn refused(greeter: &mut Greeter, refusal: Refusal) -> Task<Message> {
    greeter.form.refuse(refusal);
    move_to(greeter, Focus::Password)
}

fn power(greeter: &mut Greeter, action: Power) -> Task<Message> {
    greeter.focus = Focus::Power(action);
    let backend = Arc::clone(&greeter.backend);
    let lost = Err(format!("{} parou no meio", label(action)));
    Task::batch([
        focus_on(greeter.focus),
        Task::perform(
            blocking(move || backend.power(action), lost),
            Message::Powered,
        ),
    ])
}

fn next_session(greeter: &mut Greeter) {
    greeter.focus = Focus::Session;
    if let Some(sessions) = greeter.sessions.as_mut() {
        sessions.next();
    }
}
