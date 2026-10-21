use std::rc::Rc;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::message::Message;
use crate::app::state::Mode;
use crate::app::test_support::{Comandos, Sparky, enter, painel_com, tecla};
use crate::app::update::update;
use crate::domain::section::Section;
use crate::ports::fake::FakeCommands;

#[test]
fn usuarios_navega_e_atualiza_sem_comandos_de_alteracao() {
    let commands = Comandos(Rc::new(FakeCommands::new().with("getent passwd", "ana:x:1000:1000:Ana:/home/ana:/bin/sh\nbia:x:1001:1001::/home/bia:/bin/sh\n")));
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), commands.clone());
    let _ = update(&mut settings, Message::Section(Section::Users));
    assert_eq!(settings.count(), 2);
    assert_eq!(settings.users[0].title(), "Ana (ana)");
    tecla(&mut settings, "j");
    assert_eq!(settings.focus, 1);
    let before = commands.0.calls();
    enter(&mut settings);
    tecla(&mut settings, "l");
    tecla(&mut settings, "d");
    assert_eq!(commands.0.calls(), before);
    assert_eq!(settings.mode, Mode::Browse);
    assert_eq!(*arquivo.0.borrow(), None);
    let _ = update(&mut settings, Message::Key(Key::Named(Named::F5), Modifiers::empty()));
    assert_eq!(commands.0.calls().iter().filter(|call| *call == "getent passwd").count(), 2);
}
