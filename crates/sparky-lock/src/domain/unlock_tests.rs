use sparky_login_core::protocol::Refusal;
use sparky_login_look::reveal::Reveal;

use crate::domain::unlock::{Status, Unlock};

fn digitado(senha: &str) -> Unlock {
    let mut unlock = Unlock::default();
    unlock.type_password(senha.to_owned());
    unlock
}

#[test]
fn so_desbloqueia_com_senha() {
    let mut unlock = Unlock::default();
    assert!(!unlock.ready());
    assert!(unlock.submit().is_none());
    unlock.type_password("segredo".to_owned());
    assert!(unlock.ready());
}

#[test]
fn desbloquear_entrega_a_senha_e_espera_a_resposta() {
    let mut unlock = digitado("segredo");
    let Some(senha) = unlock.submit() else {
        panic!("deveria entregar a senha");
    };
    assert_eq!(senha.as_str(), "segredo");
    assert_eq!(unlock.status(), &Status::Waiting);
    assert!(unlock.submit().is_none());
}

#[test]
fn enquanto_confere_o_que_se_digita_e_ignorado() {
    let mut unlock = digitado("segredo");
    unlock.submit();
    unlock.type_password("outra".to_owned());
    assert_eq!(unlock.password(), "segredo");
}

#[test]
fn senha_errada_apaga_a_senha_e_avisa() {
    let mut unlock = digitado("errada");
    unlock.submit();
    unlock.refuse(Refusal::Denied);
    assert_eq!(unlock.password(), "");
    assert_eq!(unlock.warning().as_deref(), Some("Senha incorreta!"));
}

#[test]
fn a_mensagem_do_pam_aparece_como_aviso() {
    let mut unlock = digitado("segredo");
    unlock.submit();
    unlock.refuse(Refusal::Unavailable(
        "The account is locked due to 3 failed logins.".into(),
    ));
    assert_eq!(
        unlock.warning().as_deref(),
        Some("The account is locked due to 3 failed logins.")
    );
}

#[test]
fn digitar_de_novo_apaga_o_aviso() {
    let mut unlock = digitado("errada");
    unlock.submit();
    unlock.refuse(Refusal::Denied);
    unlock.type_password("s".to_owned());
    assert_eq!(unlock.warning(), None);
    assert_eq!(unlock.status(), &Status::Editing);
}

#[test]
fn esc_limpa_a_senha_e_o_aviso() {
    let mut unlock = digitado("errada");
    unlock.submit();
    unlock.refuse(Refusal::Denied);
    unlock.type_password("quase".to_owned());
    unlock.clear();
    assert_eq!(unlock.password(), "");
    assert_eq!(unlock.warning(), None);
}

#[test]
fn esc_nao_cancela_a_conferencia_em_andamento() {
    let mut unlock = digitado("segredo");
    unlock.submit();
    unlock.clear();
    assert_eq!(unlock.status(), &Status::Waiting);
}

#[test]
fn o_olho_mostra_e_esconde_a_senha() {
    let mut unlock = Unlock::default();
    assert_eq!(unlock.reveal(), Reveal::Hidden);
    unlock.toggle_reveal();
    assert_eq!(unlock.reveal(), Reveal::Shown);
}
