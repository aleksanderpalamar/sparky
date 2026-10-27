mod popup;
#[cfg(test)]
mod tests;
mod worker;

use crate::{
    client::{Client, Command},
    domain::Card,
    service::Shared,
};
use iced::{Element, Subscription, Task, window};
use iced_layershell::{
    settings::{LayerShellSettings, Settings, StartMode},
    to_layer_message,
};
use sparky_theme::Skin;
use std::sync::Arc;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Ready(Arc<Client>),
    Changed,
    Palette(sparky_theme::Palette),
    Command(Command),
    Finished(Result<(), String>),
    Geometry(window::Id, popup::Geometry),
    Fatal(String),
}

pub struct App {
    worker: worker::Worker,
    skin: Skin,
    client: Option<Arc<Client>>,
    window: Option<window::Id>,
    cards: Vec<Card>,
    geometry: popup::Geometry,
}

pub fn run() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::daemon(boot, || "sparky-notifications".to_owned(), update, view)
        .subscription(subscription)
        .theme(|app: &App, _| app.skin.theme())
        .settings(Settings {
            layer_settings: LayerShellSettings {
                start_mode: StartMode::Background,
                ..Default::default()
            },
            default_font: sparky_ui::style::FONT,
            default_text_size: 12.0.into(),
            antialiasing: true,
            ..Default::default()
        })
        .run()
}

fn boot() -> App {
    App {
        worker: worker::Worker(Shared::new()),
        skin: Skin::new(sparky_theme::current_palette()),
        client: None,
        window: None,
        cards: Vec::new(),
        geometry: popup::Geometry::default(),
    }
}

fn subscription(app: &App) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run_with(app.worker.clone(), worker::events),
        sparky_ui::palette::subscription().map(Message::Palette),
    ])
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Ready(client) => app.client = Some(client),
        Message::Changed => return refresh(app),
        Message::Palette(palette) => app.skin = Skin::new(palette),
        Message::Geometry(id, geometry) if app.window == Some(id) => {
            app.geometry = geometry;
            return Task::done(Message::NewLayerShell {
                id,
                settings: popup::settings(
                    popup::size(app.cards.len(), &app.geometry),
                    app.geometry.output.clone(),
                ),
            });
        }
        Message::Command(command) => {
            let Some(client) = app.client.clone() else {
                return Task::none();
            };
            return Task::perform(async move { client.send(command).await }, Message::Finished);
        }
        Message::Finished(Err(error)) => eprintln!("sparky-notifications: {error}"),
        Message::Fatal(error) => {
            eprintln!("sparky-notifications: {error}");
            return iced::exit();
        }
        _ => {}
    }
    Task::none()
}

fn refresh(app: &mut App) -> Task<Message> {
    let cards = match app.worker.0.read(|center| center.popups()) {
        Ok(cards) => cards,
        Err(error) => return Task::done(Message::Fatal(error.to_string())),
    };
    app.cards = cards;
    if app.cards.is_empty() {
        return app
            .window
            .take()
            .map_or_else(Task::none, |id| Task::done(Message::RemoveWindow(id)));
    }
    if let Some(id) = app.window {
        return Task::done(Message::SizeChange {
            id,
            size: popup::size(app.cards.len(), &app.geometry),
        });
    }
    let id = window::Id::unique();
    app.window = Some(id);
    Task::perform(
        async { tokio::task::spawn_blocking(popup::geometry).await },
        move |result| Message::Geometry(id, result.unwrap_or_default()),
    )
}

fn view(app: &App, _: window::Id) -> Element<'_, Message> {
    popup::view(&app.skin, &app.cards)
}
