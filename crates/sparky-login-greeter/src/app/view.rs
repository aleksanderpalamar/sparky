use iced::widget::{responsive, stack};
use iced::{Element, Length, Size};
use sparky_login_look::clock;
use sparky_login_look::layout::{Layout, Part};
use sparky_login_look::scene;
use sparky_login_look::style::Colors;
use sparky_login_look::widgets::{self as look_widgets, Look};

use crate::app::power;
use crate::app::widgets;
use crate::app::{Greeter, Message};

const PARTS: [Part; 9] = [
    Part::Time,
    Part::Date,
    Part::Warning,
    Part::Username,
    Part::Password,
    Part::Login,
    Part::PowerIcons,
    Part::PowerLabels,
    Part::Session,
];

pub fn view(greeter: &Greeter) -> Element<'_, Message> {
    responsive(move |size| screen(greeter, size)).into()
}

fn screen(greeter: &Greeter, size: Size) -> Element<'_, Message> {
    let look = Look {
        layout: Layout::new(size.width, size.height),
        colors: Colors::new(greeter.skin),
    };
    let backdrop = scene::backdrop(
        greeter.backdrop.as_ref(),
        greeter.skin.background,
        size,
        look,
    );
    let form = scene::form(size.height, look, &PARTS, |part| {
        part_of(greeter, part, look)
    });
    stack![backdrop, form]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn part_of(greeter: &Greeter, part: Part, look: Look) -> Element<'_, Message> {
    let layout = look.layout;
    match part {
        Part::Time => look_widgets::clock(clock::time(greeter.now.time()), layout.time, look),
        Part::Date => look_widgets::clock(clock::date(greeter.now.date_naive()), layout.date, look),
        Part::Warning => look_widgets::warning(
            greeter.form.warning().or_else(|| greeter.notice.clone()),
            look,
        ),
        Part::Username => widgets::username(greeter, look),
        Part::Password => widgets::password(greeter, look),
        Part::Login => widgets::login(greeter, look),
        Part::PowerIcons => power::icons(look),
        Part::PowerLabels => power::labels(greeter.focus, look),
        Part::Session => widgets::session(greeter, look),
    }
}
