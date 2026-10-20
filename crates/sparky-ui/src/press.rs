use iced::event::{Event, Status};
use iced::keyboard::{self, Key, Modifiers, key::Named};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Press {
    Escape,
    Key(Key, Modifiers),
}

pub fn press(event: Event, status: Status) -> Option<Press> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };
    match (&key, status) {
        (Key::Named(Named::Escape), _) => Some(Press::Escape),
        (_, Status::Ignored) => Some(Press::Key(key, modifiers)),
        (_, Status::Captured) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Press, press};
    use iced::event::{Event, Status};
    use iced::keyboard::key::{NativeCode, Named, Physical};
    use iced::keyboard::{self, Key, Location, Modifiers};

    fn tecla(key: Key) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn esc_chega_mesmo_com_a_caixa_de_texto_em_foco() {
        assert_eq!(press(tecla(Key::Named(Named::Escape)), Status::Captured), Some(Press::Escape));
    }

    #[test]
    fn tecla_capturada_por_um_widget_nao_vira_comando() {
        assert_eq!(press(tecla(Key::Character("j".into())), Status::Captured), None);
    }

    #[test]
    fn tecla_livre_chega_com_os_modificadores() {
        assert_eq!(
            press(tecla(Key::Character("j".into())), Status::Ignored),
            Some(Press::Key(Key::Character("j".into()), Modifiers::empty()))
        );
    }

    #[test]
    fn soltar_tecla_e_ignorado() {
        let solta = Event::Keyboard(keyboard::Event::ModifiersChanged(Modifiers::SHIFT));
        assert_eq!(press(solta, Status::Ignored), None);
    }
}
