use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use futures_util::future::BoxFuture;
use tokio::sync::{Notify, watch};
use zbus::{Connection, Proxy, zvariant::OwnedValue};

use super::{AUTHORITY, INTERFACE, PATH, Runner, agent::Agent, shared::Shared};
use crate::domain::{Outcome, Request};

pub struct Bus {
    child: Child,
    pub address: String,
}

impl Bus {
    pub fn start() -> Self {
        let mut child = Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--nopidfile", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let output = child.stdout.take().unwrap();
        let mut address = String::new();
        BufReader::new(output).read_line(&mut address).unwrap();
        assert!(address.starts_with("unix:"));
        Self {
            child,
            address: address.trim().to_owned(),
        }
    }

    pub async fn connection(&self) -> Connection {
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            zbus::connection::Builder::address(self.address.as_str())
                .unwrap()
                .build(),
        )
        .await
        .unwrap()
        .unwrap()
    }
}

impl Drop for Bus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    Accept,
    Wait,
}

pub struct FakeRunner {
    pub calls: Mutex<Vec<Request>>,
    pub entered: Arc<Notify>,
    pub mode: Mode,
}

impl Runner for FakeRunner {
    fn run(
        &self,
        request: Request,
        mut cancellation: watch::Receiver<bool>,
    ) -> BoxFuture<'static, Outcome> {
        self.calls.lock().unwrap().push(request);
        self.entered.notify_one();
        let mode = self.mode;
        Box::pin(async move {
            match mode {
                Mode::Accept => Outcome::Authorized,
                Mode::Wait => {
                    let _ = cancellation.changed().await;
                    Outcome::Cancelled
                }
            }
        })
    }
}

pub struct Scene {
    pub authority: Connection,
    pub stranger: Connection,
    pub server: Connection,
    pub runner: Arc<FakeRunner>,
    pub shared: Arc<Shared>,
    pub _bus: Bus,
}

impl Scene {
    pub async fn new(mode: Mode) -> Self {
        let bus = Bus::start();
        let authority = bus.connection().await;
        authority.request_name(AUTHORITY).await.unwrap();
        let stranger = bus.connection().await;
        let runner = Arc::new(FakeRunner {
            calls: Mutex::default(),
            entered: Arc::new(Notify::new()),
            mode,
        });
        let shared = Shared::new(runner.clone());
        let server = zbus::connection::Builder::address(bus.address.as_str())
            .unwrap()
            .serve_at(PATH, Agent(shared.clone()))
            .unwrap()
            .build()
            .await
            .unwrap();
        Self {
            authority,
            stranger,
            server,
            runner,
            shared,
            _bus: bus,
        }
    }

    pub async fn proxy<'a>(&'a self, connection: &'a Connection) -> Proxy<'a> {
        zbus::proxy::Builder::<Proxy<'a>>::new(connection)
            .destination(self.server.unique_name().unwrap().as_str())
            .unwrap()
            .path(PATH)
            .unwrap()
            .interface(INTERFACE)
            .unwrap()
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await
            .unwrap()
    }

    pub async fn begin(&self, connection: &Connection, cookie: &str) -> zbus::Result<()> {
        let identities = vec![(
            "unix-user",
            [("uid".to_owned(), OwnedValue::from(0u32))]
                .into_iter()
                .collect::<std::collections::HashMap<_, _>>(),
        )];
        let details = std::collections::HashMap::<String, String>::new();
        let proxy = self.proxy(connection).await;
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            proxy.call(
                "BeginAuthentication",
                &(
                    "org.fixture.mount",
                    "Autentique para montar",
                    "drive-harddisk",
                    details,
                    cookie,
                    identities,
                ),
            ),
        )
        .await
        .expect("o pedido privado deve terminar")
    }

    pub async fn cancel(&self, connection: &Connection, cookie: &str) -> zbus::Result<()> {
        self.proxy(connection)
            .await
            .call("CancelAuthentication", &(cookie,))
            .await
    }
}
