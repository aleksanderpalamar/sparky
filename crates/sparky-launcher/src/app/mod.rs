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
use crate::adapters::icons::Icons;
use crate::adapters::launch::launch;
use crate::app::keys::{confirms, movement};
use crate::app::state::Launcher;
use crate::domain::app::App;
use crate::domain::mode::Mode;
use crate::domain::system::entries;

const NAMESPACE: &str = "sparky-launcher";

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Icons(std::collections::HashMap<String, std::path::PathBuf>),
    Typed(String),
    Submit,
    Step(isize),
    Pick(usize),
    Close,
}

pub fn run(mode: Mode) -> Result<(), iced_layershell::Error> {
    let apps = match mode {
        Mode::Apps => installed_apps(),
        Mode::System => entries(),
    };
    let boot = move || {
        let launcher = Launcher::new(
            apps.clone(),
            Skin::new(sparky_theme::current_palette()),
            mode,
        );
        let focus = match mode.search() {
            Some(_) => prompt::focus(),
            None => Task::none(),
        };
        (launcher, Task::batch([focus, icons(apps.clone())]))
    };
    iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view::view)
        .subscription(|_| {
            Subscription::batch([
                event::listen_with(keyboard_message),
                sparky_ui::palette::subscription().map(Message::Palette),
            ])
        })
        .theme(|launcher: &Launcher| launcher.skin.theme())
        .style(transparent)
        .settings(settings(mode))
        .run()
}

fn update(launcher: &mut Launcher, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => launcher.skin = Skin::new(palette),
        Message::Icons(icons) => launcher.icons = icons,
        Message::Typed(text) => launcher.type_query(text),
        Message::Step(delta) => launcher.step(delta),
        Message::Submit => return start(launcher.chosen()),
        Message::Pick(row) => return start(launcher.pick(row)),
        Message::Close => return iced::exit(),
        _ => {}
    }
    Task::none()
}

fn icons(apps: Vec<App>) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || Icons::current().resolve(&apps))
                .await
                .unwrap_or_default()
        },
        Message::Icons,
    )
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
        Press::Key(key, _) if confirms(&key) => Some(Message::Submit),
        Press::Key(key, modifiers) => movement(&key, modifiers).map(Message::Step),
    }
}

fn settings(mode: Mode) -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some(mode.size()),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        default_font: sparky_ui::style::FONT,
        ..Settings::default()
    }
}
