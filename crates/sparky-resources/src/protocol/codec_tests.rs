use std::io::Cursor;

use super::codec::{LineError, decode, read_line, write};
use super::message::{Refusal, Request};
use crate::domain::Mode;

#[test]
fn le_uma_linha_por_vez() {
    let mut input = Cursor::new(b"um\ndois\n".to_vec());
    assert_eq!(read_line(&mut input, 16).ok(), Some(b"um".to_vec()));
    assert_eq!(read_line(&mut input, 16).ok(), Some(b"dois".to_vec()));
    assert!(matches!(read_line(&mut input, 16), Err(LineError::Closed)));
}

#[test]
fn ultima_linha_sem_quebra_ainda_vale() {
    let mut input = Cursor::new(b"fim".to_vec());
    assert_eq!(read_line(&mut input, 16).ok(), Some(b"fim".to_vec()));
}

#[test]
fn linha_acima_do_limite_e_recusada_sem_ler_tudo() {
    let mut input = Cursor::new(vec![b'a'; 64]);
    assert!(matches!(read_line(&mut input, 8), Err(LineError::TooLarge)));
    let mut exact = Cursor::new(b"12345678\n".to_vec());
    assert_eq!(read_line(&mut exact, 8).ok(), Some(b"12345678".to_vec()));
}

#[test]
fn o_pedido_vai_e_volta_com_a_versao() {
    let mut output = Vec::new();
    assert!(
        write(
            &mut output,
            Request::SetMode {
                mode: Mode::Conservative
            }
        )
        .is_ok()
    );
    assert_eq!(
        output,
        b"{\"version\":1,\"request\":\"set_mode\",\"mode\":\"conservative\"}\n".to_vec()
    );
    let line = &output[..output.len() - 1];
    assert_eq!(
        decode::<Request>(line),
        Ok(Request::SetMode {
            mode: Mode::Conservative
        })
    );
}

#[test]
fn versao_ausente_ou_diferente_e_recusada() {
    assert_eq!(
        decode::<Request>(br#"{"request":"status"}"#),
        Err(Refusal::UnsupportedVersion)
    );
    assert_eq!(
        decode::<Request>(br#"{"version":2,"request":"status"}"#),
        Err(Refusal::UnsupportedVersion)
    );
}

#[test]
fn json_invalido_ou_pedido_desconhecido_e_malformado() {
    assert_eq!(decode::<Request>(b"oi"), Err(Refusal::Malformed));
    assert_eq!(
        decode::<Request>(br#"{"version":1,"request":"matar","pid":1}"#),
        Err(Refusal::Malformed)
    );
}
