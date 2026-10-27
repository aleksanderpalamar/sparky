use crate::{
    domain::Snapshot,
    service::{PRIVATE_NAME, PRIVATE_PATH},
};
use iced::futures::{FutureExt, SinkExt, Stream, StreamExt};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use zbus::{
    Connection, Proxy,
    proxy::{CacheProperties, MethodFlags},
};

#[derive(Clone)]
pub struct Client {
    connection: Connection,
    open: Arc<AtomicBool>,
    lease: Arc<tokio::sync::Mutex<()>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NotificationsClient")
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Dnd(bool),
    Clear,
    Dismiss(u32),
    Invoke { id: u32, key: String },
    CenterOpen(bool),
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready(Arc<Client>),
    State(Snapshot),
    Unavailable(String),
}

impl Client {
    pub fn on(connection: Connection) -> Self {
        Self {
            connection,
            open: Arc::new(AtomicBool::new(false)),
            lease: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub async fn connect() -> zbus::Result<Self> {
        let connection = zbus::connection::Builder::session()?
            .method_timeout(Duration::from_secs(3))
            .build()
            .await?;
        Ok(Self::on(connection))
    }

    async fn proxy(&self) -> zbus::Result<Proxy<'_>> {
        zbus::proxy::Builder::new(&self.connection)
            .destination(PRIVATE_NAME)?
            .path(PRIVATE_PATH)?
            .interface(PRIVATE_NAME)?
            .cache_properties(CacheProperties::No)
            .build()
            .await
    }

    pub async fn snapshot(&self) -> Result<Snapshot, String> {
        let proxy = self.proxy().await.map_err(|error| error.to_string())?;
        let data: Option<String> = proxy
            .call_with_flags("Snapshot", MethodFlags::NoAutoStart.into(), &())
            .await
            .map_err(|error| error.to_string())?;
        let data = data.ok_or("O serviço não retornou o estado")?;
        if data.len() > 12 * 1024 * 1024 {
            return Err("Resposta de notificações excedeu o limite".to_owned());
        }
        serde_json::from_str(&data).map_err(|error| error.to_string())
    }

    pub fn desire_center(&self, open: bool) {
        self.open.store(open, Ordering::SeqCst);
    }

    pub async fn sync_center(&self) -> Result<(), String> {
        let _guard = self.lease.lock().await;
        self.send(Command::CenterOpen(self.open.load(Ordering::SeqCst)))
            .await
    }

    pub async fn send(&self, command: Command) -> Result<(), String> {
        self.execute(command)
            .await
            .map_err(|error| error.to_string())
    }

    async fn execute(&self, command: Command) -> zbus::Result<()> {
        let proxy = self.proxy().await?;
        match command {
            Command::Dnd(value) => proxy.call("SetDoNotDisturb", &(value,)).await,
            Command::Clear => proxy.call("Clear", &()).await,
            Command::Dismiss(id) => proxy.call("Dismiss", &(id,)).await,
            Command::Invoke { id, key } => proxy.call("Invoke", &(id, key)).await,
            Command::CenterOpen(value) => proxy.call("CenterOpen", &(value,)).await,
        }
    }
}

pub fn events() -> impl Stream<Item = Event> {
    iced::stream::channel(8, async |mut output| {
        loop {
            let result = monitor(&mut output).await;
            if output.is_closed() {
                return;
            }
            let error = result.err().map_or_else(
                || "Serviço de notificações encerrado".to_owned(),
                |error| error.to_string(),
            );
            if output.send(Event::Unavailable(error)).await.is_err() {
                return;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    })
}

async fn monitor(output: &mut iced::futures::channel::mpsc::Sender<Event>) -> Result<(), String> {
    let client = Arc::new(Client::connect().await.map_err(|error| error.to_string())?);
    let proxy = client.proxy().await.map_err(|error| error.to_string())?;
    let mut signals = proxy
        .receive_signal("Changed")
        .await
        .map_err(|error| error.to_string())?;
    let mut owners = proxy
        .receive_owner_changed()
        .await
        .map_err(|error| error.to_string())?;
    let snapshot = client.snapshot().await?;
    output
        .send(Event::Ready(client.clone()))
        .await
        .map_err(|_| "Canal fechado")?;
    let mut revision = snapshot.revision;
    output
        .send(Event::State(snapshot))
        .await
        .map_err(|_| "Canal fechado")?;
    loop {
        let event = tokio::select! {
            _ = owners.next() => return Err("O serviço de notificações mudou".to_owned()),
            event = tokio::time::timeout(Duration::from_secs(30), signals.next()) => event,
        };
        if matches!(event, Ok(None)) {
            return Err("Conexão encerrada".to_owned());
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
        while signals.next().now_or_never().flatten().is_some() {}
        let snapshot = client.snapshot().await?;
        if revision == snapshot.revision {
            continue;
        }
        revision = snapshot.revision;
        if output.send(Event::State(snapshot)).await.is_err() {
            return Ok(());
        }
    }
}
