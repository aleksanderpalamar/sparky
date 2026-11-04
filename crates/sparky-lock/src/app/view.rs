use iced::widget::{responsive, stack, text};
use iced::{Element, Length, Size, window};
use sparky_login_look::layout::{Layout, Part};
use sparky_login_look::style::Colors;
use sparky_login_look::widgets::{Field, Look, action, clock, input, warning};
use sparky_login_look::{clock as hours, i18n, icons, scene};

use crate::app::update::{PASSWORD, USER};
use crate::app::{Locker, Message};

const PARTS: [Part; 6] = [
    Part::Time,
    Part::Date,
    Part::Warning,
    Part::Username,
    Part::Password,
    Part::Login,
];

pub fn view(locker: &Locker, _window: window::Id) -> Element<'_, Message> {
    responsive(move |size| screen(locker, size)).into()
}

pub fn screen(locker: &Locker, size: Size) -> Element<'_, Message> {
    let look = Look {
        layout: Layout::new(size.width, size.height),
        colors: Colors::new(locker.skin),
    };
    let backdrop = scene::backdrop(locker.backdrop.as_ref(), locker.skin.background, size, look);
    let form = scene::form(size.height, look, &PARTS, |part| {
        part_of(locker, part, look)
    });
    stack![backdrop, form]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn part_of(locker: &Locker, part: Part, look: Look) -> Element<'_, Message> {
    let layout = look.layout;
    match part {
        Part::Time => clock(hours::time(locker.now.time()), layout.time, look),
        Part::Date => clock(hours::date(locker.now.date_naive()), layout.date, look),
        Part::Warning => warning(locker.unlock.warning(), look),
        Part::Username => user(locker, look),
        Part::Password => password(locker, look),
        Part::Login => action(
            i18n::text("unlock"),
            false,
            locker.unlock.ready().then_some(Message::Submit),
            look,
        ),
        Part::PowerIcons | Part::PowerLabels | Part::Session => text("").into(),
    }
}

fn user(locker: &Locker, look: Look) -> Element<'_, Message> {
    let field = Field {
        id: USER,
        value: &locker.user,
        secure: false,
        typed: None,
        submitted: None,
        icon: icons::user(),
        icon_press: None,
    };
    input(field, look)
}

fn password(locker: &Locker, look: Look) -> Element<'_, Message> {
    let reveal = locker.unlock.reveal();
    let field = Field {
        id: PASSWORD,
        value: locker.unlock.password(),
        secure: reveal.secure(),
        typed: Some(Message::Typed),
        submitted: Some(Message::Submit),
        icon: icons::password(reveal),
        icon_press: Some(Message::Reveal),
    };
    input(field, look)
}
