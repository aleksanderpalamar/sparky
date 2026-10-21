pub mod view;

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::event::{self, Event, Status};
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::operation;
use iced::{Subscription, Task, window};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::Skin;
use sparky_ui::press::{Press, press};
use sparky_ui::surface::transparent;

use crate::domain::form::{Field, Form, Step};
use crate::domain::mode::Mode;

const NAMESPACE: &str = "sparky-keyring-prompt";
const WIDTH: u32 = 460;
const HEIGHT: u32 = 224;
const CONFIRMATION_HEIGHT: u32 = 44;

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Typed(Field, String),
    Enter(Field),
    Next,
    Close,
}

pub struct Window {
    pub form: Form,
    pub skin: Skin,
    answered: Arc<AtomicBool>,
}

pub fn run(mode: Mode, answered: Arc<AtomicBool>) -> Result<(), iced_layershell::Error> {
    let boot = move || {
        let window = Window { form: Form::new(mode), skin: Skin::new(sparky_theme::current_palette()), answered: Arc::clone(&answered) };
        (window, focus(Field::Password))
    };
    iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view::view)
        .subscription(|_| Subscription::batch([event::listen_with(keyboard_message), sparky_ui::palette::subscription().map(Message::Palette)]))
        .theme(|window: &Window| window.skin.theme())
        .style(transparent)
        .settings(settings(mode))
        .run()
}

pub fn id_of(field: Field) -> &'static str {
    match field {
        Field::Password => "password",
        Field::Confirmation => "confirmation",
    }
}

fn update(window: &mut Window, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => window.skin = Skin::new(palette),
        Message::Typed(field, text) => window.form.type_in(field, text),
        Message::Enter(field) => {
            let step = window.form.enter(field);
            return follow(window, step);
        }
        Message::Next => return window.form.switch().map_or_else(Task::none, focus),
        Message::Close => return iced::exit(),
        _ => {}
    }
    Task::none()
}

fn follow(window: &Window, step: Step) -> Task<Message> {
    match step {
        Step::Focus(field) => focus(field),
        Step::Stay => Task::none(),
        Step::Done(password) => {
            match deliver(&password) {
                Ok(()) => window.answered.store(true, Ordering::SeqCst),
                Err(error) => eprintln!("sparky-keyring-prompt: não deu para entregar a senha: {error}"),
            }
            iced::exit()
        }
    }
}

fn deliver(password: &str) -> io::Result<()> {
    let mut output = io::stdout().lock();
    output.write_all(password.as_bytes())?;
    output.write_all(b"\n")?;
    output.flush()
}

fn focus(field: Field) -> Task<Message> {
    operation::focus(id_of(field))
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Close),
        Press::Key(Key::Named(Named::Tab), _) => Some(Message::Next),
        Press::Key(..) => None,
    }
}

fn settings(mode: Mode) -> Settings {
    let height = match mode.confirms() {
        true => HEIGHT + CONFIRMATION_HEIGHT,
        false => HEIGHT,
    };
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((WIDTH, height)),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        ..Settings::default()
    }
}
