use std::path::{Path, PathBuf};

use iced::Task;
use sparky_ui::prompt;

use crate::app::message::Message;
use crate::app::state::{ContextMenu, Files, Mode};
use crate::domain::archive::extractable;
use crate::domain::clip::Clip;
use crate::domain::context::{self, Action, Row, Target};
use crate::domain::entry::Kind;
use crate::i18n;

pub fn entry(files: &mut Files, index: usize) {
    if files.mode != Mode::Browse {
        return;
    }
    let Some(selected) = files.listing.get(index).cloned() else {
        return;
    };
    if !files.listing.is_marked(&selected) {
        files.listing.clear_marks();
    }
    files.listing.select(index);
    files.context = Some(ContextMenu {
        target: if files.in_trash() {
            Target::Trashed
        } else if selected.is_directory() {
            Target::Folder
        } else if extractable(&selected.name).is_some() {
            Target::Archive
        } else {
            Target::File
        },
        name: Some(selected.name),
        point: files.pointer,
    });
}

pub fn background(files: &mut Files) {
    if files.mode != Mode::Browse {
        return;
    }
    files.context = Some(ContextMenu {
        target: if files.in_trash() {
            Target::TrashBackground
        } else {
            Target::Background
        },
        name: None,
        point: files.pointer,
    });
}

pub fn is_open(files: &Files) -> bool {
    files.context.is_some() || files.details.is_some() || files.open_with.is_some()
}

pub fn choose(files: &mut Files, action: Action) -> Task<Message> {
    let Some(menu) = files.context.take() else {
        return Task::none();
    };
    if !offered(files, menu.target, action) || files.blocks_action_in_search(action) {
        return Task::none();
    }
    let path = menu
        .name
        .map_or_else(|| files.location.clone(), |name| files.location.join(name));
    run(files, action, path)
}

pub fn properties(files: &mut Files, path: &Path) {
    match files.services.disk.properties(path) {
        Ok(details) => files.details = Some(details),
        Err(error) => files.fail(error),
    }
}

fn offered(files: &Files, target: Target, action: Action) -> bool {
    let has_entries = if target == Target::TrashBackground {
        files.listing.total_len() > 0
    } else {
        !files.listing.is_empty()
    };
    context::entries(target, files.clip.is_some(), has_entries)
        .into_iter()
        .any(|row| matches!(row, Row::Item { action: listed, enabled: true, .. } if listed == action))
}

fn run(files: &mut Files, action: Action, path: PathBuf) -> Task<Message> {
    match action {
        Action::Rename => return files.start_rename(),
        Action::NewFolder => return prompt::select(files.begin_new(Kind::Directory)),
        Action::NewFile => return prompt::select(files.begin_new(Kind::File)),
        Action::MoveTo => return destination(files, Clip::Cut),
        Action::CopyTo => return destination(files, Clip::Copy),
        Action::Compress => {
            let paths = files.chosen_paths();
            return files
                .begin_archive(paths)
                .map_or_else(Task::none, prompt::select);
        }
        Action::Extract => return files.extract(),
        Action::Open => files.open_selected(),
        Action::OpenWith => return files.begin_open_with(path),
        Action::Cut => files.cut(),
        Action::Copy => files.copy(),
        Action::Email => email(files),
        Action::Trash => files.ask_trash(),
        Action::Delete => files.ask_delete(),
        Action::Properties => properties(files, &path),
        Action::Paste => return files.paste(),
        Action::SelectAll => files.listing.select_all(),
        Action::Restore => files.restore_selected(),
        Action::RestoreAll => files.restore_all(),
    }
    Task::none()
}

fn destination(files: &mut Files, clip: fn(Vec<PathBuf>) -> Clip) -> Task<Message> {
    files.begin_destination(clip(files.chosen_paths()));
    prompt::focus()
}

fn email(files: &mut Files) {
    let paths = files.chosen_paths();
    let folder = paths
        .iter()
        .map(|path| files.services.disk.properties(path))
        .try_fold(false, |folder, details| {
            details.map(|details| folder || details.kind == Kind::Directory)
        });
    let result = match folder {
        Ok(true) => {
            files.fail(i18n::text("compress-before-mail"));
            return;
        }
        Ok(false) => files.services.opener.email(&paths),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        files.fail(error);
    }
}
