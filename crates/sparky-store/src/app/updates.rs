use iced::Task;

use super::message::Message;
use super::state::{Page, Store};
use crate::domain::updates::{UpdateReport, UpdateTarget};

pub fn show(store: &mut Store) -> Task<Message> {
    store.page = Page::Updates;
    store.selected = None;
    store.pending = None;
    store.error = None;
    check(store, false)
}

pub fn refresh(store: &mut Store) -> Task<Message> {
    store.error = None;
    check(store, true)
}

pub fn check(store: &mut Store, refresh: bool) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    store.busy = true;
    store.update_pending = None;
    let backend = store.backend.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.check_updates(refresh))
                .await
                .map_err(|error| error.to_string())
        },
        Message::UpdatesChecked,
    )
}

pub fn checked(store: &mut Store, result: Result<UpdateReport, String>) {
    store.busy = false;
    match result {
        Ok(report) => store.updates = Some(report),
        Err(error) => store.error = Some(error),
    }
}

pub fn ask(store: &mut Store, targets: Vec<UpdateTarget>) {
    if store.busy || targets.is_empty() {
        return;
    }
    let Some(report) = &store.updates else {
        return;
    };
    let groups = report.targets();
    let valid = targets.iter().all(|target| {
        groups.contains(target)
            || report
                .groups
                .iter()
                .any(|group| group.items.iter().any(|item| item.target() == *target))
    });
    if valid {
        store.update_pending = Some(targets);
    }
}

pub fn ask_all(store: &mut Store) {
    if store.busy {
        return;
    }
    let Some(report) = &store.updates else {
        return;
    };
    let targets = report.targets();
    if !targets.is_empty() {
        store.update_pending = Some(targets);
    }
}

pub fn confirm(store: &mut Store) -> Task<Message> {
    if store.busy {
        return Task::none();
    }
    let Some(targets) = store.update_pending.take() else {
        return Task::none();
    };
    store.update_queue = targets.into();
    store.update_failures.clear();
    store.error = None;
    store.notice = None;
    store.busy = true;
    next(store)
}

fn next(store: &mut Store) -> Task<Message> {
    let Some(target) = store.update_queue.pop_front() else {
        store.updating = None;
        store.busy = false;
        if store.update_failures.is_empty() {
            store.notice = Some(crate::i18n::text("operation-complete"));
        } else {
            store.error = Some(store.update_failures.join("; "));
        }
        return check(store, false);
    };
    store.updating = Some(target.clone());
    let backend = store.backend.clone();
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || backend.apply_update(&target))
                .await
                .map_err(|error| error.to_string())?
        },
        Message::UpdateFinished,
    )
}

pub fn finished(store: &mut Store, result: Result<(), String>) -> Task<Message> {
    if let Err(error) = result {
        store.update_failures.push(error);
    }
    next(store)
}
