use iced::Task;

use super::message::Message;
use super::state::{Page, Store};
use crate::domain::SearchResult;

pub fn browse(store: &mut Store) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    store.busy = true;
    store.error = None;
    store.pending = None;
    let backend = store.backend.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.browse())
                .await
                .map_err(|error| error.to_string())?
        },
        Message::Browsed,
    )
}

pub fn browsed(store: &mut Store, result: Result<SearchResult, String>) {
    store.busy = false;
    match result {
        Ok(result) => {
            store.page = Page::Browse;
            store.apps = result.apps;
            store.selected = None;
            store.visible_count = 80;
            store.flathub_missing = result.flathub_missing;
            store.error = result.warning;
        }
        Err(error) => store.error = Some(error),
    }
}
