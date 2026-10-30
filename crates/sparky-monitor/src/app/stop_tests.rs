use std::sync::Arc;

use sparky_ui::notice::Notice;

use super::actions::question;
use super::message::{Message, Selection};
use super::page::Page;
use super::state::Sampling;
use super::test_support::{Sinais, enviar, monitor};
use crate::ports::Stop;

#[test]
fn encerrar_pede_confirmacao_e_envia_o_sinal_so_depois_do_enter() {
    let sinais = Arc::new(Sinais::default());
    let mut monitor = monitor(sinais.clone());
    enviar(&mut monitor, vec![Message::Ask(Stop::Terminate)]);
    assert!(matches!(&monitor.notice, Some(Notice::Error(_))));
    enviar(
        &mut monitor,
        vec![
            Message::Select(Selection::Process(200)),
            Message::Ask(Stop::Kill),
        ],
    );
    let pedido = monitor.confirm.as_ref().map(question);
    assert_eq!(
        pedido.as_deref(),
        Some("Forçar o fim de zsh (PID 200)? Enter confirma · Esc cancela")
    );
    assert!(sinais.enviados.lock().unwrap().is_empty());
    enviar(&mut monitor, vec![Message::Confirm]);
    assert_eq!(*sinais.enviados.lock().unwrap(), [(200, Stop::Kill)]);
    assert_eq!(
        monitor.notice,
        Some(Notice::Info(
            "zsh (PID 200) foi forçado a fechar".to_owned()
        ))
    );
}

#[test]
fn esc_cancela_e_aplicativo_encerra_todos_os_processos_dele() {
    let sinais = Arc::new(Sinais::default());
    let mut monitor = monitor(sinais.clone());
    enviar(
        &mut monitor,
        vec![
            Message::Select(Selection::App("firefox".to_owned())),
            Message::Ask(Stop::Terminate),
            Message::Escape,
        ],
    );
    assert!(monitor.confirm.is_none());
    assert_eq!(monitor.page, Page::Dashboard);
    enviar(
        &mut monitor,
        vec![Message::Ask(Stop::Terminate), Message::Confirm],
    );
    assert_eq!(
        *sinais.enviados.lock().unwrap(),
        [(100, Stop::Terminate), (101, Stop::Terminate)]
    );
    assert_eq!(
        monitor.notice,
        Some(Notice::Info(
            "Pedido de encerramento enviado a Firefox (2 processos)".to_owned()
        ))
    );
}

#[test]
fn processo_de_outro_usuario_mostra_o_erro_e_amostragem_nao_se_sobrepoe() {
    let sinais = Arc::new(Sinais {
        negados: vec![1],
        ..Sinais::default()
    });
    let mut monitor = monitor(sinais);
    enviar(
        &mut monitor,
        vec![
            Message::Select(Selection::Process(1)),
            Message::Ask(Stop::Terminate),
            Message::Confirm,
        ],
    );
    assert_eq!(
        monitor.notice,
        Some(Notice::Error(
            "Sem permissão para encerrar este processo".to_owned()
        ))
    );
    let _ = monitor.tick();
    assert_eq!(monitor.sampling, Sampling::Busy);
    let segunda = monitor.tick();
    assert!(iced_runtime::task::into_stream(segunda).is_none());
}

#[test]
fn processo_que_termina_com_a_pergunta_aberta_nao_tem_o_pid_reaproveitado_atingido() {
    let sinais = Arc::new(Sinais::default());
    let mut monitor = monitor(sinais.clone());
    enviar(
        &mut monitor,
        vec![
            Message::Select(Selection::Process(200)),
            Message::Ask(Stop::Kill),
        ],
    );
    sinais.mortos.lock().unwrap().push(200);
    enviar(&mut monitor, vec![Message::Confirm]);
    assert!(sinais.enviados.lock().unwrap().is_empty());
    assert_eq!(
        monitor.notice,
        Some(Notice::Error("O processo já terminou".to_owned()))
    );
}

#[test]
fn processo_que_ja_terminou_antes_da_pergunta_nao_abre_confirmacao() {
    let sinais = Arc::new(Sinais::default());
    let mut monitor = monitor(sinais.clone());
    sinais.mortos.lock().unwrap().push(200);
    enviar(
        &mut monitor,
        vec![
            Message::Select(Selection::Process(200)),
            Message::Ask(Stop::Terminate),
        ],
    );
    assert!(monitor.confirm.is_none());
    assert_eq!(
        monitor.notice,
        Some(Notice::Error("O processo já terminou".to_owned()))
    );
}
