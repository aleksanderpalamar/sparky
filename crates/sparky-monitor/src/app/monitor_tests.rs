use std::sync::Arc;

use super::message::{Message, Move, Selection};
use super::page::Page;
use super::state::Focus;
use super::test_support::{Sinais, enviar, monitor};
use crate::domain::table::{Column, Order};

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
fn roda_do_mouse_move_a_tabela_sob_o_ponteiro() {
    let mut monitor = monitor(Arc::new(Sinais::default()));
    assert_eq!(monitor.focus, Focus::Processes);
    enviar(
        &mut monitor,
        vec![Message::Scrolled(Focus::Applications, 1)],
    );
    assert_eq!(monitor.selected_app.as_deref(), Some("firefox"));
    assert_eq!(monitor.selected_process, None);
    assert_eq!(monitor.focus, Focus::Applications);
    enviar(&mut monitor, vec![Message::Scrolled(Focus::Processes, 2)]);
    assert_eq!(monitor.selected_process, Some(100));
    assert_eq!(monitor.focus, Focus::Processes);
}
