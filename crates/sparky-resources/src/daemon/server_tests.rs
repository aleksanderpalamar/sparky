use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

use super::server::Server;
use super::service::Service;
use crate::domain::{Mode, ServiceState};
use crate::protocol::codec::MAX_LINE;
use crate::protocol::{Reply, Request, ask};

fn started(directory: &tempfile::TempDir) -> Option<std::path::PathBuf> {
    let path = directory.path().join("resources.sock");
    let server = Server::bind(&path).ok()?;
    let mut service = Service::default();
    service.ready();
    let service = Arc::new(Mutex::new(service));
    thread::spawn(move || server.serve(&service));
    Some(path)
}

fn raw(path: &std::path::Path, bytes: &[u8]) -> String {
    let Ok(mut stream) = UnixStream::connect(path) else {
        return String::new();
    };
    let _ = stream.write_all(bytes);
    let mut line = String::new();
    let _ = BufReader::new(&stream).read_line(&mut line);
    line
}

#[test]
fn atende_status_e_configuracao_pelo_socket() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let Some(path) = started(&directory) else {
        panic!("o servidor não abriu o socket")
    };
    assert!(matches!(
        ask(
            &path,
            Request::SetMode {
                mode: Mode::Adaptive
            }
        ),
        Ok(Reply::Done)
    ));
    let Ok(Reply::Status(status)) = ask(&path, Request::Status) else {
        panic!("sem status")
    };
    assert_eq!(status.mode, Mode::Adaptive);
    assert_eq!(status.state, ServiceState::Observing);
}

#[test]
fn o_socket_so_e_acessivel_pelo_dono() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let Some(path) = started(&directory) else {
        panic!("o servidor não abriu o socket")
    };
    let mode = std::fs::metadata(&path)
        .map(|meta| meta.permissions().mode() & 0o777)
        .ok();
    assert_eq!(mode, Some(0o600));
}

#[test]
fn recusa_payload_grande_e_versao_errada() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let Some(path) = started(&directory) else {
        panic!("o servidor não abriu o socket")
    };
    let huge = vec![b'a'; MAX_LINE * 2];
    assert!(raw(&path, &huge).contains(r#""reason":"too_large""#));
    assert!(
        raw(&path, b"{\"version\":9,\"request\":\"status\"}\n").contains("unsupported_version")
    );
}

#[test]
fn socket_abandonado_e_substituido() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let path = directory.path().join("resources.sock");
    drop(UnixListener::bind(&path));
    assert!(Server::bind(&path).is_ok());
}

#[test]
fn nao_rouba_o_socket_de_outra_instancia_viva() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let path = directory.path().join("resources.sock");
    let Ok(_alive) = UnixListener::bind(&path) else {
        return;
    };
    assert!(Server::bind(&path).is_err());
}
