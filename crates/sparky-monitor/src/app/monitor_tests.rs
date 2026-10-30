use std::sync::Arc;

use super::actions::question;
use super::message::{Message, Move, Selection};
use super::page::Page;
use super::state::{Focus, Sampling};
use super::test_support::{Sinais, monitor};
use super::update::update;
use crate::domain::table::{Column, Order};
use crate::ports::Stop;
use sparky_ui::notice::Notice;

fn enviar(monitor: &mut super::state::Monitor, messages: Vec<Message>) {
    for message in messages {
        let _ = update(monitor, message);
    }
}

#[test]
fn processos_e_aplicativos_chegam_ordenados_por_cpu() {
    let monitor = monitor(Arc::new(Sinais::default()));
    let nomes: Vec<_> = monitor
        .processes
        .iter()
        .map(|row| row.name.as_str())
        .collect();
    assert_eq!(nomes, ["Web Content", "firefox", "systemd", "zsh"]);
    assert_eq!(monitor.processes[0].usage.cpu, 20.0);
    assert_eq!(monitor.apps.len(), 1);
    assert_eq!(monitor.apps[0].pids, [100, 101]);
    assert_eq!(monitor.apps[0].usage.memory, 500 * 4096);
    assert_eq!(monitor.histories.cpu.values().count(), 2);
}

#[test]
fn letras_ordenam_e_paginas_de_recurso_ordenam_pelo_recurso() {
    let mut monitor = monitor(Arc::new(Sinais::default()));
    enviar(&mut monitor, vec![Message::SortBy(Column::Name)]);
    assert_eq!(monitor.processes[0].name, "firefox");
    enviar(&mut monitor, vec![Message::Open(Page::Memory)]);
    assert_eq!(
        (monitor.sort.column, monitor.sort.order),
        (Column::Memory, Order::Descending)
    );
    assert_eq!(monitor.processes[0].name, "firefox");
    enviar(&mut monitor, vec![Message::Escape]);
    assert_eq!(monitor.page, Page::Dashboard);
}

#[test]
fn selecao_segue_o_pid_e_tab_alterna_as_tabelas_do_painel() {
    let mut monitor = monitor(Arc::new(Sinais::default()));
    enviar(
        &mut monitor,
        vec![Message::Move(Move::Down), Message::Move(Move::Down)],
    );
    assert_eq!(monitor.selected_process, Some(100));
    enviar(
        &mut monitor,
        vec![Message::SortBy(Column::Name), Message::Move(Move::Down)],
    );
    assert_eq!(monitor.selected_process, Some(1));
    enviar(
        &mut monitor,
        vec![Message::SwitchFocus, Message::Move(Move::Last)],
    );
    assert_eq!(monitor.focus, Focus::Applications);
    assert_eq!(monitor.selected_app.as_deref(), Some("firefox"));
    enviar(&mut monitor, vec![Message::Select(Selection::Process(200))]);
    assert_eq!(
        (monitor.focus, monitor.selected_process),
        (Focus::Processes, Some(200))
    );
}

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
