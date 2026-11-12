mod browse;
mod installed;
mod message;
mod state;
mod update;
mod updates;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod updates_tests;

use std::sync::Arc;

use iced::{Size, Subscription, Task};

use crate::adapters::SystemStore;
use crate::ports::StoreBackend;
use crate::view;

pub use message::Message;
pub use state::{Page, Store};

pub fn run() -> iced::Result {
    iced::application(boot, update::update, view::view)
        .title(title)
        .subscription(|_: &Store| -> Subscription<Message> {
            sparky_ui::palette::subscription().map(Message::Palette)
        })
        .default_font(sparky_ui::style::FONT)
        .theme(|store: &Store| store.skin.theme())
        .window(sparky_ui::window::settings(
            "sparky-store",
            Size::new(1050.0, 680.0),
        ))
        .run()
}

fn title(_: &Store) -> String {
    crate::i18n::text("title")
}

fn boot() -> (Store, Task<Message>) {
    let backend: Arc<dyn StoreBackend> = Arc::new(SystemStore::default());
    let store = Store::new(
        backend,
        sparky_theme::Skin::new(sparky_theme::current_palette()),
    );
    (store, Task::done(Message::Browse))
}
