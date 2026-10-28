use std::path::{Path, PathBuf};

use iced::Task;
use sparky_ui::prompt;

use crate::app::message::Message;
use crate::app::state::{ContextMenu, Files, Mode, OpenWith};
use crate::domain::clip::Clip;
use crate::domain::context::{self, Action, Row, Target};
use crate::domain::entry::Kind;

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
        target: if selected.is_directory() {
            Target::Folder
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
        target: Target::Background,
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
    if !offered(files, menu.target, action) {
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

pub fn open_with(files: &mut Files, id: &str) {
    let Some(choice) = files.open_with.as_ref() else {
        return;
    };
    if !choice.apps.iter().any(|app| app.id == id) {
        return;
    }
    let path = choice.path.clone();
    files.open_with = None;
    if let Err(error) = files.services.opener.open_with(&path, id) {
        files.fail(error);
    }
}

fn offered(files: &Files, target: Target, action: Action) -> bool {
    context::entries(target, files.clip.is_some(), !files.listing.is_empty())
        .into_iter()
        .any(|row| matches!(row, Row::Item { action: listed, enabled: true, .. } if listed == action))
}

fn run(files: &mut Files, action: Action, path: PathBuf) -> Task<Message> {
    match action {
        Action::Rename => return files.begin_rename().map_or_else(Task::none, prompt::select),
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
        Action::Open => files.open_selected(),
        Action::OpenWith => applications(files, path),
        Action::Cut => files.cut(),
        Action::Copy => files.copy(),
        Action::Email => email(files),
        Action::Trash => files.ask_trash(),
        Action::Properties => properties(files, &path),
        Action::Paste => files.paste(),
        Action::SelectAll => files.listing.select_all(),
    }
    Task::none()
}

fn destination(files: &mut Files, clip: fn(Vec<PathBuf>) -> Clip) -> Task<Message> {
    files.begin_destination(clip(files.chosen_paths()));
    prompt::focus()
}

fn applications(files: &mut Files, path: PathBuf) {
    let apps = files.services.opener.applications(&path);
    if apps.is_empty() {
        files.fail("Nenhum aplicativo disponível para este tipo de arquivo");
        return;
    }
    files.open_with = Some(OpenWith { path, apps });
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
            files.fail("Compacte as pastas antes de enviá-las por e-mail");
            return;
        }
        Ok(false) => files.services.opener.email(&paths),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        files.fail(error);
    }
}
