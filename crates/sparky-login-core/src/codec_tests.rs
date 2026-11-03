use std::io::{BufReader, Cursor};

use zeroize::Zeroizing;

use crate::codec::{CodecError, LIMIT, receive, send};
use crate::protocol::{Power, Refusal, Reply, Request};

fn ida_e_volta(reply: &Reply) -> Option<Reply> {
    let mut bytes = Vec::new();
    send(&mut bytes, reply).ok()?;
    receive(&mut BufReader::new(Cursor::new(bytes))).ok()?
}

#[test]
fn as_respostas_vao_e_voltam_iguais() {
    let respostas = [
        Reply::Welcome {
            last_user: Some("palamar".into()),
            last_session: None,
        },
        Reply::LoggedIn,
        Reply::Refused {
            refusal: Refusal::Denied,
        },
        Reply::Refused {
            refusal: Refusal::Unavailable("sem sessão".into()),
        },
        Reply::Done,
        Reply::Failed {
            message: "logind fora do ar".into(),
        },
    ];
    for reply in respostas {
        assert_eq!(ida_e_volta(&reply), Some(reply));
    }
}

#[test]
fn o_pedido_de_login_leva_usuario_senha_e_sessao_numa_linha() {
    let pedido = Request::Login {
        username: "palamar".into(),
        password: Zeroizing::new("segredo".into()),
        session: "sparky".into(),
    };
    let mut bytes = Vec::new();
    assert!(send(&mut bytes, &pedido).is_ok());
    assert_eq!(bytes.iter().filter(|byte| **byte == b'\n').count(), 1);
    let lido: Option<Request> = receive(&mut BufReader::new(Cursor::new(bytes))).unwrap_or(None);
    let Some(Request::Login {
        username,
        password,
        session,
    }) = lido
    else {
        panic!("deveria ler o pedido de login");
    };
    assert_eq!(
        (username.as_str(), password.as_str(), session.as_str()),
        ("palamar", "segredo", "sparky")
    );
}

#[test]
fn energia_e_ola_tem_nomes_estaveis_no_fio() {
    let mut bytes = Vec::new();
    assert!(
        send(
            &mut bytes,
            &Request::Power {
                action: Power::Suspend
            }
        )
        .is_ok()
    );
    assert!(send(&mut bytes, &Request::Hello).is_ok());
    let texto = String::from_utf8(bytes).unwrap_or_default();
    assert_eq!(
        texto,
        "{\"request\":\"power\",\"action\":\"suspend\"}\n{\"request\":\"hello\"}\n"
    );
}

#[test]
fn fim_da_conexao_nao_e_erro() {
    let lido: Result<Option<Reply>, CodecError> =
        receive(&mut BufReader::new(Cursor::new(Vec::new())));
    assert!(matches!(lido, Ok(None)));
}

#[test]
fn linha_grande_demais_e_recusada() {
    let mut bytes = vec![b'a'; LIMIT + 10];
    bytes.push(b'\n');
    let lido: Result<Option<Reply>, CodecError> = receive(&mut BufReader::new(Cursor::new(bytes)));
    assert!(matches!(lido, Err(CodecError::TooLong)));
}

#[test]
fn lixo_e_mensagem_invalida() {
    let lido: Result<Option<Reply>, CodecError> =
        receive(&mut BufReader::new(Cursor::new(b"{oi}\n".to_vec())));
    assert!(matches!(lido, Err(CodecError::Invalid(_))));
}
