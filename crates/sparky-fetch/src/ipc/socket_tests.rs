use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use sparky_ipc::codec::{decode, encode};
use sparky_ipc::{OutputInfo, Reply, Request, SessionInfo};

use crate::ipc::port::{Compositor, Problem};
use crate::ipc::socket::SocketCompositor;

const PRAZO: Duration = Duration::from_millis(200);

fn servidor(responder: fn(Request) -> Option<Reply>) -> (tempfile::TempDir, PathBuf) {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("sparky.sock");
    let ouvinte = UnixListener::bind(&caminho).unwrap_or_else(|error| panic!("{error}"));
    thread::spawn(move || {
        let Ok((conexao, _)) = ouvinte.accept() else {
            return;
        };
        let mut leitor = BufReader::new(&conexao);
        let mut linha = String::new();
        while leitor.read_line(&mut linha).unwrap_or(0) > 0 {
            let resposta = decode::<Request>(&linha).ok().and_then(responder);
            linha.clear();
            let Some(texto) = resposta.and_then(|resposta| encode(&resposta).ok()) else {
                thread::sleep(Duration::from_secs(3));
                return;
            };
            if (&conexao).write_all(texto.as_bytes()).is_err() {
                return;
            }
        }
    });
    (pasta, caminho)
}

fn atual(pedido: Request) -> Option<Reply> {
    match pedido {
        Request::Info => Some(Reply::Info {
            info: SessionInfo {
                version: "0.30.0".to_owned(),
                ..SessionInfo::default()
            },
        }),
        Request::Outputs => Some(Reply::Outputs {
            outputs: vec![OutputInfo {
                name: "DP-1".to_owned(),
                ..OutputInfo::default()
            }],
        }),
        _ => None,
    }
}

fn antigo(pedido: Request) -> Option<Reply> {
    match pedido {
        Request::Info => Some(Reply::Error {
            message: "pedido desconhecido".to_owned(),
        }),
        other => atual(other),
    }
}

fn mudo(_: Request) -> Option<Reply> {
    None
}

#[test]
fn le_informacoes_e_monitores_por_uma_conexao() {
    let (_pasta, caminho) = servidor(atual);
    let retrato = SocketCompositor::new(Some(caminho), PRAZO).snapshot();
    assert_eq!(
        retrato.info.map(|info| info.version),
        Ok("0.30.0".to_owned())
    );
    assert_eq!(retrato.outputs.map(|outputs| outputs.len()), Ok(1));
}

#[test]
fn compositor_antigo_recusa_info_mas_responde_os_monitores() {
    let (_pasta, caminho) = servidor(antigo);
    let retrato = SocketCompositor::new(Some(caminho), PRAZO).snapshot();
    assert_eq!(retrato.info, Err(Problem::Unsupported));
    assert_eq!(retrato.outputs.map(|outputs| outputs.len()), Ok(1));
}

#[test]
fn fora_de_uma_sessao_do_sparky_nao_ha_compositor() {
    let retrato = SocketCompositor::new(None, PRAZO).snapshot();
    assert_eq!(retrato.info, Err(Problem::NoCompositor));
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let sem_socket = SocketCompositor::new(Some(pasta.path().join("nada.sock")), PRAZO).snapshot();
    assert_eq!(sem_socket.outputs, Err(Problem::NoCompositor));
}

#[test]
fn compositor_mudo_estoura_o_prazo_sem_travar() {
    let (_pasta, caminho) = servidor(mudo);
    let inicio = Instant::now();
    let retrato = SocketCompositor::new(Some(caminho), PRAZO).snapshot();
    assert_eq!(retrato.info, Err(Problem::TimedOut));
    assert_eq!(retrato.outputs, Err(Problem::TimedOut));
    assert!(inicio.elapsed() < Duration::from_secs(1));
}
