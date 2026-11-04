use iced::Element;
use iced::widget::{mouse_area, text};
use sparky_login_look::icons;
use sparky_login_look::widgets::{Field, Look, action, input, underlined};

use crate::app::update::{PASSWORD, USERNAME};
use crate::app::{Greeter, Message};
use crate::domain::focus::Focus;

const LOGIN: &str = "Login";

pub fn username(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let field = Field {
        id: USERNAME,
        value: greeter.form.username(),
        secure: false,
        typed: Some(Message::TypedUsername),
        submitted: Some(Message::Submitted(Focus::Username)),
        icon: icons::user(),
        icon_press: None,
    };
    input(field, look)
}

pub fn password(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let reveal = greeter.form.reveal();
    let field = Field {
        id: PASSWORD,
        value: greeter.form.password(),
        secure: reveal.secure(),
        typed: Some(Message::TypedPassword),
        submitted: Some(Message::Submitted(Focus::Password)),
        icon: icons::password(reveal),
        icon_press: Some(Message::Reveal),
    };
    input(field, look)
}

pub fn login(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let ready = greeter.form.ready();
    action(
        LOGIN,
        greeter.focus == Focus::Login,
        ready.then_some(Message::Login),
        look,
    )
}

pub fn session(greeter: &Greeter, look: Look) -> Element<'_, Message> {
    let Some(sessions) = &greeter.sessions else {
        return text("").into();
    };
    let label = underlined(sessions.label(), greeter.focus == Focus::Session, look);
    mouse_area(label).on_press(Message::NextSession).into()
}
