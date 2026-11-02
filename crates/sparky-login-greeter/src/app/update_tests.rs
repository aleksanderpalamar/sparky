use std::sync::Arc;

use crate::adapters::preview::Preview;
use crate::app::update::update;
use crate::app::{Greeter, Message, Start, Step, boot};
use crate::domain::focus::Focus;
use crate::domain::form::{Refusal, Status};
use crate::domain::power::Power;
use crate::domain::sessions::{Session, Sessions};

fn sessoes() -> Option<Sessions> {
    Sessions::new(
        vec![
            Session {
                id: "hyprland".into(),
                name: "Hyprland".into(),
            },
            Session {
                id: "sparky".into(),
                name: "Sparky".into(),
            },
        ],
        None,
    )
}

fn tela(sessions: Option<Sessions>) -> Greeter {
    let start = Start {
        backend: Arc::new(Preview),
        sessions,
        last_user: Some("palamar".into()),
        wallpaper: None,
        palette: sparky_theme::Palette::TOKYO_NIGHT,
    };
    boot(start).0
}

#[test]
fn enter_no_usuario_vai_para_a_senha() {
    let mut greeter = tela(sessoes());
    let _ = update(&mut greeter, Message::Submitted(Focus::Username));
    assert_eq!(greeter.focus, Focus::Password);
}

#[test]
fn enter_na_senha_entrega_as_credenciais() {
    let mut greeter = tela(sessoes());
    let _ = update(&mut greeter, Message::TypedPassword("segredo".into()));
    let _ = update(&mut greeter, Message::Submitted(Focus::Password));
    assert_eq!(greeter.form.status(), &Status::Waiting);
}

#[test]
fn tab_e_shift_tab_andam_pela_tela() {
    let mut greeter = tela(sessoes());
    let _ = update(&mut greeter, Message::Move(Step::Forward));
    assert_eq!(greeter.focus, Focus::Login);
    let _ = update(&mut greeter, Message::Move(Step::Back));
    let _ = update(&mut greeter, Message::Move(Step::Back));
    assert_eq!(greeter.focus, Focus::Username);
}

#[test]
fn enter_na_sessao_troca_a_sessao() {
    let mut greeter = tela(sessoes());
    greeter.focus = Focus::Session;
    let _ = update(&mut greeter, Message::Activate);
    assert_eq!(
        greeter
            .sessions
            .as_ref()
            .map(|sessions| sessions.current().id.clone())
            .as_deref(),
        Some("hyprland")
    );
}

#[test]
fn enter_num_botao_de_energia_pede_a_acao_e_mantem_o_foco_nele() {
    let mut greeter = tela(sessoes());
    greeter.focus = Focus::Power(Power::Reboot);
    let _ = update(&mut greeter, Message::Activate);
    assert_eq!(greeter.focus, Focus::Power(Power::Reboot));
}

#[test]
fn a_recusa_volta_o_foco_para_a_senha_com_o_aviso() {
    let mut greeter = tela(sessoes());
    let _ = update(&mut greeter, Message::TypedPassword("errada".into()));
    let _ = update(&mut greeter, Message::Login);
    greeter.focus = Focus::Login;
    let _ = update(&mut greeter, Message::Answer(Err(Refusal::Denied)));
    assert_eq!(greeter.focus, Focus::Password);
    assert_eq!(greeter.form.password(), "");
    assert_eq!(greeter.form.warning().as_deref(), Some("Falha no login!"));
}

#[test]
fn sem_sessao_instalada_avisa_e_nao_tenta_entrar() {
    let mut greeter = tela(None);
    let _ = update(&mut greeter, Message::TypedPassword("segredo".into()));
    let _ = update(&mut greeter, Message::Login);
    assert_eq!(greeter.form.status(), &Status::Editing);
    assert_eq!(
        greeter.notice.as_deref(),
        Some("nenhuma sessão Wayland instalada")
    );
}

#[test]
fn a_falha_de_energia_aparece_como_aviso_e_some_ao_digitar() {
    let mut greeter = tela(sessoes());
    let _ = update(
        &mut greeter,
        Message::Powered(Err("na prévia, Desligar não faz nada".into())),
    );
    assert_eq!(
        greeter.notice.as_deref(),
        Some("na prévia, Desligar não faz nada")
    );
    let _ = update(&mut greeter, Message::TypedPassword("s".into()));
    assert_eq!(greeter.notice, None);
}
