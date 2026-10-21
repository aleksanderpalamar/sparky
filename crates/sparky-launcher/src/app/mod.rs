pub mod keys;
pub mod state;
pub mod view;

use iced::event::{self, Event, Status};
use iced::{Subscription, Task, window};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::Skin;
use sparky_ui::press::{Press, press};
use sparky_ui::prompt;
use sparky_ui::surface::transparent;

use crate::adapters::catalog::installed_apps;
use crate::adapters::launch::launch;
use crate::app::keys::movement;
use crate::app::state::Launcher;
use crate::domain::app::App;

const NAMESPACE: &str = "sparky-launcher";
const SIZE: (u32, u32) = (640, 440);

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Typed(String),
    Submit,
    Step(isize),
    Pick(usize),
    Close,
}

pub fn run() -> Result<(), iced_layershell::Error> {
    let apps = installed_apps();
    let boot = move || {
        let launcher = Launcher::new(apps.clone(), Skin::new(sparky_theme::current_palette()));
        (launcher, prompt::focus())
    };
    iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view::view)
        .subscription(|_| Subscription::batch([
            event::listen_with(keyboard_message),
            sparky_ui::palette::subscription().map(Message::Palette),
        ]))
        .theme(|launcher: &Launcher| launcher.skin.theme())
        .style(transparent)
        .settings(settings())
        .run()
}

fn update(launcher: &mut Launcher, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => launcher.skin = Skin::new(palette),
        Message::Typed(text) => launcher.type_query(text),
        Message::Step(delta) => launcher.step(delta),
        Message::Submit => return start(launcher.chosen()),
        Message::Pick(row) => return start(launcher.pick(row)),
        Message::Close => return iced::exit(),
        _ => {}
    }
    Task::none()
}

fn start(app: Option<App>) -> Task<Message> {
    let Some(app) = app else {
        return Task::none();
    };
    if let Err(error) = launch(&app) {
        eprintln!("sparky-launcher: não deu para abrir {}: {error}", app.name);
    }
    iced::exit()
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Close),
        Press::Key(key, modifiers) => movement(&key, modifiers).map(Message::Step),
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some(SIZE),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        default_font: sparky_ui::style::FONT,
        ..Settings::default()
    }
}
