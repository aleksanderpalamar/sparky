use std::time::{Duration, Instant};

use iced::futures::{StreamExt, channel::mpsc, executor::block_on};
use iced_runtime::{Action, clipboard, core::clipboard::Kind, task};

use super::message::Message;
use super::update::update;
use super::workspace::Workspace;
use super::workspace_tests::workspace;
use crate::adapters::listener::Listener;
use crate::adapters::session::Session;
use crate::domain::grid::{Half, Spot};
use crate::domain::launch::{Launch, Program};
use crate::domain::tabs::TabId;

fn start(app: &mut Workspace, id: TabId, script: &str) {
    let Some(tab) = app.tabs.get_mut(id) else {
        panic!("sem aba")
    };
    let (sender, _receiver) = mpsc::unbounded();
    let launch = Launch {
        program: Program::Command {
            program: "sh".into(),
            args: vec!["-c".into(), script.into()],
        },
        directory: None,
    };
    let Ok(session) = Session::start(&launch, tab.value.size, Listener(sender)) else {
        panic!("sem PTY privado")
    };
    tab.value.session = Some(session);
}

fn text(app: &Workspace, id: TabId) -> String {
    let Some(tab) = app.tabs.get(id) else {
        panic!("sem aba")
    };
    let Some(session) = &tab.value.session else {
        panic!("sem sessão")
    };
    session
        .screen(&app.palette)
        .rows
        .iter()
        .flatten()
        .map(|run| run.text.as_str())
        .collect()
}

fn wait_for(app: &Workspace, id: TabId, expected: &str) {
    let start = Instant::now();
    while !text(app, id).contains(expected) && start.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(text(app, id).contains(expected), "não recebeu {expected:?}");
}

#[test]
fn cada_aba_tem_seu_pty_e_a_colagem_chega_a_aba_original() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    start(
        &mut app,
        first,
        "printf 'A-pronta\\n'; read -r linha; printf 'A:%s\\n' \"$linha\"; read -r fim",
    );
    let Some(second) = app.new_tab() else {
        panic!("sem segunda")
    };
    start(
        &mut app,
        second,
        "printf 'B-pronta\\n'; read -r linha; printf 'B:%s\\n' \"$linha\"; read -r fim",
    );
    wait_for(&app, first, "A-pronta");
    wait_for(&app, second, "B-pronta");
    let _ = update(&mut app, Message::Pasted(first, Some("primeira\n".into())));
    wait_for(&app, first, "A:primeira");
    assert!(!text(&app, second).contains("primeira"));
    assert_eq!(app.tabs.active_id(), Some(second));
    let _ = update(&mut app, Message::Commit("segunda\n".into()));
    wait_for(&app, second, "B:segunda");
    assert!(!text(&app, first).contains("segunda"));
}

#[test]
fn selecionar_texto_publica_na_primaria_sem_sobrescrever_o_clipboard_comum() {
    let mut app = workspace();
    let Some(id) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    start(&mut app, id, "printf 'A-pronta\\n'; read -r fim");
    wait_for(&app, id, "A-pronta");
    let left = Spot {
        line: 0,
        column: 0,
        half: Half::Left,
    };
    let right = Spot {
        half: Half::Right,
        ..left
    };
    let _ = update(&mut app, Message::Press(id, left));
    let _ = update(&mut app, Message::Drag(id, right));
    let Some(mut stream) = task::into_stream(update(&mut app, Message::Selected(id))) else {
        panic!("sem cópia")
    };
    let Some(Action::Clipboard(clipboard::Action::Write { target, contents })) =
        block_on(stream.next())
    else {
        panic!("sem destino")
    };
    assert_eq!(target, Kind::Primary);
    assert_eq!(contents, "A");
}

#[test]
fn o_pty_continua_processando_saida_enquanto_a_aba_esta_inativa() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    start(
        &mut app,
        first,
        "printf 'pronta\\n'; sleep 0.1; printf 'trabalho concluído\\n'; read -r fim",
    );
    let Some(second) = app.new_tab() else {
        panic!("sem segunda")
    };
    wait_for(&app, first, "trabalho concluído");
    assert_eq!(app.tabs.active_id(), Some(second));
    app.select(first);
    let Some(tab) = app.tabs.active() else {
        panic!("sem aba")
    };
    let visible: String = tab
        .value
        .screen
        .rows
        .iter()
        .flatten()
        .map(|run| run.text.as_str())
        .collect();
    assert!(visible.contains("trabalho concluído"));
}
