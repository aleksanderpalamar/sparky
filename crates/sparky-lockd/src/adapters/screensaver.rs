use std::sync::{Arc, Mutex};

use futures_util::stream::StreamExt;
use tokio::sync::mpsc::Sender;
use zbus::fdo::{self, DBusProxy};
use zbus::message::Header;
use zbus::{Connection, interface};

use crate::app::Event;
use crate::domain::guard::Input;
use crate::domain::inhibitions::{Hold, Inhibitions};

pub const NAME: &str = "org.freedesktop.ScreenSaver";
const PATHS: [&str; 2] = ["/org/freedesktop/ScreenSaver", "/ScreenSaver"];
const BROKEN: &str = "os pedidos de tela acesa ficaram inacessíveis";

#[derive(Clone)]
pub struct ScreenSaver {
    inhibitions: Arc<Mutex<Inhibitions>>,
    sender: Sender<Event>,
}

impl ScreenSaver {
    fn new(sender: Sender<Event>) -> Self {
        Self {
            inhibitions: Arc::new(Mutex::new(Inhibitions::default())),
            sender,
        }
    }

    fn with<T>(&self, change: impl FnOnce(&mut Inhibitions) -> T) -> Option<T> {
        let mut inhibitions = self.inhibitions.lock().ok()?;
        Some(change(&mut inhibitions))
    }

    async fn report(&self, change: Option<Hold>) {
        if let Some(hold) = change {
            let _ = self.sender.send(Event::Seen(Input::Idle(hold))).await;
        }
    }
}

#[interface(name = "org.freedesktop.ScreenSaver")]
impl ScreenSaver {
    async fn inhibit(
        &self,
        #[zbus(header)] header: Header<'_>,
        _application_name: String,
        _reason_for_inhibit: String,
    ) -> fdo::Result<u32> {
        let owner = header.sender().map(ToString::to_string).unwrap_or_default();
        let Some((cookie, change)) = self.with(|inhibitions| inhibitions.inhibit(&owner)) else {
            return Err(fdo::Error::Failed(BROKEN.to_owned()));
        };
        self.report(change).await;
        Ok(cookie)
    }

    async fn un_inhibit(&self, cookie: u32) {
        let change = self
            .with(|inhibitions| inhibitions.release(cookie))
            .flatten();
        self.report(change).await;
    }
}

pub async fn serve(connection: &Connection, sender: Sender<Event>) -> zbus::Result<()> {
    let service = ScreenSaver::new(sender);
    for path in PATHS {
        connection.object_server().at(path, service.clone()).await?;
    }
    let mut owners = DBusProxy::new(connection)
        .await?
        .receive_name_owner_changed()
        .await?;
    connection.request_name(NAME).await?;
    tokio::spawn(async move {
        while let Some(signal) = owners.next().await {
            let Ok(args) = signal.args() else {
                continue;
            };
            if args.new_owner().is_some() {
                continue;
            }
            let name = args.name().to_string();
            let change = service
                .with(|inhibitions| inhibitions.forget(&name))
                .flatten();
            service.report(change).await;
        }
    });
    Ok(())
}
