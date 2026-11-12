use std::path::PathBuf;

use crate::app::context;
use crate::app::keys::Command;
use crate::app::state::Files;
use crate::domain::grid::{Direction, ViewMode};
use crate::domain::removal::Removal;

pub(super) fn act(files: &mut Files, command: Command) {
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
        Command::Trash => files.ask_trash(),
        Command::Delete => files.ask_removal(Removal::Delete),
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
        | Command::Paste
        | Command::Undo
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
