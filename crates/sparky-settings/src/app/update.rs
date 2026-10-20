use iced::Task;
use iced::keyboard::{Key, Modifiers};
use sparky_ui::prompt;

use crate::app::actions::Opened;
use crate::app::keys::{self, Command};
use crate::app::message::Message;
use crate::app::state::{Mode, Settings};
use crate::domain::section::Section;
use crate::domain::value::Step;
use crate::view::style::rows_for;

pub fn update(settings: &mut Settings, message: Message) -> Task<Message> {
    match message {
        Message::Key(key, modifiers) => key_pressed(settings, &key, modifiers),
        Message::Activate(index) => {
            settings.point(index);
            opened(settings.activate())
        }
        other => {
            apply(settings, other);
            Task::none()
        }
    }
}

fn apply(settings: &mut Settings, message: Message) {
    match message {
        Message::Escape => settings.escape(),
        Message::Typed(text) => settings.input = text,
        Message::Submit => settings.submit(),
        Message::Section(section) => settings.go(section),
        Message::Point(index) => settings.point(index),
        Message::Nudge(index, step) => {
            settings.point(index);
            settings.nudge(step);
        }
        Message::Remove(index) => {
            settings.point(index);
            settings.remove();
        }
        Message::Scrolled(lines) => settings.list.scroll(lines, settings.count()),
        Message::Resized(size) => {
            let (focus, count) = (settings.focus, settings.count());
            settings.list.resize(rows_for(size.height), focus, count);
        }
        Message::Key(..) | Message::Activate(_) => {}
    }
}

fn key_pressed(settings: &mut Settings, key: &Key, modifiers: Modifiers) -> Task<Message> {
    if settings.mode != Mode::Browse {
        return Task::none();
    }
    let Some(command) = keys::browse(key, modifiers) else {
        return Task::none();
    };
    settings.notice = None;
    match command {
        Command::Activate => return opened(settings.activate()),
        Command::Quit => return iced::exit(),
        Command::Down => settings.move_focus(1),
        Command::Up => settings.move_focus(-1),
        Command::Less => settings.nudge(Step::Less),
        Command::More => settings.nudge(Step::More),
        Command::Remove => settings.remove(),
        Command::NextSection => settings.go(settings.section.next()),
        Command::PreviousSection => settings.go(settings.section.previous()),
        Command::Section(index) => {
            if let Some(section) = Section::nth(index) {
                settings.go(section);
            }
        }
        Command::Refresh => settings.reload(),
    }
    Task::none()
}

fn opened(opened: Opened) -> Task<Message> {
    match opened {
        Opened::Prompt => prompt::focus(),
        Opened::Nothing => Task::none(),
    }
}
