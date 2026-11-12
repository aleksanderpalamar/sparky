use iced::Task;
use iced::keyboard::{Key, Modifiers};

use crate::app::commands::act;
use crate::app::context;
use crate::app::keys::{self, Command};
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::app::virtual_commands::virtual_command;
use crate::app::volumes;
use crate::domain::entry::Kind;
use sparky_ui::prompt;

pub fn update(files: &mut Files, message: Message) -> Task<Message> {
    let pointer = matches!(message, Message::Pointer(_));
    let task = dispatch(files, message);
    if pointer {
        return task;
    }
    Task::batch([task, files.request_previews()])
}

fn dispatch(files: &mut Files, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => files.skin = sparky_theme::Skin::new(palette),
        Message::Scrolled(lines) => files
            .viewport
            .scroll(lines, files.layout.row_count(files.listing.len())),
        Message::Pointer(point) => files.pointer = point,
        Message::ContextEntry(index) => context::entry(files, index),
        Message::ContextBackground => context::background(files),
        Message::ContextDismiss => {
            files.context = None;
            files.details = None;
            files.open_with = None;
        }
        Message::OpenWithChoice(id) => context::open_with(files, &id),
        Message::ArchiveCompleted(path, result) => files.archive_completed(path, result),
        Message::PreviewReady(path, stamp, image) => files.preview_ready(path, stamp, image),
        Message::Trash(action) => files.handle_trash(action),
        Message::TrashEmptied(result) => files.trash_emptied(result),
        Message::Sidebar(page) => files.open_sidebar(page),
        Message::SidebarItem(index) => return files.open_sidebar_item(index),
        Message::NetworkLoaded(uri, result) => files.network_loaded(&uri, result),
        Message::FavoriteSelected => files.toggle_favorite(),
        Message::RemoveFavorite(index) => files.remove_favorite(index),
        Message::ContextAction(action) => return context::choose(files, action),
        Message::Submit => {
            let task = files.submit();
            files.follow();
            return task;
        }
        Message::Resized(size) => files.resize(size),
        Message::View(mode) => files.change_view(mode),
        Message::OpenVolume(device) => return volumes::open(files, &device),
        Message::UnmountVolume(device) => return volumes::unmount(files, &device),
        Message::EjectVolume(device) => return volumes::eject(files, &device),
        Message::VolumesCompleted(operation, result) => {
            volumes::completed(files, operation, result)
        }
        Message::FolderChanged => files.refresh(),
        Message::Job(event) => files.job_event(event),
        Message::Tick => {
            files.refresh_if_changed();
            files.icons = files.services.icons.load();
            files.sidebar_icons = files.services.icons.load_sidebar();
            return volumes::refresh(files);
        }
        other => return steer(files, other),
    }
    Task::none()
}

fn steer(files: &mut Files, message: Message) -> Task<Message> {
    let task = match message {
        Message::Key(key, modifiers) => key_pressed(files, &key, modifiers),
        other => {
            apply(files, other);
            Task::none()
        }
    };
    files.follow();
    task
}

fn apply(files: &mut Files, message: Message) {
    match message {
        Message::Escape => files.escape(),
        Message::Typed(text) => files.type_text(text),
        Message::Clicked(index) => files.listing.select(index),
        Message::Activated(index) if files.listing.get(index).is_some() => {
            files.listing.select(index);
            files.open_selected();
        }
        Message::Place(index) => files.place(index),
        _ => {}
    }
}

fn key_pressed(files: &mut Files, key: &Key, modifiers: Modifiers) -> Task<Message> {
    if context::is_open(files) {
        return Task::none();
    }
    let command = match &files.mode {
        Mode::Browse => keys::browse(key, modifiers),
        Mode::Filter => keys::filtering(key),
        Mode::ConfirmTrash(_) | Mode::ConfirmEmptyTrash(_) => return files.confirm(key),
        Mode::Conflict(_) => {
            return keys::conflict(key)
                .map_or_else(Task::none, |choice| files.resolve_conflict(choice));
        }
        Mode::Rename(_)
        | Mode::Create
        | Mode::New(_)
        | Mode::Destination(_)
        | Mode::Compress(_) => None,
    };
    command.map_or_else(Task::none, |command| run(files, command))
}

fn run(files: &mut Files, command: Command) -> Task<Message> {
    if files.sidebar_page.is_some() {
        return virtual_command(files, command);
    }
    if files.blocks_in_trash(command) {
        return Task::none();
    }
    files.notice = None;
    match command {
        Command::Filter => {
            files.begin_filter();
            prompt::focus()
        }
        Command::Create => {
            files.begin_create();
            prompt::focus()
        }
        Command::Rename => files.begin_rename().map_or_else(Task::none, prompt::select),
        Command::NewFolder => prompt::select(files.begin_new(Kind::Directory)),
        Command::Quit => iced::exit(),
        Command::View(mode) => {
            files.change_view(mode);
            Task::none()
        }
        Command::Volume(index) => {
            let device = files.volumes.get(index).map(|volume| volume.path.clone());
            device.map_or_else(Task::none, |device| volumes::open(files, &device))
        }
        Command::UnmountCurrent => volumes::unmount_current(files),
        Command::Paste => files.paste(),
        Command::Undo => files.undo(),
        Command::Refresh => {
            files.refresh();
            volumes::refresh(files)
        }
        other => {
            act(files, other);
            Task::none()
        }
    }
}
