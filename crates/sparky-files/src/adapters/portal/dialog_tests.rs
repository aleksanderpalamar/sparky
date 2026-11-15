use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::dialog::ProcessDialogs;
use crate::domain::choice::{Outcome, Purpose, Request};
use crate::ports::chooser::{Closer, Dialogs, Shown};

fn pedido() -> Request {
    Request {
        purpose: Purpose::Open {
            multiple: false,
            directory: false,
        },
        title: "Enviar".to_owned(),
        accept: None,
        folder: None,
        filters: Vec::new(),
        filter: None,
    }
}

fn programa(pasta: &Path, corpo: &str) -> PathBuf {
    let caminho = pasta.join("janela");
    assert!(std::fs::write(&caminho, format!("#!/bin/sh\n{corpo}\n")).is_ok());
    assert!(std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o755)).is_ok());
    caminho
}

fn mostrar(corpo: &str) -> Shown {
    let Ok(pasta) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    ProcessDialogs::new(programa(pasta.path(), corpo)).show(&pedido(), &Closer::default())
}

#[test]
fn a_janela_que_escreve_o_resultado_devolve_a_escolha() {
    let escolhido =
        mostrar(r#"cat > /dev/null; printf '{"paths":[[47,116,47,97]],"filter":null}'"#);
    assert_eq!(
        escolhido,
        Shown::Chosen(Outcome {
            paths: vec![PathBuf::from("/t/a")],
            filter: None
        })
    );
}

#[test]
fn janela_fechada_sem_resposta_cancela_e_resposta_quebrada_falha() {
    assert_eq!(mostrar("cat > /dev/null"), Shown::Cancelled);
    assert_eq!(mostrar("cat > /dev/null; printf 'lixo'"), Shown::Failed);
}

#[test]
fn fechar_o_pedido_encerra_a_janela_aberta() {
    let Ok(pasta) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let janela = ProcessDialogs::new(programa(pasta.path(), "cat > /dev/null; exec sleep 30"));
    let closer = Closer::default();
    let fechador = closer.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        fechador.close();
    });
    let inicio = Instant::now();
    assert_eq!(janela.show(&pedido(), &closer), Shown::Cancelled);
    assert!(inicio.elapsed() < Duration::from_secs(5));
}
