use std::fmt;
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::vte::ansi::Rgb;
use iced::futures::channel::mpsc::UnboundedSender;

pub type TextReply = Arc<dyn Fn(&str) -> String + Sync + Send + 'static>;
pub type ColorReply = Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>;
pub type SizeReply = Arc<dyn Fn(WindowSize) -> String + Sync + Send + 'static>;

#[derive(Clone)]
pub enum Signal {
    Redraw,
    Title(String),
    ResetTitle,
    Store(String),
    Load(TextReply),
    Write(String),
    Color(usize, ColorReply),
    Size(SizeReply),
    Exited,
}

impl fmt::Debug for Signal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::Redraw => formatter.write_str("Redraw"),
            Signal::Title(title) => write!(formatter, "Title({title})"),
            Signal::ResetTitle => formatter.write_str("ResetTitle"),
            Signal::Store(_) => formatter.write_str("Store(..)"),
            Signal::Load(_) => formatter.write_str("Load(..)"),
            Signal::Write(_) => formatter.write_str("Write(..)"),
            Signal::Color(index, _) => write!(formatter, "Color({index})"),
            Signal::Size(_) => formatter.write_str("Size(..)"),
            Signal::Exited => formatter.write_str("Exited"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Listener(pub UnboundedSender<Signal>);

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        if let Some(signal) = signal_of(event) {
            let _ = self.0.unbounded_send(signal);
        }
    }
}

pub fn signal_of(event: Event) -> Option<Signal> {
    match event {
        Event::Wakeup => Some(Signal::Redraw),
        Event::Title(title) => Some(Signal::Title(title)),
        Event::ResetTitle => Some(Signal::ResetTitle),
        Event::ClipboardStore(_, text) => Some(Signal::Store(text)),
        Event::ClipboardLoad(_, reply) => Some(Signal::Load(reply)),
        Event::PtyWrite(text) => Some(Signal::Write(text)),
        Event::ColorRequest(index, reply) => Some(Signal::Color(index, reply)),
        Event::TextAreaSizeRequest(reply) => Some(Signal::Size(reply)),
        Event::Exit | Event::ChildExit(_) => Some(Signal::Exited),
        Event::MouseCursorDirty | Event::CursorBlinkingChange | Event::Bell => None,
    }
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::event::Event;

    use super::{Signal, signal_of};

    #[test]
    fn so_os_eventos_que_o_terminal_usa_viram_sinais() {
        assert!(matches!(signal_of(Event::Wakeup), Some(Signal::Redraw)));
        assert!(
            matches!(signal_of(Event::Title("zsh".to_owned())), Some(Signal::Title(title)) if title == "zsh")
        );
        assert!(matches!(signal_of(Event::Exit), Some(Signal::Exited)));
        assert!(signal_of(Event::Bell).is_none());
        assert!(signal_of(Event::MouseCursorDirty).is_none());
    }
}
