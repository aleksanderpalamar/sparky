use iced::Task;

use super::message::Message;
use super::state::{Page, Store};
use crate::domain::Application;

pub fn installed(store: &mut Store) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    store.busy = true;
    store.error = None;
    store.pending = None;
    store.selected = None;
    let backend = store.backend.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.installed())
                .await
                .map_err(|error| error.to_string())?
        },
        Message::Installed,
    )
}

pub fn loaded(store: &mut Store, result: Result<Vec<Application>, String>) {
    store.busy = false;
    match result {
        Ok(apps) => {
            store.page = Page::Installed;
            store.apps = apps;
            store.selected = None;
            store.visible_count = 80;
            store.icons = crate::adapters::icons::resolve(&store.apps, store.visible_count);
        }
        Err(error) => store.error = Some(error),
    }
}
