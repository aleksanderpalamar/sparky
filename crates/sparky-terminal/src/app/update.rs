use alacritty_terminal::grid::Scroll as Viewport;
use alacritty_terminal::term::TermMode;
use iced::{Task, clipboard};

use super::input::{commit, key, paste, wheel};
use super::message::Message;
use super::state::Terminal;
use super::workspace::Workspace;
use crate::adapters::session::Session;

pub fn update(app: &mut Workspace, message: Message) -> Task<Message> {
    match message {
        Message::Connected(id, listener) => super::signals::connect(app, id, listener),
        Message::Pty(id, signal) => super::signals::receive(app, id, signal),
        Message::Key(pressed, modifiers, text, latin) => {
            key(app, &pressed, modifiers, text.as_deref(), latin)
        }
        Message::Tab(action) => super::tabs::action(app, action),
        Message::CloseTab(id) => super::tabs::close(app, id),
        Message::SelectTab(id) => {
            app.select(id);
            super::tabs_view::reveal(app)
        }
        Message::PastePrimary(id) => {
            clipboard::read_primary().map(move |text| Message::Pasted(id, text))
        }
        Message::Selected(id) => {
            let text = app
                .tabs
                .get(id)
                .and_then(|tab| tab.value.session.as_ref())
                .and_then(Session::copy);
            match text {
                Some(text) => clipboard::write_primary(text),
                None => Task::none(),
            }
        }
        other => {
            change(app, other);
            Task::none()
        }
    }
}

fn change(app: &mut Workspace, message: Message) {
    match message {
        Message::Resized(size) => app.fit(size),
        Message::Rescaled(scale) => app.rescale(scale),
        Message::Colors(palette, colors) => app.observe_colors(palette, colors),
        Message::Focus(change) => app.focus(change),
        Message::Commit(text) => {
            if let Some(tab) = app.tabs.active_mut() {
                commit(&mut tab.value, text);
            }
        }
        Message::Pasted(id, text) => {
            if let Some(tab) = app.tabs.get_mut(id) {
                paste(&mut tab.value, text);
            }
        }
        Message::Press(id, spot) => {
            if let Some(tab) = app.tabs.get_mut(id) {
                with_session(&mut tab.value, |session| session.select(spot));
            }
        }
        Message::Drag(id, spot) => {
            if let Some(tab) = app.tabs.get_mut(id) {
                with_session(&mut tab.value, |session| session.extend(spot));
            }
        }
        Message::Wheel(id, lines, at) => {
            if let Some(tab) = app.tabs.get_mut(id) {
                wheel(&mut tab.value, lines, at);
            }
        }
        _ => {}
    }
}

pub fn with_session(terminal: &mut Terminal, action: impl FnOnce(&Session)) {
    if let Some(session) = &terminal.session {
        action(session);
    }
    terminal.refresh();
}

pub fn mode(terminal: &Terminal) -> TermMode {
    terminal
        .session
        .as_ref()
        .map_or(TermMode::empty(), Session::mode)
}

pub fn scroll(terminal: &mut Terminal, viewport: Viewport) {
    with_session(terminal, |session| session.scroll(viewport));
}
