use alacritty_terminal::grid::Scroll as Viewport;
use alacritty_terminal::term::TermMode;
use iced::keyboard::{Key, Modifiers};
use iced::{Task, clipboard};

use crate::app::keys::{Cursor, Pressed, Scroll, pressed};
use crate::app::message::Message;
use crate::app::state::Terminal;
use crate::app::update::{mode, scroll, with_session};
use crate::domain::input::{
    Bracketed, Buffer, Focus, Reporting, Wheel, focus_report, paste as pasted, wheel as rolled,
};

pub fn key(
    terminal: &mut Terminal,
    key: &Key,
    modifiers: Modifiers,
    text: Option<&str>,
    latin: Option<char>,
) -> Task<Message> {
    let cursor = match mode(terminal).contains(TermMode::APP_CURSOR) {
        true => Cursor::Application,
        false => Cursor::Normal,
    };
    match pressed(key, modifiers, text, latin, cursor) {
        Pressed::Send(bytes) => typed(terminal, bytes),
        Pressed::Copy => return copy(terminal),
        Pressed::Paste => return clipboard::read().map(Message::Pasted),
        Pressed::Scroll(page) => scroll(terminal, viewport_of(page)),
        Pressed::Ignored => {}
    }
    Task::none()
}

pub fn paste(terminal: &mut Terminal, text: Option<String>) {
    let Some(text) = text else {
        return;
    };
    let bracketed = match mode(terminal).contains(TermMode::BRACKETED_PASTE) {
        true => Bracketed::Yes,
        false => Bracketed::No,
    };
    typed(terminal, pasted(&text, bracketed));
}

pub fn wheel(terminal: &mut Terminal, lines: f32) {
    let buffer = match mode(terminal).contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) {
        true => Buffer::Alternate,
        false => Buffer::Main,
    };
    let (rolled, rest) = rolled(lines, terminal.wheel_rest, buffer);
    terminal.wheel_rest = rest;
    match rolled {
        Wheel::History(0) => {}
        Wheel::History(steps) => scroll(terminal, Viewport::Delta(steps)),
        Wheel::Keys(bytes) => with_session(terminal, |session| session.write(bytes)),
    }
}

pub fn commit(terminal: &mut Terminal, text: String) {
    typed(terminal, text.into_bytes());
}

pub fn focus(terminal: &mut Terminal, focus: Focus) {
    let reporting = match mode(terminal).contains(TermMode::FOCUS_IN_OUT) {
        true => Reporting::On,
        false => Reporting::Off,
    };
    if let Some(report) = focus_report(focus, reporting) {
        with_session(terminal, |session| session.write(report.to_vec()));
    }
}

fn typed(terminal: &mut Terminal, bytes: Vec<u8>) {
    with_session(terminal, |session| {
        session.clear_selection();
        session.scroll(Viewport::Bottom);
        session.write(bytes);
    });
}

fn copy(terminal: &Terminal) -> Task<Message> {
    match terminal.session.as_ref().and_then(|session| session.copy()) {
        Some(text) => clipboard::write(text),
        None => Task::none(),
    }
}

fn viewport_of(page: Scroll) -> Viewport {
    match page {
        Scroll::PageUp => Viewport::PageUp,
        Scroll::PageDown => Viewport::PageDown,
        Scroll::Top => Viewport::Top,
        Scroll::Bottom => Viewport::Bottom,
    }
}
