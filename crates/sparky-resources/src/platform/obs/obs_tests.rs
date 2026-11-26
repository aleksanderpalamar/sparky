use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tungstenite::{Message, accept};

use super::probe::ObsProbe;
use super::protocol::answer;
use super::setup::{Server, Setup, read};
use crate::context::{Sensed, Unavailable};

#[derive(Clone, Copy)]
struct Fake {
    recording: bool,
    streaming: bool,
    password: &'static str,
}

fn serve(fake: Fake) -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|error| panic!("{error}"));
    let port = listener
        .local_addr()
        .map(|address| address.port())
        .unwrap_or(0);
    let visits = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&visits);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            counted.fetch_add(1, Ordering::SeqCst);
            let Ok(mut socket) = accept(stream) else {
                continue;
            };
            let (salt, challenge) = ("sal", "desafio");
            let hello = json!({"op": 0, "d": {"rpcVersion": 1, "authentication": {"salt": salt, "challenge": challenge}}});
            let _ = socket.send(Message::text(hello.to_string()));
            let Ok(identify) = socket.read() else {
                continue;
            };
            let sent: Value =
                serde_json::from_str(identify.to_text().unwrap_or("{}")).unwrap_or_default();
            let expected = answer(fake.password, salt, challenge);
            if sent["d"]["authentication"].as_str() != Some(expected.as_str()) {
                let _ = socket.close(None);
                continue;
            }
            let _ = socket.send(Message::text(
                json!({"op": 2, "d": {"negotiatedRpcVersion": 1}}).to_string(),
            ));
            while let Ok(message) = socket.read() {
                let request: Value =
                    serde_json::from_str(message.to_text().unwrap_or("{}")).unwrap_or_default();
                let active = match request["d"]["requestType"].as_str() {
                    Some("GetRecordStatus") => fake.recording,
                    _ => fake.streaming,
                };
                let reply = json!({"op": 7, "d": {"requestType": request["d"]["requestType"], "requestId": request["d"]["requestId"], "requestStatus": {"result": true, "code": 100}, "responseData": {"outputActive": active}}});
                let _ = socket.send(Message::text(reply.to_string()));
            }
        }
    });
    (port, visits)
}

fn config(enabled: bool, port: u16, password: Option<&str>) -> String {
    json!({
        "server_enabled": enabled,
        "server_port": port,
        "auth_required": password.is_some(),
        "server_password": password.unwrap_or(""),
    })
    .to_string()
}

fn sensed(text: &str) -> Sensed {
    ObsProbe::with_setup(read(Some(text))).sense(Instant::now())
}

#[test]
fn autenticacao_confere_com_o_exemplo_do_protocolo() {
    assert_eq!(
        answer(
            "supersecretpassword",
            "lM1GncleQOaCu9lT1yeUZhFYnqhsLLP1G5lAGo3ixaI=",
            "+IxH4CnCiqpX1rM9scsNynZzbOe4KhDeYcTNS3PDaeY="
        ),
        "1Ct943GAT+6YQUUX47Ia/ncufilbe6+oD6lY+5kaCu4="
    );
}

#[test]
fn le_a_configuracao_do_servidor_do_obs() {
    assert_eq!(
        read(Some(&config(true, 4455, Some("s3gredo")))),
        Setup::Enabled(Server {
            port: 4455,
            password: Some("s3gredo".to_owned())
        })
    );
    assert_eq!(
        read(Some(&config(true, 4460, None))),
        Setup::Enabled(Server {
            port: 4460,
            password: None
        })
    );
    assert_eq!(read(Some(&config(false, 4455, None))), Setup::Disabled);
    assert_eq!(read(Some("lixo")), Setup::Disabled);
    assert_eq!(read(None), Setup::Disabled);
}

#[test]
fn obs_parado_gravando_ou_transmitindo() {
    let cases = [
        (false, false, Sensed::Idle),
        (true, false, Sensed::Capturing),
        (false, true, Sensed::Capturing),
    ];
    for (recording, streaming, expected) in cases {
        let (port, _) = serve(Fake {
            recording,
            streaming,
            password: "s3gredo",
        });
        assert_eq!(sensed(&config(true, port, Some("s3gredo"))), expected);
    }
}

#[test]
fn servidor_desligado_inacessivel_ou_senha_errada_ficam_indisponiveis() {
    assert_eq!(
        sensed(&config(false, 4455, None)),
        Sensed::Unavailable(Unavailable::Disabled)
    );
    let closed = TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map_or(1, |address| address.port());
    assert_eq!(
        sensed(&config(true, closed, None)),
        Sensed::Unavailable(Unavailable::Unreachable)
    );
    let (port, _) = serve(Fake {
        recording: true,
        streaming: false,
        password: "certa",
    });
    assert_eq!(
        sensed(&config(true, port, Some("errada"))),
        Sensed::Unavailable(Unavailable::Unauthorized)
    );
}

#[test]
fn consulta_no_maximo_a_cada_cinco_segundos() {
    let (port, visits) = serve(Fake {
        recording: false,
        streaming: false,
        password: "s3gredo",
    });
    let mut probe = ObsProbe::with_setup(read(Some(&config(true, port, Some("s3gredo")))));
    let start = Instant::now();
    probe.sense(start);
    probe.sense(start + Duration::from_secs(2));
    assert_eq!(visits.load(Ordering::SeqCst), 1);
    probe.sense(start + Duration::from_secs(6));
    assert_eq!(visits.load(Ordering::SeqCst), 2);
}
