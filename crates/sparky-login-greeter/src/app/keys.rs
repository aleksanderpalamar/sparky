use iced::event::{Event, Status};
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::window;
use sparky_ui::press::{Press, press};

use crate::app::{Message, Mode, Step};

type Listener = fn(Event, Status, window::Id) -> Option<Message>;

pub fn for_mode(mode: Mode) -> Listener {
    match mode {
        Mode::Preview => preview,
        Mode::Greeter => greeter,
    }
}

fn preview(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    translate(press(event, status)?, Some(Message::Close))
}

fn greeter(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    translate(press(event, status)?, None)
}

fn translate(press: Press, escape: Option<Message>) -> Option<Message> {
    match press {
        Press::Escape => escape,
        Press::Key(Key::Named(Named::Tab), modifiers) if modifiers.shift() => {
            Some(Message::Move(Step::Back))
        }
        Press::Key(Key::Named(Named::Tab), _) => Some(Message::Move(Step::Forward)),
        Press::Key(Key::Named(Named::Enter), _) => Some(Message::Activate),
        Press::Key(..) => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::event::{Event, Status};
    use iced::keyboard::key::{Named, NativeCode, Physical};
    use iced::keyboard::{self, Key, Location, Modifiers};
    use iced::window;

    use super::for_mode;
    use crate::app::{Message, Mode, Step};

    fn tecla(named: Named) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(named),
            modified_key: Key::Named(named),
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            text: None,
            repeat: false,
        })
    }

    fn mensagem(mode: Mode, named: Named) -> Option<Message> {
        for_mode(mode)(tecla(named), Status::Ignored, window::Id::unique())
    }

    #[test]
    fn esc_fecha_so_a_previa() {
        assert!(matches!(
            mensagem(Mode::Preview, Named::Escape),
            Some(Message::Close)
        ));
        assert!(mensagem(Mode::Greeter, Named::Escape).is_none());
    }

    #[test]
    fn tab_e_enter_valem_nos_dois_modos() {
        for mode in [Mode::Preview, Mode::Greeter] {
            assert!(matches!(
                mensagem(mode, Named::Tab),
                Some(Message::Move(Step::Forward))
            ));
            assert!(matches!(
                mensagem(mode, Named::Enter),
                Some(Message::Activate)
            ));
        }
    }
}
