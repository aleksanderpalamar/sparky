mod keys;
mod update;
mod view;
#[cfg(test)]
mod view_tests;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Local};
use iced::{Subscription, Task, event};
use iced_sessionlock::settings::Settings;
use iced_sessionlock::to_session_message;
use sparky_login_core::protocol::Refusal;
use sparky_login_look::backdrop::{self, Backdrop};
use sparky_login_look::style::REGULAR;
use sparky_theme::{Palette, Skin};

use crate::domain::unlock::Unlock;
use crate::ports::authenticator::Authenticator;

const TICK: Duration = Duration::from_secs(1);
const LOST: &str = "a conferência da senha parou no meio";

#[to_session_message]
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Backdrop(Result<Backdrop, String>),
    Typed(String),
    Submit,
    Reveal,
    Escape,
    Answer(Result<(), Refusal>),
}

#[derive(Clone)]
pub struct Start {
    pub user: String,
    pub wallpaper: Option<PathBuf>,
    pub palette: Palette,
    pub authenticator: Arc<dyn Authenticator>,
}

pub struct Locker {
    pub user: String,
    pub unlock: Unlock,
    pub skin: Skin,
    pub now: DateTime<Local>,
    pub backdrop: Option<Backdrop>,
    authenticator: Arc<dyn Authenticator>,
}

pub fn run(start: Start) -> Result<(), iced_sessionlock::Error> {
    let boot = move || boot(start.clone());
    iced_sessionlock::application(boot, update::update, view::view)
        .subscription(|_: &Locker| {
            Subscription::batch([
                event::listen_with(keys::message),
                iced::time::every(TICK).map(|_| Message::Tick),
            ])
        })
        .theme(|locker: &Locker| locker.skin.theme())
        .settings(Settings {
            default_font: REGULAR,
            antialiasing: true,
            ..Settings::default()
        })
        .run()
}

pub fn boot(start: Start) -> (Locker, Task<Message>) {
    let locker = Locker {
        user: start.user,
        unlock: Unlock::default(),
        skin: Skin::new(start.palette),
        now: Local::now(),
        backdrop: None,
        authenticator: start.authenticator,
    };
    let load = match start.wallpaper {
        Some(path) => Task::perform(
            blocking(move || backdrop::load(&path), Err(LOST.to_owned())),
            Message::Backdrop,
        ),
        None => Task::none(),
    };
    (locker, Task::batch([update::focus_password(), load]))
}

async fn blocking<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static, lost: T) -> T {
    tokio::task::spawn_blocking(job).await.unwrap_or(lost)
}
