mod controller;
#[cfg(test)]
mod controller_tests;
pub mod input;
pub mod view;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use iced::event::{self, Event, Status};
use iced::keyboard::{Key, key::Named};
use iced::{Subscription, Task, window};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::Skin;
use sparky_ui::press::{Press, press};

use crate::domain::{Form, Identity, Outcome, Request, Secret};
use crate::ports::{Authenticator, Responses};

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Typed(Secret),
    Select(Identity),
    Submit,
    Next,
    Close,
    Backend(u64, crate::domain::Event),
}

pub struct Window {
    pub form: Form,
    pub skin: Skin,
    backend: Arc<dyn Authenticator>,
    responses: Option<Box<dyn Responses>>,
    generation: u64,
    outcome: Arc<AtomicU8>,
}

pub fn run(
    request: Request,
    cancellation: tokio::sync::oneshot::Receiver<()>,
    backend: Arc<dyn Authenticator>,
    outcome: Arc<AtomicU8>,
) -> Result<(), String> {
    let form = Form::new(request, unsafe { libc::geteuid() })?;
    let boot_backend = backend.clone();
    let cancellation = Mutex::new(Some(cancellation));
    let boot = move || {
        let mut window = Window {
            form: form.clone(),
            skin: Skin::new(sparky_theme::current_palette()),
            backend: boot_backend.clone(),
            responses: None,
            generation: 0,
            outcome: outcome.clone(),
        };
        let Some(cancellation) = cancellation.lock().ok().and_then(|mut slot| slot.take()) else {
            return (window, Task::done(Message::Close));
        };
        let auth = controller::start(&mut window);
        let cancelled = Task::perform(
            async move {
                let _ = cancellation.await;
            },
            |()| Message::Close,
        );
        (window, Task::batch([auth, cancelled]))
    };
    let result = iced_layershell::application(
        boot,
        || "polkit-sparky".to_owned(),
        controller::update,
        show,
    )
    .subscription(|_| subscription())
    .theme(|window: &Window| window.skin.theme())
    .style(sparky_ui::surface::transparent)
    .settings(settings())
    .run()
    .map_err(|error| error.to_string());
    backend.shutdown();
    result
}

fn show(window: &Window) -> iced::Element<'_, Message> {
    view::view(&window.form, window.skin)
}

fn subscription() -> Subscription<Message> {
    Subscription::batch([
        event::listen_with(keyboard),
        sparky_ui::palette::subscription().map(Message::Palette),
    ])
}

fn keyboard(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Close),
        Press::Key(Key::Named(Named::Tab), _) => Some(Message::Next),
        Press::Key(Key::Named(Named::Enter), _) => Some(Message::Submit),
        _ => None,
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((520, 360)),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        default_font: sparky_ui::style::FONT,
        antialiasing: true,
        ..Settings::default()
    }
}

pub fn result(outcome: &AtomicU8) -> Outcome {
    Outcome::from_code(Some(i32::from(outcome.load(Ordering::SeqCst))))
}
