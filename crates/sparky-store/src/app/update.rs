use iced::Task;

use super::browse::{browse, browsed};
use super::installed::{installed, loaded};
use super::message::Message;
use super::state::{Page, Store};
use crate::domain::{Action, SearchSource};

pub fn update(store: &mut Store, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => store.skin = sparky_theme::Skin::new(palette),
        Message::Query(query) => {
            store.query = query;
            store.pending = None;
        }
        Message::Source(source) => {
            store.source = source;
            store.pending = None;
        }
        Message::Browse => return browse(store),
        Message::Browsed(result) => browsed(store, result),
        Message::LoadMore => {
            store.visible_count += 80;
            store.icons = crate::adapters::icons::resolve(&store.apps, store.visible_count);
        }
        Message::Search => return search(store),
        Message::Searched(result) => searched(store, result),
        Message::ShowInstalled => return installed(store),
        Message::ShowUpdates => {
            store.page = Page::Updates;
            store.selected = None;
            store.pending = None;
            store.error = None;
        }
        Message::Installed(result) => loaded(store, result),
        Message::Select(index) => {
            store.selected = store.apps.get(index).map(|_| index);
            store.pending = None;
        }
        Message::Back => {
            store.selected = None;
            store.pending = None;
        }
        Message::FeatureNext => advance_feature(store, 1),
        Message::FeaturePrevious => advance_feature(store, 4),
        Message::Topic(query) => {
            store.query = query;
            return search(store);
        }
        Message::Ask(action) => ask(store, action),
        Message::AskFor(index, action) => {
            if store.busy {
                return Task::none();
            }
            store.selected = store.apps.get(index).map(|_| index);
            ask(store, action);
        }
        Message::Cancel => store.pending = None,
        Message::Confirm => return perform(store),
        Message::Performed(result) => return performed(store, result),
        Message::EnableFlathub => return enable_flathub(store),
        Message::FlathubEnabled(result) => return flathub_enabled(store, result),
    }
    Task::none()
}

fn advance_feature(store: &mut Store, step: usize) {
    let count = store.apps.len().min(5);
    if count == 0 {
        return;
    }
    store.feature_index = (store.feature_index + step) % count;
}

fn search(store: &mut Store) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    store.busy = true;
    store.error = None;
    store.pending = None;
    let backend = store.backend.clone();
    let query = store.query.clone();
    let source = store.source;
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.search(&query, source))
                .await
                .map_err(|error| error.to_string())?
        },
        Message::Searched,
    )
}

fn searched(store: &mut Store, result: Result<crate::domain::SearchResult, String>) {
    store.busy = false;
    match result {
        Ok(result) => {
            store.page = Page::Search;
            store.apps = result.apps;
            store.selected = None;
            store.visible_count = 80;
            store.icons = crate::adapters::icons::resolve(&store.apps, store.visible_count);
            store.feature_index = 0;
            store.flathub_missing = result.flathub_missing;
            store.error = result.warning;
        }
        Err(error) => store.error = Some(error),
    }
}

fn ask(store: &mut Store, action: Action) {
    if store.busy {
        return;
    }
    store.pending = store.selected_app().cloned().map(|app| (app, action));
}

fn perform(store: &mut Store) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    let Some((app, action)) = store.pending.take() else {
        return Task::none();
    };
    store.busy = true;
    store.error = None;
    let backend = store.backend.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.perform(&app, action))
                .await
                .map_err(|error| error.to_string())?
        },
        Message::Performed,
    )
}

fn performed(store: &mut Store, result: Result<(), String>) -> Task<Message> {
    store.busy = false;
    match result {
        Ok(()) => {
            store.notice = Some(crate::i18n::text("operation-complete"));
            match store.page {
                Page::Installed => installed(store),
                Page::Search => search(store),
                Page::Browse => browse(store),
                Page::Updates => Task::none(),
            }
        }
        Err(error) => {
            store.error = Some(error);
            Task::none()
        }
    }
}

fn enable_flathub(store: &mut Store) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    store.busy = true;
    let backend = store.backend.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.enable_flathub())
                .await
                .map_err(|error| error.to_string())?
        },
        Message::FlathubEnabled,
    )
}

fn flathub_enabled(store: &mut Store, result: Result<(), String>) -> Task<Message> {
    store.busy = false;
    match result {
        Ok(()) => {
            store.flathub_missing = false;
            store.source = SearchSource::Preferred;
            browse(store)
        }
        Err(error) => {
            store.error = Some(error);
            Task::none()
        }
    }
}
