use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::Duration;

use super::{Runner, process::PromptProcess};
use crate::domain::{Identity, Outcome, Request, Secret};

fn request() -> Request {
    Request {
        action_id: "org.fixture.mount".to_owned(),
        message: "Autentique".to_owned(),
        cookie: Secret::new("cookie-pelo-pipe".to_owned()),
        identities: vec![Identity {
            uid: 0,
            name: "root".to_owned(),
        }],
    }
}

fn fixture(directory: &std::path::Path, script: &str) -> PromptProcess {
    let path = directory.join("prompt");
    std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    PromptProcess(path)
}

#[tokio::test]
async fn cookie_vai_pelo_pipe_e_nao_pelos_argumentos() {
    let directory = tempfile::tempdir().unwrap();
    let runner = fixture(
        directory.path(),
        r#"
test "$#" = 1 && test "$1" = --authenticate || exit 2
IFS= read -r packet || exit 2
case "$packet" in *cookie-pelo-pipe*) exit 0;; *) exit 2;; esac
"#,
    );
    let (_sender, cancellation) = tokio::sync::watch::channel(false);
    assert_eq!(
        runner.run(request(), cancellation).await,
        Outcome::Authorized
    );
}

#[tokio::test]
async fn cancelar_fecha_o_pipe_e_recolhe_o_processo_da_janela() {
    let directory = tempfile::tempdir().unwrap();
    let runner = Arc::new(fixture(
        directory.path(),
        r#"
IFS= read -r packet || exit 2
printf '%s' "$$" > "$0.pid"
IFS= read -r cancelled
exit 1
"#,
    ));
    let (sender, cancellation) = tokio::sync::watch::channel(false);
    let waiting = runner.run(request(), cancellation);
    let pid_file = directory.path().join("prompt.pid");
    let cancel = async {
        while !pid_file.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        sender.send(true).unwrap();
    };
    let (outcome, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(waiting, cancel)
    })
    .await
    .unwrap();
    assert_eq!(outcome, Outcome::Cancelled);
    let pid = std::fs::read_to_string(pid_file).unwrap();
    assert!(!std::path::Path::new("/proc").join(pid).exists());
}

#[tokio::test]
async fn cancelamento_previo_nao_abre_processo_e_falha_nao_autoriza() {
    let directory = tempfile::tempdir().unwrap();
    let runner = fixture(directory.path(), "exit 23");
    let (sender, cancellation) = tokio::sync::watch::channel(true);
    assert_eq!(
        runner.run(request(), cancellation).await,
        Outcome::Cancelled
    );
    sender.send(false).unwrap_err();
    let (_sender, cancellation) = tokio::sync::watch::channel(false);
    assert_eq!(runner.run(request(), cancellation).await, Outcome::Denied);
}
