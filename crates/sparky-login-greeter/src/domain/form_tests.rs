use super::focus::Focus;
use super::form::{Form, Refusal, Reveal, Status};

fn preenchido() -> Form {
    let mut form = Form::new(Some("palamar"));
    form.type_password("segredo".to_owned());
    form
}

#[test]
fn o_ultimo_usuario_ja_vem_preenchido_e_o_foco_comeca_na_senha() {
    let form = Form::new(Some("palamar"));
    assert_eq!(form.username(), "palamar");
    assert_eq!(form.first_focus(), Focus::Password);
}

#[test]
fn sem_ultimo_usuario_o_foco_comeca_no_usuario() {
    let form = Form::new(None);
    assert_eq!(form.username(), "");
    assert_eq!(form.first_focus(), Focus::Username);
}

#[test]
fn o_nome_de_usuario_fica_em_minusculas() {
    let mut form = Form::new(None);
    form.type_username("PalaMar".to_owned());
    assert_eq!(form.username(), "palamar");
}

#[test]
fn so_entra_com_usuario_e_senha() {
    let mut form = Form::new(None);
    assert!(!form.ready());
    form.type_username("palamar".to_owned());
    assert!(!form.ready());
    form.type_password("segredo".to_owned());
    assert!(form.ready());
}

#[test]
fn entrar_entrega_as_credenciais_e_espera_a_resposta() {
    let mut form = preenchido();
    let Some(credentials) = form.submit() else {
        panic!("o formulário preenchido deveria entregar as credenciais");
    };
    assert_eq!(credentials.username, "palamar");
    assert_eq!(credentials.password.as_str(), "segredo");
    assert_eq!(form.status(), &Status::Waiting);
    assert!(form.submit().is_none());
    assert!(!form.ready());
}

#[test]
fn enquanto_espera_o_texto_nao_muda() {
    let mut form = preenchido();
    form.submit();
    form.type_password("outra".to_owned());
    form.type_username("ana".to_owned());
    assert_eq!(form.password(), "segredo");
    assert_eq!(form.username(), "palamar");
}

#[test]
fn a_recusa_apaga_a_senha_e_mostra_o_aviso() {
    let mut form = preenchido();
    form.submit();
    form.refuse(Refusal::Denied);
    assert_eq!(form.password(), "");
    assert_eq!(form.username(), "palamar");
    assert_eq!(
        form.warning(),
        Some(sparky_login_look::i18n::text("login-denied"))
    );
}

#[test]
fn o_aviso_de_falha_tecnica_traz_o_motivo() {
    let mut form = preenchido();
    form.submit();
    form.refuse(Refusal::Unavailable(
        "o serviço de login não respondeu".to_owned(),
    ));
    assert_eq!(
        form.warning().as_deref(),
        Some("o serviço de login não respondeu")
    );
}

#[test]
fn digitar_de_novo_apaga_o_aviso() {
    let mut form = preenchido();
    form.submit();
    form.refuse(Refusal::Denied);
    form.type_password("s".to_owned());
    assert_eq!(form.warning(), None);
    assert_eq!(form.status(), &Status::Editing);
}

#[test]
fn o_olho_mostra_e_esconde_a_senha() {
    let mut form = preenchido();
    assert_eq!(form.reveal(), Reveal::Hidden);
    form.toggle_reveal();
    assert_eq!(form.reveal(), Reveal::Shown);
    form.toggle_reveal();
    assert_eq!(form.reveal(), Reveal::Hidden);
}
