use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc;
use zbus::Connection;
use zbus::zvariant::{OwnedFd, OwnedObjectPath};

use crate::adapters::logind::{SystemLogind, WHY};
use crate::app::Event;
use crate::domain::guard::{Input, Screen};
use crate::ports::Logind;

const SESSAO: &str = "/org/freedesktop/login1/session/teste";

struct Barramento {
    processo: Child,
    endereco: String,
}

impl Barramento {
    fn iniciar() -> Self {
        let mut processo = Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--nopidfile", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|error| panic!("{error}"));
        let saida = processo
            .stdout
            .take()
            .unwrap_or_else(|| panic!("sem saída"));
        let mut endereco = String::new();
        let _ = BufReader::new(saida).read_line(&mut endereco);
        Self {
            processo,
            endereco: endereco.trim().to_owned(),
        }
    }

    async fn conectar(&self) -> Connection {
        zbus::connection::Builder::address(self.endereco.as_str())
            .map(|builder| builder.build())
            .unwrap_or_else(|error| panic!("{error}"))
            .await
            .unwrap_or_else(|error| panic!("{error}"))
    }
}

impl Drop for Barramento {
    fn drop(&mut self) {
        let _ = self.processo.kill();
        let _ = self.processo.wait();
    }
}

type Inibicao = (String, String, String, String);

#[derive(Default, Clone)]
struct Registro {
    inibicoes: Arc<Mutex<Vec<Inibicao>>>,
    dicas: Arc<Mutex<Vec<bool>>>,
}

struct Gerente(Registro);
struct Sessao(Registro);

#[zbus::interface(name = "org.freedesktop.login1.Manager")]
impl Gerente {
    #[zbus(name = "GetSessionByPID")]
    fn get_session_by_pid(&self, _pid: u32) -> OwnedObjectPath {
        OwnedObjectPath::try_from(SESSAO).unwrap_or_else(|error| panic!("{error}"))
    }

    fn inhibit(&self, what: String, who: String, why: String, mode: String) -> OwnedFd {
        if let Ok(mut inibicoes) = self.0.inibicoes.lock() {
            inibicoes.push((what, who, why, mode));
        }
        let (ponta, _outra) = UnixStream::pair().unwrap_or_else(|error| panic!("{error}"));
        OwnedFd::from(std::os::fd::OwnedFd::from(ponta))
    }
}

#[zbus::interface(name = "org.freedesktop.login1.Session")]
impl Sessao {
    fn set_locked_hint(&self, locked: bool) {
        if let Ok(mut dicas) = self.0.dicas.lock() {
            dicas.push(locked);
        }
    }
}

async fn cena() -> (Barramento, Connection, Registro, SystemLogind) {
    let barramento = Barramento::iniciar();
    let registro = Registro::default();
    let logind = zbus::connection::Builder::address(barramento.endereco.as_str())
        .and_then(|builder| builder.name("org.freedesktop.login1"))
        .and_then(|builder| builder.serve_at("/org/freedesktop/login1", Gerente(registro.clone())))
        .and_then(|builder| builder.serve_at(SESSAO, Sessao(registro.clone())))
        .unwrap_or_else(|error| panic!("{error}"))
        .build()
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    let cliente = barramento.conectar().await;
    let sistema = SystemLogind::connect(&cliente)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    (barramento, logind, registro, sistema)
}

#[tokio::test]
async fn a_dica_de_bloqueio_vai_para_a_sessao_do_daemon() {
    let (_barramento, _logind, registro, sistema) = cena().await;
    assert_eq!(sistema.hint(Screen::Locked).await, Ok(()));
    assert_eq!(sistema.hint(Screen::Open).await, Ok(()));
    assert_eq!(
        registro.dicas.lock().map(|d| d.clone()).ok(),
        Some(vec![true, false])
    );
}

#[tokio::test]
async fn pede_um_inibidor_de_atraso_para_a_suspensao() {
    let (_barramento, _logind, registro, sistema) = cena().await;
    assert!(sistema.inhibit().await.is_ok());
    let esperado = ("sleep".into(), "Sparky".into(), WHY.into(), "delay".into());
    assert_eq!(
        registro.inibicoes.lock().map(|i| i.clone()).ok(),
        Some(vec![esperado])
    );
}

#[tokio::test]
async fn os_sinais_do_logind_viram_entradas() {
    let (_barramento, logind, _registro, sistema) = cena().await;
    let (sender, mut receiver) = mpsc::channel(8);
    sistema
        .watch(sender)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    let sinais: [(&str, &str, &str, Option<bool>, Input); 4] = [
        (
            SESSAO,
            "org.freedesktop.login1.Session",
            "Lock",
            None,
            Input::LockAsked,
        ),
        (
            SESSAO,
            "org.freedesktop.login1.Session",
            "Unlock",
            None,
            Input::UnlockAsked,
        ),
        (
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
            "PrepareForSleep",
            Some(true),
            Input::Sleeping,
        ),
        (
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
            "PrepareForSleep",
            Some(false),
            Input::Woke,
        ),
    ];
    for (caminho, interface, nome, corpo, esperado) in sinais {
        let emitido = match corpo {
            Some(valor) => {
                logind
                    .emit_signal(None::<()>, caminho, interface, nome, &(valor,))
                    .await
            }
            None => {
                logind
                    .emit_signal(None::<()>, caminho, interface, nome, &())
                    .await
            }
        };
        assert!(emitido.is_ok(), "{nome}");
        let recebido = tokio::time::timeout(Duration::from_secs(3), receiver.recv()).await;
        assert_eq!(
            recebido.ok().flatten(),
            Some(Event::Seen(esperado)),
            "{nome}"
        );
    }
}
