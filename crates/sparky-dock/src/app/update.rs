use iced::{Task, window};
use sparky_theme::Skin;

use super::contact::Contact;
use super::message::Message;
use super::state::{Dock, OpenPicker};
use super::{animate, lost, pointer, surface, surfaces};
use crate::adapters::{catalog, icons, outputs};
use crate::domain::picker::Picker;

pub fn update(dock: &mut Dock, message: Message) -> Task<Message> {
    let task = handle(dock, message);
    animate::aim(dock);
    let shape = surface::sync(dock);
    animate::aim(dock);
    Task::batch([task, shape])
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
        Message::Preferences(preferences) => {
            dock.preferences = preferences;
            return measure();
        }
        Message::Output(width) => return lost::found(dock, width),
        Message::Closed(window) => return lost::closed(dock, window),
        Message::Hover(inside) => {
            dock.hovering = inside;
            if !inside {
                dock.menu = None;
                pointer::left(dock);
            }
        }
        Message::Surface(window, contact) if window == dock.window => {
            return handle(dock, felt(contact));
        }
        Message::Pointer(point) => pointer::moved(dock, point),
        Message::Press(id) => pointer::pressed(dock, &id),
        Message::Release => pointer::released(dock),
        Message::Frame(now) => animate::advance(dock, now),
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
        Message::Opened(window) if window == dock.window => {
            dock.accepting = None;
            return measure();
        }
        Message::Opened(window)
            if dock
                .picker
                .as_ref()
                .is_some_and(|open| open.window == window) =>
        {
            return sparky_ui::prompt::focus();
        }
        Message::Search(query) => edit(dock, |picker, _| picker.search(query)),
        Message::Toggle(id) => edit(dock, |picker, dock| {
            picker.toggle(&id, &dock.pinned, dock.room())
        }),
        Message::Confirm => return confirm(dock),
        Message::Cancel => return close_picker(dock),
        _ => {}
    }
    Task::none()
}

fn felt(contact: Contact) -> Message {
    match contact {
        Contact::Entered => Message::Hover(true),
        Contact::Left => Message::Hover(false),
        Contact::Moved(point) => Message::Pointer(point),
        Contact::Released => Message::Release,
    }
}

pub fn measure() -> Task<Message> {
    Task::perform(
        async { tokio::task::spawn_blocking(outputs::focused_width).await },
        |width| Message::Output(width.ok().flatten()),
    )
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
        measure(),
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
