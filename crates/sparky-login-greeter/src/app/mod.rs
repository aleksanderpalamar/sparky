mod icons;
mod keys;
mod power;
mod style;
mod update;
#[cfg(test)]
mod update_tests;
mod view;
#[cfg(test)]
mod view_tests;
mod widgets;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Local};
use iced::{Subscription, Task, event};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::{Palette, Skin};

use crate::adapters::backdrop::{self, Backdrop};
use crate::domain::focus::Focus;
use crate::domain::form::{Form, Refusal};
use crate::domain::power::Power;
use crate::domain::sessions::Sessions;
use crate::ports::backend::Backend;

const NAMESPACE: &str = "sparky-login-greeter";
const TICK: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Forward,
    Back,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Backdrop(Result<Backdrop, String>),
    TypedUsername(String),
    TypedPassword(String),
    Submitted(Focus),
    Reveal,
    Login,
    Power(Power),
    NextSession,
    Move(Step),
    Activate,
    Close,
    Answer(Result<(), Refusal>),
    Powered(Result<(), String>),
}

#[derive(Clone)]
pub struct Start {
    pub backend: Arc<dyn Backend>,
    pub sessions: Option<Sessions>,
    pub last_user: Option<String>,
    pub wallpaper: Option<PathBuf>,
    pub palette: Palette,
}

pub struct Greeter {
    pub form: Form,
    pub sessions: Option<Sessions>,
    pub focus: Focus,
    pub skin: Skin,
    pub now: DateTime<Local>,
    pub backdrop: Option<Backdrop>,
    pub notice: Option<String>,
    backend: Arc<dyn Backend>,
}

pub fn run(start: Start) -> Result<(), iced_layershell::Error> {
    let boot = move || boot(start.clone());
    iced_layershell::application(boot, || NAMESPACE.to_owned(), update::update, view::view)
        .subscription(|_| {
            Subscription::batch([
                event::listen_with(keys::message),
                iced::time::every(TICK).map(|_| Message::Tick),
            ])
        })
        .theme(|greeter: &Greeter| greeter.skin.theme())
        .settings(settings())
        .run()
}

pub fn boot(start: Start) -> (Greeter, Task<Message>) {
    let form = Form::new(start.last_user.as_deref());
    let focus = form.first_focus();
    let greeter = Greeter {
        form,
        sessions: start.sessions,
        focus,
        skin: Skin::new(start.palette),
        now: Local::now(),
        backdrop: None,
        notice: None,
        backend: start.backend,
    };
    let load = match start.wallpaper {
        Some(path) => Task::perform(
            blocking(
                move || backdrop::load(&path),
                Err("a leitura do papel de parede parou".to_owned()),
            ),
            Message::Backdrop,
        ),
        None => Task::none(),
    };
    (greeter, Task::batch([update::focus_on(focus), load]))
}

async fn blocking<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static, lost: T) -> T {
    tokio::task::spawn_blocking(job).await.unwrap_or(lost)
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: -1,
            size: None,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        default_font: style::REGULAR,
        ..Settings::default()
    }
}
