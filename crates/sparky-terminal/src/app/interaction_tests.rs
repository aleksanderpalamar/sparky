use alacritty_terminal::term::ClipboardType;
use iced::futures::{StreamExt, executor::block_on};
use iced::keyboard::{Key, Modifiers};
use iced_runtime::core::clipboard::Kind;
use iced_runtime::{Action, clipboard, task};

use super::message::Message;
use super::update::update;
use super::workspace_tests::workspace;
use crate::adapters::listener::Signal;

#[test]
fn criar_e_fechar_abas_pelo_teclado_nao_depende_do_programa_em_execucao() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    let key = |letter: &str| {
        Message::Key(
            Key::Character(letter.into()),
            Modifiers::CTRL | Modifiers::SHIFT,
            None,
            None,
        )
    };
    let _ = update(&mut app, key("t"));
    assert_eq!(app.tabs.iter().len(), 2);
    assert_ne!(app.tabs.active_id(), Some(first));
    let _ = update(&mut app, key("q"));
    assert_eq!(app.tabs.active_id(), Some(first));
    let Some(stream) = task::into_stream(update(&mut app, key("q"))) else {
        panic!("sem efeito")
    };
    let actions = block_on(stream.collect::<Vec<_>>());
    assert!(actions.iter().any(|action| matches!(action, Action::Exit)));
}

#[test]
fn terminar_uma_aba_inativa_nao_fecha_a_janela_nem_muda_o_titulo_da_ativa() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    let Some(second) = app.new_tab() else {
        panic!("sem segunda")
    };
    let _ = update(
        &mut app,
        Message::Pty(first, Signal::Title("compilando".into())),
    );
    assert_eq!(app.title(), "Terminal");
    assert_eq!(
        app.tabs.get(first).map(|tab| tab.value.title.as_str()),
        Some("compilando")
    );
    let _ = update(&mut app, Message::Pty(first, Signal::Exited));
    assert_eq!(app.tabs.active_id(), Some(second));
    assert!(task::into_stream(update(&mut app, Message::Pty(first, Signal::Exited))).is_none());
    let _ = update(
        &mut app,
        Message::Pty(first, Signal::Title("evento atrasado".into())),
    );
    assert_eq!(app.title(), "Terminal");
}

#[test]
fn a_leitura_assincrona_do_clipboard_preserva_a_aba_que_pediu_a_colagem() {
    for primary in [false, true] {
        let mut app = workspace();
        let Some(id) = app.tabs.active_id() else {
            panic!("sem aba")
        };
        let request = match primary {
            true => Message::PastePrimary(id),
            false => Message::Key(
                Key::Character("v".into()),
                Modifiers::CTRL | Modifiers::SHIFT,
                None,
                None,
            ),
        };
        let Some(mut stream) = task::into_stream(update(&mut app, request)) else {
            panic!("sem leitura")
        };
        let Some(Action::Clipboard(clipboard::Action::Read { target, channel })) =
            block_on(stream.next())
        else {
            panic!("não pediu clipboard")
        };
        assert_eq!(
            target,
            if primary {
                Kind::Primary
            } else {
                Kind::Standard
            }
        );
        app.new_tab();
        assert!(channel.send(Some("conteúdo simulado".into())).is_ok());
        let Some(Action::Output(Message::Pasted(found, text))) = block_on(stream.next()) else {
            panic!("sem resposta")
        };
        assert_eq!(found, id);
        assert_eq!(text.as_deref(), Some("conteúdo simulado"));
        assert_ne!(app.tabs.active_id(), Some(found));
    }
}

#[test]
fn osc52_preserva_a_distincao_entre_clipboard_e_selecao_primaria() {
    for (kind, expected) in [
        (ClipboardType::Clipboard, Kind::Standard),
        (ClipboardType::Selection, Kind::Primary),
    ] {
        let mut app = workspace();
        let Some(id) = app.tabs.active_id() else {
            panic!("sem aba")
        };
        let signal = Signal::Store(kind, "texto".into());
        let Some(mut stream) = task::into_stream(update(&mut app, Message::Pty(id, signal))) else {
            panic!("sem escrita")
        };
        let Some(Action::Clipboard(clipboard::Action::Write { target, contents })) =
            block_on(stream.next())
        else {
            panic!("sem destino")
        };
        assert_eq!(target, expected);
        assert_eq!(contents, "texto");
    }
}

#[test]
fn colagem_atrasada_para_aba_fechada_nao_cria_nem_modifica_outra_aba() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    let Some(second) = app.new_tab() else {
        panic!("sem segunda")
    };
    app.close(first);
    let _ = update(&mut app, Message::Pasted(first, Some("não enviar".into())));
    assert_eq!(app.tabs.active_id(), Some(second));
    assert_eq!(app.tabs.iter().len(), 1);
    assert!(app.tabs.get(first).is_none());
}
