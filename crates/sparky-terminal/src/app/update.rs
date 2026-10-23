use alacritty_terminal::grid::Scroll as Viewport;
use alacritty_terminal::term::TermMode;
use alacritty_terminal::vte::ansi::Rgb as Spec;
use iced::{Task, clipboard};

use crate::adapters::listener::Signal;
use crate::adapters::session::Session;
use crate::app::input::{key, wheel};
use crate::app::message::Message;
use crate::app::state::Terminal;
use crate::domain::color::by_index;

pub fn update(terminal: &mut Terminal, message: Message) -> Task<Message> {
    match message {
        Message::Connected(listener) => return connect(terminal, listener),
        Message::Pty(signal) => return signal_received(terminal, signal),
        Message::Resized(size) => terminal.fit(size.width, size.height),
        Message::Key(pressed, modifiers, text) => {
            return key(terminal, &pressed, modifiers, text.as_deref());
        }
        Message::Colors(palette, colors) => {
            terminal.observe_colors(palette, colors);
        }
        Message::Pasted(text) => crate::app::input::paste(terminal, text),
        Message::Loaded(Signal::Load(reply), Some(text)) => {
            with_session(terminal, |session| session.write(reply(&text).into_bytes()))
        }
        Message::Loaded(..) => {}
        Message::Press(spot) => with_session(terminal, |session| session.select(spot)),
        Message::Drag(spot) => with_session(terminal, |session| session.extend(spot)),
        Message::Wheel(lines) => wheel(terminal, lines),
    }
    Task::none()
}

pub fn with_session(terminal: &mut Terminal, action: impl FnOnce(&Session)) {
    if let Some(session) = &terminal.session {
        action(session);
    }
    terminal.refresh();
}

fn connect(
    terminal: &mut Terminal,
    listener: crate::adapters::listener::Listener,
) -> Task<Message> {
    match Session::start(&terminal.launch, terminal.size, listener) {
        Ok(session) => {
            terminal.session = Some(session);
            terminal.refresh();
            Task::none()
        }
        Err(error) => {
            eprintln!("sparky-terminal: não deu para abrir o shell: {error}");
            iced::exit()
        }
    }
}

fn signal_received(terminal: &mut Terminal, signal: Signal) -> Task<Message> {
    match signal {
        Signal::Redraw => terminal.refresh(),
        Signal::Title(title) => terminal.title = title,
        Signal::ResetTitle => terminal.reset_title(),
        Signal::Store(text) => return clipboard::write(text),
        Signal::Load(reply) => {
            return clipboard::read()
                .map(move |text| Message::Loaded(Signal::Load(reply.clone()), text));
        }
        Signal::Write(text) => with_session(terminal, |session| session.write(text.into_bytes())),
        Signal::Color(index, reply) => {
            let color = by_index(index, &terminal.palette);
            let answer = reply(Spec {
                r: color.red,
                g: color.green,
                b: color.blue,
            });
            with_session(terminal, |session| session.write(answer.into_bytes()));
        }
        Signal::Size(reply) => {
            let answer = reply(terminal.size.window());
            with_session(terminal, |session| session.write(answer.into_bytes()));
        }
        Signal::Exited => return iced::exit(),
    }
    Task::none()
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
