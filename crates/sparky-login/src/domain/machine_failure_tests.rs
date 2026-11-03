use sparky_login_core::protocol::{Power, Refusal, Reply, Request};

use crate::domain::machine::{Effect, Event};
use crate::domain::machine_support::{maquina, pedido};

#[test]
fn tela_que_cai_volta_sozinha() {
    assert_eq!(maquina().on(Event::GreeterGone), vec![Effect::StartGreeter]);
}

#[test]
fn tela_que_cai_durante_a_conferencia_cancela_o_login() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "sparky"));
    assert_eq!(
        machine.on(Event::GreeterGone),
        vec![Effect::Abandon, Effect::StartGreeter]
    );
    assert_eq!(machine.on(Event::Request(Request::Hello)).len(), 1);
}

#[test]
fn worker_que_morre_na_conferencia_vira_recusa() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "sparky"));
    let refusal = Refusal::Unavailable("o login parou no meio".into());
    assert_eq!(
        machine.on(Event::SessionEnded),
        vec![Effect::Reply(Reply::Refused { refusal })]
    );
}

#[test]
fn worker_que_morre_na_passagem_reabre_a_tela_quando_ela_fechar() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "sparky"));
    machine.on(Event::Authenticated);
    assert_eq!(machine.on(Event::SessionEnded), Vec::new());
    assert_eq!(machine.on(Event::GreeterGone), vec![Effect::StartGreeter]);
}

#[test]
fn energia_vai_ao_logind_e_a_resposta_volta_para_a_tela() {
    let mut machine = maquina();
    assert_eq!(
        machine.on(Event::Request(Request::Power {
            action: Power::Reboot
        })),
        vec![Effect::Power(Power::Reboot)]
    );
    assert_eq!(
        machine.on(Event::Powered(Ok(()))),
        vec![Effect::Reply(Reply::Done)]
    );
    let failed = Reply::Failed {
        message: "sem permissão".into(),
    };
    assert_eq!(
        machine.on(Event::Powered(Err("sem permissão".into()))),
        vec![Effect::Reply(failed)]
    );
}

#[test]
fn durante_a_sessao_nao_ha_tela_para_responder() {
    let mut machine = maquina();
    machine.on(pedido("palamar", "sparky"));
    machine.on(Event::Authenticated);
    machine.on(Event::GreeterGone);
    assert_eq!(machine.on(Event::Request(Request::Hello)), Vec::new());
}
