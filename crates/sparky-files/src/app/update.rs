use std::path::PathBuf;

use iced::Task;
use iced::keyboard::{Key, Modifiers};

use crate::app::context;
use crate::app::keys::{self, Command};
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::app::volumes;
use crate::domain::entry::Kind;
use crate::domain::grid::{Direction, ViewMode};
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
        Message::VolumesCompleted(operation, result) => {
            volumes::completed(files, operation, result)
        }
        Message::Tick => {
            files.refresh_if_changed();
            files.icons = files.services.icons.load();
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
        Mode::ConfirmTrash(_) => {
            if let Some(answer) = keys::confirm(key) {
                files.answer_trash(answer);
            }
            None
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

fn act(files: &mut Files, command: Command) {
    let page = files.layout.capacity() as isize;
    match command {
        Command::Down => files.move_cursor(Direction::Down),
        Command::Up => files.move_cursor(Direction::Up),
        Command::Left if files.view_mode == ViewMode::List => files.parent(),
        Command::Right if files.view_mode == ViewMode::List => files.open_selected(),
        Command::Left => files.move_cursor(Direction::Left),
        Command::Right => files.move_cursor(Direction::Right),
        Command::PageDown => files.listing.move_by(page),
        Command::PageUp => files.listing.move_by(-page),
        Command::First => files.listing.first(),
        Command::Last => files.listing.last(),
        Command::Open => files.open_selected(),
        Command::Parent => files.parent(),
        Command::Home => files.home(),
        Command::Place(index) => files.place(index),
        Command::ToggleHidden => files.listing.toggle_hidden(),
        Command::Mark => {
            files.listing.toggle_mark();
            files.listing.move_by(1);
        }
        Command::Copy => files.copy(),
        Command::Cut => files.cut(),
        Command::Paste => files.paste(),
        Command::Trash => files.ask_trash(),
        Command::SelectAll => files.listing.select_all(),
        Command::Properties => {
            let path = selected_path(files);
            context::properties(files, &path);
        }
        Command::Filter
        | Command::Create
        | Command::NewFolder
        | Command::Rename
        | Command::Quit
        | Command::View(_)
        | Command::Volume(_)
        | Command::UnmountCurrent
        | Command::Refresh => {}
    }
}

fn selected_path(files: &Files) -> PathBuf {
    let selected = files.listing.selected();
    selected.map_or_else(
        || files.location.clone(),
        |entry| files.location.join(&entry.name),
    )
}
