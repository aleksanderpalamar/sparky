use super::{Event, Form, Identity, Outcome, Phase, Request, Secret};

fn request() -> Request {
    Request {
        action_id: "org.freedesktop.udisks2.filesystem-mount-system".to_owned(),
        message: "Autentique-se para montar Dados".to_owned(),
        cookie: Secret::new("cookie-de-teste".to_owned()),
        identities: vec![
            Identity {
                uid: 1000,
                name: "palamar".to_owned(),
            },
            Identity {
                uid: 0,
                name: "root".to_owned(),
            },
        ],
    }
}

#[test]
fn senhas_e_cookies_nao_aparecem_no_debug() {
    let secret = Secret::new("senha-nao-publica".to_owned());
    assert!(!format!("{secret:?}").contains(secret.expose()));
    assert!(!format!("{:?}", request()).contains("cookie-de-teste"));
}

#[test]
fn pedido_exige_cookie_e_identidades_validas() {
    assert!(request().validate().is_ok());
    let mut invalid = request();
    invalid.cookie = Secret::default();
    assert!(invalid.validate().is_err());
    invalid = request();
    invalid.identities.clear();
    assert!(invalid.validate().is_err());
    invalid = request();
    invalid.identities[0].uid = u32::MAX;
    assert!(invalid.validate().is_err());
}

#[test]
fn cookie_com_quebra_de_linha_nao_chega_ao_helper() {
    let mut invalid = request();
    invalid.cookie = Secret::new("cookie\ninjetado".to_owned());
    assert!(invalid.validate().is_err());
}

#[test]
fn somente_usuarios_oferecidos_pelo_polkit_podem_ser_escolhidos() {
    let mut form = Form::new(request(), 1000).unwrap();
    assert_eq!(form.identity.uid, 1000);
    assert!(!form.select(999));
    assert_eq!(form.identity.uid, 1000);
    assert!(form.select(0));
    assert_eq!(form.identity.uid, 0);
}

#[test]
fn senha_fica_oculta_e_e_retirada_do_formulario_ao_enviar() {
    let mut form = Form::new(request(), 1000).unwrap();
    assert_eq!(form.phase, Phase::Waiting);
    form.receive(Event::Prompt {
        text: "Senha:".to_owned(),
        echo: false,
    });
    assert_eq!(form.phase, Phase::Input { echo: false });
    form.type_in(Secret::new("segredo".to_owned()));
    let response = form.submit().unwrap();
    assert_eq!(response.expose(), "segredo");
    assert!(form.response().is_empty());
    assert_eq!(form.phase, Phase::Verifying);
    assert!(form.submit().is_none());
}

#[test]
fn pam_pode_pedir_uma_resposta_visivel_ou_vazia() {
    let mut form = Form::new(request(), 1000).unwrap();
    form.receive(Event::Prompt {
        text: "Nome:".to_owned(),
        echo: true,
    });
    assert_eq!(form.phase, Phase::Input { echo: true });
    assert_eq!(form.submit().unwrap().expose(), "");
}

#[test]
fn respostas_nao_podem_injetar_linhas_no_protocolo_pam() {
    let mut form = Form::new(request(), 1000).unwrap();
    form.receive(Event::Prompt {
        text: "Senha:".to_owned(),
        echo: false,
    });
    for invalid in [
        "primeira\nsegunda",
        "primeira\rsegunda",
        "primeira\0segunda",
    ] {
        form.type_in(Secret::new(invalid.to_owned()));
        assert!(form.response().is_empty());
        assert!(form.problem.is_some());
    }
}

#[test]
fn falha_limpa_a_resposta_e_permite_tres_tentativas() {
    let mut form = Form::new(request(), 1000).unwrap();
    for _ in 0..2 {
        form.receive(Event::Prompt {
            text: "Senha:".to_owned(),
            echo: false,
        });
        form.type_in(Secret::new("incorreta".to_owned()));
        assert_eq!(form.receive(Event::Completed(false)), None);
        assert!(form.response().is_empty());
        assert_eq!(form.phase, Phase::Retry);
        assert!(form.retry());
    }
    assert_eq!(form.receive(Event::Completed(false)), Some(Outcome::Denied));
}

#[test]
fn somente_a_conclusao_do_backend_pode_confirmar_a_autenticacao() {
    let mut form = Form::new(request(), 1000).unwrap();
    assert_eq!(form.receive(Event::Info("Verificando".to_owned())), None);
    assert_eq!(
        form.receive(Event::Completed(true)),
        Some(Outcome::Authorized)
    );
    assert_eq!(Outcome::from_code(Some(0)), Outcome::Authorized);
    assert_eq!(Outcome::from_code(None), Outcome::Denied);
    assert_eq!(Outcome::from_code(Some(1)), Outcome::Cancelled);
}
