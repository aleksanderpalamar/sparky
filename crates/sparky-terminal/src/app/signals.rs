use alacritty_terminal::term::ClipboardType;
use alacritty_terminal::vte::ansi::Rgb as Spec;
use iced::{Task, clipboard};

use super::message::Message;
use super::workspace::Workspace;
use crate::adapters::listener::{Listener, Signal};
use crate::adapters::session::Session;
use crate::domain::color::by_index;
use crate::domain::input::Focus;
use crate::domain::tabs::TabId;

pub fn connect(app: &mut Workspace, id: TabId, listener: Listener) -> Task<Message> {
    let focus = match app.tabs.active_id() == Some(id) {
        true => app.focus,
        false => Focus::Lost,
    };
    let Some(tab) = app.tabs.get_mut(id) else {
        return Task::none();
    };
    if tab.value.session.is_some() {
        return Task::none();
    }
    let terminal = &mut tab.value;
    match Session::start(&terminal.launch, terminal.size, listener) {
        Ok(session) => {
            terminal.session = Some(session);
            terminal.refresh();
            super::input::focus(terminal, focus);
        }
        Err(error) => {
            terminal.title = "Erro ao abrir o shell".into();
            terminal.error = Some(format!("Não deu para abrir o shell: {error}"));
        }
    }
    Task::none()
}

pub fn receive(app: &mut Workspace, id: TabId, signal: Signal) -> Task<Message> {
    let active = app.tabs.active_id() == Some(id);
    let Some(tab) = app.tabs.get_mut(id) else {
        return Task::none();
    };
    let terminal = &mut tab.value;
    match signal {
        Signal::Redraw if active => terminal.refresh(),
        Signal::Redraw => {}
        Signal::Title(title) => terminal.title = title,
        Signal::ResetTitle => terminal.reset_title(),
        Signal::Store(ClipboardType::Clipboard, text) => return clipboard::write(text),
        Signal::Store(ClipboardType::Selection, text) => return clipboard::write_primary(text),
        Signal::Write(text) => write(terminal, text),
        Signal::Color(index, reply) => {
            let color = by_index(index, &terminal.palette);
            write(
                terminal,
                reply(Spec {
                    r: color.red,
                    g: color.green,
                    b: color.blue,
                }),
            );
        }
        Signal::Size(reply) => write(terminal, reply(terminal.size.window())),
        Signal::Exited => return super::tabs::close(app, id),
    }
    Task::none()
}

fn write(terminal: &super::state::Terminal, text: String) {
    if let Some(session) = &terminal.session {
        session.write(text.into_bytes());
    }
}
