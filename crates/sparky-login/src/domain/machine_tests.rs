use sparky_login_core::protocol::{Refusal, Reply, Request};

use crate::domain::machine::{Effect, Event, Machine};
use crate::domain::machine_support::{login, maquina, pedido};
use crate::domain::memory::Memory;

#[test]
fn ao_ligar_abre_a_tela() {
    let mut machine = Machine::new(Vec::new(), Memory::default());
    assert_eq!(machine.on(Event::Started), vec![Effect::StartGreeter]);
}

#[test]
fn ola_responde_o_ultimo_usuario_e_a_ultima_sessao() {
    let welcome = Reply::Welcome {
        last_user: Some("palamar".into()),
        last_session: Some("sparky".into()),
    };
    assert_eq!(
        maquina().on(Event::Request(Request::Hello)),
        vec![Effect::Reply(welcome)]
    );
}

#[test]
fn login_com_sessao_instalada_vai_para_a_conferencia() {
    assert_eq!(
        maquina().on(pedido("palamar", "hyprland")),
        vec![login("hyprland")]
    );
}

#[test]
fn sessao_que_nao_esta_instalada_e_recusada_sem_tentar() {
    let refusal = Refusal::Unavailable("a sessão gnome não está instalada".into());
    assert_eq!(
        maquina().on(pedido("palamar", "gnome")),
        vec![Effect::Reply(Reply::Refused { refusal })]
    );
}

#[test]
fn usuario_vazio_e_recusado_sem_tentar() {
    assert_eq!(
        maquina().on(pedido("  ", "sparky")),
        vec![Effect::Reply(Reply::Refused {
            refusal: Refusal::Denied
        })]
    );
}

#[test]
fn senha_errada_responde_e_aceita_outra_tentativa() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "sparky"));
    assert_eq!(
        machine.on(Event::Refused(Refusal::Denied)),
        vec![Effect::Reply(Reply::Refused {
            refusal: Refusal::Denied
        })]
    );
    assert_eq!(
        machine.on(pedido("palamar", "sparky")),
        vec![login("sparky")]
    );
}

#[test]
fn pedido_durante_a_conferencia_espera_a_vez() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "sparky"));
    let busy = Reply::Failed {
        message: "já há um login em andamento".into(),
    };
    assert_eq!(
        machine.on(Event::Request(Request::Hello)),
        vec![Effect::Reply(busy)]
    );
}

#[test]
fn senha_certa_lembra_fecha_a_tela_e_so_depois_abre_a_sessao() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "hyprland"));
    let memory = Memory {
        last_user: Some("palamar".into()),
        last_session: Some("hyprland".into()),
    };
    assert_eq!(
        machine.on(Event::Authenticated),
        vec![
            Effect::Reply(Reply::LoggedIn),
            Effect::Remember(memory),
            Effect::StopGreeter
        ]
    );
    assert_eq!(machine.on(Event::GreeterGone), vec![Effect::StartSession]);
    assert_eq!(machine.on(Event::SessionEnded), vec![Effect::StartGreeter]);
}
