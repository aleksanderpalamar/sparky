use std::fs;
use std::io::ErrorKind;
use std::time::{Duration, Instant};

use crate::linux::command;
use crate::linux::host::{Host, Ran};
use crate::linux::system::System;

#[test]
fn comando_devolve_a_saida_padrao() {
    let ran = command::run("sh", &["-c", "echo um; echo dois"], Duration::from_secs(5));
    assert_eq!(ran, Ran::Output("um\ndois\n".to_owned()));
}

#[test]
fn comando_lento_e_encerrado_no_prazo() {
    let inicio = Instant::now();
    let ran = command::run("sh", &["-c", "sleep 5"], Duration::from_millis(50));
    assert_eq!(ran, Ran::TimedOut);
    assert!(inicio.elapsed() < Duration::from_secs(2));
}

#[test]
fn comando_inexistente_ou_com_erro_e_distinguido() {
    let ausente = command::run(
        "sparky-fetch-comando-inexistente",
        &[],
        Duration::from_secs(1),
    );
    assert_eq!(ausente, Ran::Missing);
    let erro = command::run("sh", &["-c", "exit 3"], Duration::from_secs(5));
    assert!(matches!(erro, Ran::Failed(_)));
}

#[test]
fn sistema_le_arquivos_lista_diretorios_e_mede_o_disco() {
    let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let file = dir.path().join("arquivo");
    fs::write(&file, "conteudo").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(System.read(&file).ok().as_deref(), Some("conteudo"));
    assert_eq!(System.list(dir.path()).ok(), Some(vec![file]));
    let ausente = System
        .read(&dir.path().join("x"))
        .map_err(|error| error.kind());
    assert_eq!(ausente, Err(ErrorKind::NotFound));
    let usage = System
        .usage(dir.path())
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(usage.total > 0 && usage.free <= usage.total);
}
