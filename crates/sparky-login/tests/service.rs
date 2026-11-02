use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sparky_login::domain::memory::Memory;
use sparky_login::service::serve;
use sparky_login::service::settings::Settings;
use sparky_login_core::codec::{decode, encode};
use sparky_login_core::protocol::{Refusal, Reply, Request};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use zeroize::Zeroizing;

const WORKER: &str = r#"#!/bin/sh
read -r job
case "$job" in
  *'"job":"greeter"'*) sleep 3 ;;
  *'"password":"segredo"'*) printf '{"report":"authenticated"}\n'; read -r order ;;
  *) printf '{"report":"refused","refusal":{"kind":"denied"}}\n' ;;
esac
"#;

struct Cena {
    dir: TempDir,
    settings: Settings,
}

fn cena() -> Cena {
    let dir = tempfile::Builder::new()
        .prefix("servico")
        .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
        .unwrap_or_else(|error| panic!("{error}"));
    let worker = dir.path().join("worker");
    escrever(&worker, WORKER);
    fs::set_permissions(&worker, fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|error| panic!("{error}"));
    let sessions = dir.path().join("dados/wayland-sessions");
    fs::create_dir_all(&sessions).unwrap_or_else(|error| panic!("{error}"));
    escrever(
        &sessions.join("teste.desktop"),
        "[Desktop Entry]\nName=Teste\nExec=true\n",
    );
    let user = nix::unistd::User::from_uid(nix::unistd::getuid())
        .ok()
        .flatten()
        .map(|user| user.name)
        .unwrap_or_default();
    let settings = Settings {
        vt: 63,
        socket: dir.path().join("run/greeter.sock"),
        memory: dir.path().join("memory.json"),
        greeter_user: user,
        greeter_command: vec!["true".into()],
        worker,
        data_dirs: Some(dir.path().join("dados").to_string_lossy().into_owned()),
    };
    Cena { dir, settings }
}

fn escrever(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

async fn conectar(socket: &PathBuf) -> (BufReader<OwnedReadHalf>, OwnedWriteHalf) {
    for _ in 0..100 {
        if let Ok(stream) = UnixStream::connect(socket).await {
            let (reader, writer) = stream.into_split();
            return (BufReader::new(reader), writer);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("o daemon não abriu o socket");
}

async fn pedir(
    conexao: &mut (BufReader<OwnedReadHalf>, OwnedWriteHalf),
    request: &Request,
) -> Reply {
    let line = encode(request).unwrap_or_else(|error| panic!("{error}"));
    conexao
        .1
        .write_all(line.as_bytes())
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    let mut resposta = String::new();
    let lida =
        tokio::time::timeout(Duration::from_secs(5), conexao.0.read_line(&mut resposta)).await;
    assert!(
        matches!(lida, Ok(Ok(read)) if read > 0),
        "sem resposta para {request:?}"
    );
    decode(&resposta).unwrap_or_else(|error| panic!("{error}"))
}

fn login(senha: &str, session: &str) -> Request {
    Request::Login {
        username: "palamar".into(),
        password: Zeroizing::new(senha.into()),
        session: session.into(),
    }
}

#[tokio::test]
async fn a_tela_conversa_com_o_daemon_e_o_worker_confere_a_senha() {
    let cena = cena();
    let socket = cena.settings.socket.clone();
    let memory = cena.settings.memory.clone();
    let daemon = tokio::spawn(serve(cena.settings.clone()));
    let mut conexao = conectar(&socket).await;
    assert_eq!(
        pedir(&mut conexao, &Request::Hello).await,
        Reply::Welcome {
            last_user: None,
            last_session: None
        }
    );
    let ausente = Refusal::Unavailable("a sessão gnome não está instalada".into());
    assert_eq!(
        pedir(&mut conexao, &login("segredo", "gnome")).await,
        Reply::Refused { refusal: ausente }
    );
    assert_eq!(
        pedir(&mut conexao, &login("errada", "teste")).await,
        Reply::Refused {
            refusal: Refusal::Denied
        }
    );
    assert_eq!(
        pedir(&mut conexao, &login("segredo", "teste")).await,
        Reply::LoggedIn
    );
    let lembrado: Option<Memory> = fs::read_to_string(&memory)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok());
    assert_eq!(
        lembrado,
        Some(Memory {
            last_user: Some("palamar".into()),
            last_session: Some("teste".into())
        })
    );
    assert_eq!(
        fs::metadata(&socket)
            .map(|meta| meta.permissions().mode() & 0o777)
            .ok(),
        Some(0o600)
    );
    daemon.abort();
    drop(cena.dir);
}
