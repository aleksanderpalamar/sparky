use sparky_login_core::protocol::Request;
use sparky_login_core::sessions::Session;
use zeroize::Zeroizing;

use crate::domain::machine::{Effect, Event, Login, Machine};
use crate::domain::memory::Memory;

pub fn sessao(id: &str) -> Session {
    Session {
        id: id.to_owned(),
        name: id.to_owned(),
        exec: id.to_owned(),
        desktop_names: Vec::new(),
    }
}

pub fn lembrado() -> Memory {
    Memory {
        last_user: Some("palamar".into()),
        last_session: Some("sparky".into()),
    }
}

pub fn maquina() -> Machine {
    let mut machine = Machine::new(vec![sessao("hyprland"), sessao("sparky")], lembrado());
    machine.on(Event::Started);
    machine
}

pub fn pedido(username: &str, session: &str) -> Event {
    Event::Request(Request::Login {
        username: username.into(),
        password: Zeroizing::new("segredo".into()),
        session: session.into(),
    })
}

pub fn login(session: &str) -> Effect {
    Effect::Authenticate(Login {
        username: "palamar".into(),
        password: Zeroizing::new("segredo".into()),
        session: sessao(session),
    })
}
