use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

use super::forgetting::Forgetting;
use super::server::Server;
use super::service::Service;
use crate::domain::{Activity, Change, Mode, ServiceState};
use crate::orchestration::{Decision, Purpose, Verdict};
use crate::protocol::codec::MAX_LINE;
use crate::protocol::{Reply, Request, ask};

fn started(directory: &tempfile::TempDir) -> Option<std::path::PathBuf> {
    serving(directory, Service::default())
}

fn serving(directory: &tempfile::TempDir, mut service: Service) -> Option<std::path::PathBuf> {
    let path = directory.path().join("resources.sock");
    let server = Server::bind(&path).ok()?;
    service.ready();
    let service = Arc::new(Mutex::new(service));
    thread::spawn(move || server.serve(&service, &Forgetting::default()));
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
    assert_eq!(status.state, ServiceState::Adaptive);
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

fn decision(index: u64) -> Decision {
    Decision {
        at_seconds: 1_000_000 + index,
        activities: [Activity::Gaming, Activity::Recording]
            .into_iter()
            .collect(),
        purpose: Purpose::EaseCpu,
        process: "steamwebhelper1".to_owned(),
        change: Change::Affinity((0..32).collect()),
        stall: Some(42.5),
        verdict: Verdict::Finished(crate::safety::Outcome::ChangedElsewhere),
    }
}

#[test]
fn historico_cheio_de_decisoes_cabe_na_resposta() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let entries: Vec<Decision> = (0..100).map(decision).collect();
    let reply = Reply::Decisions {
        entries: entries.clone(),
    };
    assert!(serde_json::to_vec(&reply).is_ok_and(|bytes| bytes.len() > MAX_LINE));
    let mut service = Service::default();
    service.decided(entries.clone());
    let Some(path) = serving(&directory, service) else {
        panic!("o servidor não abriu o socket")
    };
    let Ok(Reply::Decisions { entries: received }) = ask(&path, Request::Decisions) else {
        panic!("sem decisões")
    };
    assert_eq!(received, entries);
}
