use std::os::unix::net::UnixListener;
use std::thread;
use std::time::{Duration, Instant};

use crate::client::Client;
use crate::error::IpcError;
use crate::message::Request;

#[test]
fn servidor_que_nao_responde_estoura_o_prazo() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("sparky.sock");
    let ouvinte = UnixListener::bind(&caminho).unwrap_or_else(|error| panic!("{error}"));
    let _servidor = thread::spawn(move || {
        let conexao = ouvinte.accept();
        thread::sleep(Duration::from_secs(3));
        drop(conexao);
    });
    let inicio = Instant::now();
    let mut cliente = Client::connect_with_timeout(&caminho, Duration::from_millis(100))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(cliente.request(&Request::Info), Err(IpcError::TimedOut));
    assert!(inicio.elapsed() < Duration::from_secs(2));
}

#[test]
fn sem_socket_a_conexao_falha_sem_esperar() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("nada.sock");
    let resultado = Client::connect_with_timeout(&caminho, Duration::from_millis(100));
    assert!(matches!(resultado, Err(IpcError::Io(_))));
}
