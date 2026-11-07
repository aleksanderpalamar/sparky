use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use sparky_ipc::codec::{decode, encode};
use sparky_ipc::{OutputInfo, Reply, Request, SessionInfo, SessionKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Current,
    Old,
    Silent,
}

pub fn serve(directory: &Path, kind: Kind) -> PathBuf {
    let path = directory.join("sparky.sock");
    let listener = UnixListener::bind(&path).unwrap_or_else(|error| panic!("{error}"));
    thread::spawn(move || {
        for connection in listener.incoming().flatten() {
            let mut reader = BufReader::new(&connection);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let reply = decode::<Request>(&line)
                    .ok()
                    .and_then(|request| answer(kind, request));
                line.clear();
                let Some(text) = reply.and_then(|reply| encode(&reply).ok()) else {
                    thread::sleep(Duration::from_secs(5));
                    break;
                };
                if (&connection).write_all(text.as_bytes()).is_err() {
                    break;
                }
            }
        }
    });
    path
}

fn answer(kind: Kind, request: Request) -> Option<Reply> {
    match (kind, request) {
        (Kind::Silent, _) => None,
        (Kind::Old, Request::Info) => Some(Reply::Error {
            message: "pedido desconhecido".to_owned(),
        }),
        (_, Request::Info) => Some(Reply::Info { info: info() }),
        (_, Request::Outputs) => Some(Reply::Outputs {
            outputs: vec![monitor()],
        }),
        _ => None,
    }
}

fn info() -> SessionInfo {
    SessionInfo {
        version: "0.30.0".to_owned(),
        display: "wayland-1".to_owned(),
        session: SessionKind::Real,
        theme: Some("tokyo-night".to_owned()),
        accent: "#7aa2f7".to_owned(),
        swatches: vec![
            "#1a1b26".to_owned(),
            "#f7768e".to_owned(),
            "#449dab".to_owned(),
        ],
    }
}

fn monitor() -> OutputInfo {
    OutputInfo {
        name: "DP-1".to_owned(),
        description: "LG ULTRAGEAR".to_owned(),
        width: 2048,
        height: 1152,
        mode: Some(sparky_config::OutputMode {
            width: 2560,
            height: 1440,
            refresh: 143_973,
        }),
        scale: sparky_config::OutputScale::from_factor(1.25)
            .unwrap_or(sparky_config::OutputScale::ONE),
        enabled: true,
        ..OutputInfo::default()
    }
}
