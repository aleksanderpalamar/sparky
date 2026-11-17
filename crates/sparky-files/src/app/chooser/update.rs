use std::path::PathBuf;

use iced::Task;
use iced::keyboard::{Key, Modifiers};

use super::ChooserEvent;
use super::guard::{allowed, blocked};
use crate::app::keys::{self, Command};
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::choice::{Decision, Purpose, decide};
use crate::domain::entry::Kind;
use crate::domain::grid::ViewMode;

pub fn update(files: &mut Files, message: Message) -> Task<Message> {
    if files.chooser.is_none() {
        return crate::app::update::update(files, message);
    }
    match message {
        Message::Chooser(event) => handle(files, event),
        Message::Activated(index) => activate(files, index),
        Message::Key(key, modifiers) => pressed(files, key, modifiers),
        Message::Escape if files.mode == Mode::Browse && files.search.is_none() => iced::exit(),
        message if blocked(&message) => Task::none(),
        message => {
            if matches!(message, Message::Clicked(_)) {
                touch(files);
            }
            crate::app::update::update(files, message)
        }
    }
}

pub fn accept(files: &mut Files) -> Task<Message> {
    let Some(chooser) = &files.chooser else {
        return Task::none();
    };
    let purpose = chooser.purpose();
    let untouched = !chooser.touched_here(&files.location);
    let selected = match purpose {
        Purpose::Open {
            directory: true, ..
        }
        | Purpose::SaveFiles { .. }
            if untouched =>
        {
            Vec::new()
        }
        _ => selection(files),
    };
    let disk = files.services.disk.as_ref();
    match decide(&purpose, &files.location, &selected, |path| {
        disk.exists(path)
    }) {
        Decision::Choose(paths) => finish(files, paths),
        Decision::Enter(folder) => {
            files.go(folder);
            Task::none()
        }
        Decision::Confirm(paths) => {
            if let Some(chooser) = &mut files.chooser {
                chooser.replace = Some(paths);
            }
            Task::none()
        }
        Decision::Nothing => Task::none(),
    }
}

fn handle(files: &mut Files, event: ChooserEvent) -> Task<Message> {
    let Some(chooser) = &mut files.chooser else {
        return Task::none();
    };
    match event {
        ChooserEvent::Name(name) => {
            chooser.name = name;
            chooser.replace = None;
        }
        ChooserEvent::Filter(index) => {
            chooser.filter = Some(index);
            files.refresh();
        }
        ChooserEvent::Accept => return accept(files),
        ChooserEvent::Replace => {
            let paths = chooser.replace.take().unwrap_or_default();
            return finish(files, paths);
        }
        ChooserEvent::KeepEditing => chooser.replace = None,
        ChooserEvent::Cancel => return iced::exit(),
    }
    Task::none()
}

fn activate(files: &mut Files, index: usize) -> Task<Message> {
    let Some(entry) = files.listing.get(index).cloned() else {
        return Task::none();
    };
    touch(files);
    let naming = files
        .chooser
        .as_ref()
        .is_some_and(|chooser| chooser.naming());
    match (entry.kind, naming) {
        (Kind::Directory, _) => crate::app::update::update(files, Message::Activated(index)),
        (Kind::File, true) => {
            files.listing.select(index);
            if let Some(chooser) = &mut files.chooser {
                chooser.name = entry.name;
                chooser.replace = None;
            }
            Task::none()
        }
        (Kind::File, false) => {
            files.listing.select(index);
            accept(files)
        }
    }
}

fn pressed(files: &mut Files, key: Key, modifiers: Modifiers) -> Task<Message> {
    if files.mode == Mode::Browse && files.context.is_none() {
        match keys::browse(&key, modifiers) {
            Some(Command::Open) => return accept(files),
            Some(Command::Right) if files.view_mode == ViewMode::List => {
                return activate(files, files.listing.cursor());
            }
            Some(Command::Quit) => return iced::exit(),
            Some(command) if !allowed(command) => return Task::none(),
            Some(_) => touch(files),
            None => {}
        }
    }
    crate::app::update::update(files, Message::Key(key, modifiers))
}

fn finish(files: &mut Files, paths: Vec<PathBuf>) -> Task<Message> {
    if paths.is_empty() {
        return Task::none();
    }
    if let Some(chooser) = &files.chooser {
        chooser.answer(paths);
    }
    iced::exit()
}

fn selection(files: &Files) -> Vec<(PathBuf, Kind)> {
    files
        .listing
        .chosen()
        .into_iter()
        .filter_map(|name| {
            let kind = files.listing.find(&name)?.kind;
            Some((files.location.join(name), kind))
        })
        .collect()
}

fn touch(files: &mut Files) {
    let location = files.location.clone();
    if let Some(chooser) = &mut files.chooser {
        chooser.touched = Some(location);
    }
}
