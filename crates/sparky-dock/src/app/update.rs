use iced::{Task, window};
use sparky_theme::Skin;

use super::message::Message;
use super::state::{Dock, OpenPicker};
use super::{surface, surfaces};
use crate::adapters::{catalog, icons};
use crate::domain::picker::Picker;

pub fn update(dock: &mut Dock, message: Message) -> Task<Message> {
    let task = handle(dock, message);
    Task::batch([task, surface::sync(dock)])
}

fn handle(dock: &mut Dock, message: Message) -> Task<Message> {
    match message {
        Message::Catalog(apps) => {
            dock.apps = apps;
            dock.catalogued = true;
            return load_icons(dock);
        }
        Message::Icons(icons) => dock.icons = icons,
        Message::Windows(windows) => dock.windows = windows,
        Message::Palette(palette) => dock.skin = Skin::new(palette),
        Message::Preferences(preferences) => dock.preferences = preferences,
        Message::Hover(inside) => {
            dock.hovering = inside;
            if !inside {
                dock.menu = None;
            }
        }
        Message::Menu(id) => dock.menu = Some(id),
        Message::Unpin(id) => {
            dock.menu = None;
            dock.unpin(&id);
        }
        Message::Start(id) => {
            dock.menu = None;
            dock.start(&id);
        }
        Message::OpenPicker => return open_picker(dock),
        Message::Opened(window)
            if dock
                .picker
                .as_ref()
                .is_some_and(|open| open.window == window) =>
        {
            return sparky_ui::prompt::focus();
        }
        Message::Search(query) => edit(dock, |picker, _| picker.search(query)),
        Message::Toggle(id) => edit(dock, |picker, dock| picker.toggle(&id, &dock.pinned)),
        Message::Confirm => return confirm(dock),
        Message::Cancel => return close_picker(dock),
        _ => {}
    }
    Task::none()
}

pub fn load_catalog() -> Task<Message> {
    Task::perform(
        async { tokio::task::spawn_blocking(catalog::installed_apps).await },
        |apps| Message::Catalog(apps.unwrap_or_default()),
    )
}

fn load_icons(dock: &Dock) -> Task<Message> {
    let apps = dock.apps.clone();
    Task::perform(
        async move { tokio::task::spawn_blocking(move || icons::resolve(&apps)).await },
        |icons| Message::Icons(icons.unwrap_or_default()),
    )
}

fn open_picker(dock: &mut Dock) -> Task<Message> {
    dock.menu = None;
    if dock.picker.is_some() {
        return Task::none();
    }
    let window = window::Id::unique();
    dock.picker = Some(OpenPicker {
        window,
        picker: Picker::default(),
    });
    Task::batch([
        Task::done(Message::NewLayerShell {
            settings: surfaces::picker(),
            id: window,
        }),
        load_catalog(),
    ])
}

fn edit(dock: &mut Dock, change: impl FnOnce(&mut Picker, &Dock)) {
    let Some(mut open) = dock.picker.take() else {
        return;
    };
    change(&mut open.picker, dock);
    dock.picker = Some(open);
}

fn confirm(dock: &mut Dock) -> Task<Message> {
    let chosen = dock
        .picker
        .as_ref()
        .map(|open| open.picker.chosen().to_vec())
        .unwrap_or_default();
    dock.pin(&chosen);
    close_picker(dock)
}

fn close_picker(dock: &mut Dock) -> Task<Message> {
    dock.picker.take().map_or_else(Task::none, |open| {
        Task::done(Message::RemoveWindow(open.window))
    })
}
