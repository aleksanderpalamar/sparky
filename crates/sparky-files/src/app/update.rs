use iced::Task;
use iced::keyboard::{Key, Modifiers};

use crate::app::keys::{self, Command};
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::view::style::rows_for;
use sparky_ui::prompt;

pub fn update(files: &mut Files, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => files.skin = sparky_theme::Skin::new(palette),
        Message::Scrolled(lines) => files.viewport.scroll(lines, files.listing.len()),
        Message::Resized(size) => {
            let (cursor, len) = (files.listing.cursor(), files.listing.len());
            files.viewport.resize(rows_for(size.height), cursor, len);
        }
        Message::Tick => files.refresh_if_changed(),
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
        Message::Submit => files.submit(),
        Message::Clicked(row) => files.listing.select(files.viewport.top() + row),
        Message::Activated(row) => {
            files.listing.select(files.viewport.top() + row);
            files.open_selected();
        }
        Message::Place(index) => files.place(index),
        Message::Key(..) | Message::Scrolled(_) | Message::Resized(_) | Message::Tick | Message::Palette(_) => {}
    }
}

fn key_pressed(files: &mut Files, key: &Key, modifiers: Modifiers) -> Task<Message> {
    let command = match &files.mode {
        Mode::Browse => keys::browse(key, modifiers),
        Mode::Filter => keys::filtering(key),
        Mode::ConfirmTrash(_) => {
            if let Some(answer) = keys::confirm(key) {
                files.answer_trash(answer);
            }
            None
        }
        Mode::Rename(_) | Mode::Create => None,
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
        Command::Quit => iced::exit(),
        other => {
            act(files, other);
            Task::none()
        }
    }
}

fn act(files: &mut Files, command: Command) {
    let page = files.viewport.rows() as isize;
    match command {
        Command::Down => files.listing.move_by(1),
        Command::Up => files.listing.move_by(-1),
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
        Command::Refresh => files.refresh(),
        Command::Filter | Command::Create | Command::Rename | Command::Quit => {}
    }
}
